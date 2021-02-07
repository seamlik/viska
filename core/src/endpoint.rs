use crate::packet::ResponseWindow;
use crate::pki::CanonicalId;
use crate::util::TaskSink;
use crate::util::TOKIO_02;
use crate::Connection;
use crate::ConnectionError;
use blake3::Hash;
use futures_channel::mpsc::UnboundedSender;
use futures_core::Stream;
use futures_util::FutureExt;
use futures_util::SinkExt;
use futures_util::StreamExt;
use quinn::CertificateChain;
use quinn::Endpoint;
use quinn::Incoming;
use quinn::IncomingBiStreams;
use quinn::NewConnection;
use rustls::internal::msgs::handshake::DistinguishedNames;
use rustls::ClientCertVerified;
use rustls::ClientCertVerifier;
use rustls::RootCertStore;
use rustls::ServerCertVerified;
use rustls::ServerCertVerifier;
use rustls::TLSError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;
use webpki::DNSName;
use webpki::DNSNameRef;

const ALPN_PROTOCOL: &str = "viska";

pub struct Config<'a> {
    pub certificate: &'a [u8],
    pub key: &'a [u8],
}

/// QUIC endpoint that binds to all local interfaces in the network.
///
/// This also serves as the main endpoint that is used to connect with remote [Node](crate::Node)s.
struct LocalEndpoint {
    account_id: Hash,
    quic: Endpoint,
    client_config: quinn::ClientConfig,
}

impl LocalEndpoint {
    pub fn start(
        config: &Config,
        verifier: Arc<CertificateVerifier>,
    ) -> Result<(Self, Incoming), Error> {
        let cert_chain = CertificateChain::from_certs(std::iter::once(
            quinn::Certificate::from_der(&config.certificate)?,
        ));
        let quinn_key = quinn::PrivateKey::from_der(config.key)?;

        let mut transport_config = quinn::TransportConfig::default();
        transport_config.stream_window_uni(0);
        let transport_config = Arc::new(transport_config);

        // Server config
        let mut server_config_builder = quinn::ServerConfigBuilder::default();
        server_config_builder.protocols(&[ALPN_PROTOCOL.as_bytes()]);
        server_config_builder.certificate(cert_chain.clone(), quinn_key)?;
        let mut server_config = server_config_builder.build();
        server_config.transport = transport_config.clone();

        // Client config
        let rustls_key = rustls::PrivateKey(config.key.into());
        let mut client_config_builder = quinn::ClientConfigBuilder::default();
        client_config_builder.protocols(&[ALPN_PROTOCOL.as_bytes()]);
        let mut client_config = client_config_builder.build();
        let client_tls_config = Arc::get_mut(&mut client_config.crypto).unwrap();
        client_tls_config.set_single_client_cert(cert_chain.into_iter().collect(), rustls_key)?;
        client_tls_config
            .dangerous()
            .set_certificate_verifier(verifier);
        client_config.transport = transport_config;

        let mut endpoint_builder = Endpoint::builder();
        endpoint_builder.listen(server_config);
        let socket = UdpSocket::bind("[::]:0")?;
        let (endpoint, incoming) = TOKIO_02.enter(|| endpoint_builder.with_socket(socket))?;
        log::info!(
            "Started local QUIC endpoint on port {}",
            endpoint.local_addr().unwrap().port()
        );

        Ok((
            Self {
                account_id: config.certificate.canonical_id(),
                quic: endpoint,
                client_config,
            },
            incoming,
        ))
    }

    async fn connect(&self, addr: &SocketAddr) -> Result<NewConnection, ConnectionError> {
        log::info!("Outgoing connection to {}", addr);
        let connecting = TOKIO_02.enter(|| {
            self.quic
                .connect_with(self.client_config.clone(), addr, "viska.local")
        });
        connecting?.await.map_err(Into::into)
    }

    pub fn local_port(&self) -> std::io::Result<u16> {
        Ok(self.quic.local_addr()?.port())
    }
}

impl Drop for LocalEndpoint {
    fn drop(&mut self) {
        self.quic.close(0_u8.into(), &[]);
    }
}

/// Error when starting a QUIC endpoint.
#[derive(thiserror::Error, Debug)]
#[error("Failed to start a QUIC endpoint")]
pub enum Error {
    CryptoMaterial(#[from] quinn::ParseError),
    Grpc(#[from] tonic::transport::Error),
    Quic(#[from] quinn::EndpointError),
    Socket(#[from] std::io::Error),
    TlsConfiguration(#[from] rustls::TLSError),
}

pub trait ConnectionInfo {
    /// Gets the account ID of the [Node](crate::Node) who opened this connection.
    ///
    /// Consult [AuthenticationData](quinn::crypto::rustls::AuthenticationData) for the option-ness.
    fn account_id(&self) -> Option<Hash>;
    fn remote_address(&self) -> SocketAddr;
}

impl ConnectionInfo for quinn::Connection {
    fn remote_address(&self) -> SocketAddr {
        self.remote_address()
    }

    fn account_id(&self) -> Option<Hash> {
        self.authentication_data()
            .peer_certificates
            .and_then(|chain| chain.iter().next().map(|cert| cert.canonical_id()))
    }
}

pub struct ConnectionManager {
    connections: Arc<tokio::sync::RwLock<HashMap<Uuid, Arc<Connection>>>>,
    endpoint: LocalEndpoint,
    response_window_sink: UnboundedSender<ResponseWindow>,
    task_sink: TaskSink,
}

impl ConnectionManager {
    pub fn new(
        config: &Config,
        verifier: Arc<CertificateVerifier>,
        response_window_sink: UnboundedSender<ResponseWindow>,
    ) -> Result<(Self, impl Future<Output = ()>), Error> {
        let (task_sink, dynamic_task) = TaskSink::new();
        let (endpoint, incoming) = LocalEndpoint::start(config, verifier)?;

        let instance = Self {
            endpoint,
            response_window_sink: response_window_sink.clone(),
            connections: Default::default(),
            task_sink: task_sink.clone(),
        };
        let account_id = instance.endpoint.account_id;
        let connections = instance.connections.clone();

        // Process incoming connections
        let incoming_connections_task = incoming.for_each_concurrent(None, move |connecting| {
            let connections = connections.clone();
            let task_sink = task_sink.clone();
            let response_window_sink = response_window_sink.clone();
            async move {
                match connecting.await {
                    Ok(new_connection) => {
                        Self::add(
                            new_connection,
                            connections.clone(),
                            response_window_sink,
                            task_sink.clone(),
                            account_id,
                        )
                        .await;
                    }
                    Err(err) => log::error!("Failed to accept an incoming connection: {:?}", err),
                }
            }
        });

        let task = async {
            futures_util::join!(dynamic_task, incoming_connections_task);
        };

        Ok((instance, task))
    }

    async fn add(
        new_connection: NewConnection,
        connections: Arc<tokio::sync::RwLock<HashMap<Uuid, Arc<Connection>>>>,
        response_window_sink: UnboundedSender<ResponseWindow>,
        task_sink: TaskSink,
        account_id: Hash,
    ) -> Arc<Connection> {
        let connection_id = Uuid::new_v4();
        let connection = Arc::<Connection>::new(Connection {
            quic: new_connection.connection,
            id: connection_id,
        });
        connections
            .write()
            .await
            .insert(connection.id, connection.clone());

        // Create ResponseWindow
        let connection_clone = connection.clone();
        let bi_streams = new_connection.bi_streams;
        let response_windows_creator_task = bi_streams
            .for_each_concurrent(None, move |incoming| {
                Self::consume_bi_streams(
                    incoming,
                    connection_clone.clone(),
                    response_window_sink.clone(),
                )
            })
            .then(move |_| async move {
                // Auto-remove the connection after it is closed (stream ends)
                connections.write().await.remove(&connection_id);
            });
        task_sink.submit(response_windows_creator_task);

        log::info!(
            "Connected to {} {:?}",
            if Some(account_id) == connection.account_id() {
                "Device"
            } else {
                "Peer"
            },
            &connection
        );
        connection
    }

    async fn consume_bi_streams(
        incoming: <IncomingBiStreams as Stream>::Item,
        connection: Arc<Connection>,
        mut response_window_sink: impl SinkExt<ResponseWindow> + Unpin,
    ) {
        match incoming {
            Ok((sender, receiver)) => {
                let window = ResponseWindow::new(connection, sender, receiver).await;
                if let Some(w) = window {
                    let _ = response_window_sink.send(w).await;
                };
            }
            Err(err) => {
                log::error!("Error on {:?}: {:?}", &connection, err);
            }
        };
    }

    pub async fn connect(&self, addr: &SocketAddr) -> Result<Arc<Connection>, ConnectionError> {
        Ok(Self::add(
            self.endpoint.connect(addr).await?,
            self.connections.clone(),
            self.response_window_sink.clone(),
            self.task_sink.clone(),
            self.endpoint.account_id,
        )
        .await)
    }

    pub fn local_port(&self) -> std::io::Result<u16> {
        self.endpoint.local_port()
    }
}

pub struct CertificateVerifier {
    pub account_id: Hash,
    peer_whitelist: RwLock<HashSet<Vec<u8>>>,
    peer_blacklist: RwLock<HashSet<Vec<u8>>>,
}

impl CertificateVerifier {
    pub fn new(account_id: Hash) -> Self {
        Self {
            account_id,
            peer_whitelist: Default::default(),
            peer_blacklist: Default::default(),
        }
    }

    fn verify(&self, presented_certs: &[rustls::Certificate]) -> Result<(), TLSError> {
        // TODO: Check expiration
        match presented_certs {
            [cert] => {
                let peer_id = cert.canonical_id();
                if self.account_id == peer_id || self.peer_is_allowed(peer_id) {
                    log::info!("Peer {} is known, accepting connection.", peer_id.to_hex());
                    Ok(())
                } else {
                    Err(TLSError::General("Unrecognized certificate ID".into()))
                }
            }
            [] => Err(TLSError::NoCertificatesPresented),
            _ => Err(TLSError::PeerMisbehavedError(
                "Only 1 certificate is allowed in the chain".into(),
            )),
        }
    }

    fn peer_is_allowed(&self, id: Hash) -> bool {
        let id_bytes = crate::database::bytes_from_hash(id);
        let whitelist = self.peer_whitelist.read().unwrap();
        if !whitelist.is_empty() && whitelist.contains(&id_bytes) {
            true
        } else {
            !self.peer_blacklist.read().unwrap().contains(&id_bytes)
        }
    }

    pub fn set_rules(
        &self,
        peer_whitelist: impl IntoIterator<Item = Vec<u8>>,
        peer_blacklist: impl IntoIterator<Item = Vec<u8>>,
    ) {
        let mut whitelist = self.peer_whitelist.write().unwrap();
        whitelist.clear();
        whitelist.extend(peer_whitelist);

        let mut blacklist = self.peer_blacklist.write().unwrap();
        blacklist.clear();
        blacklist.extend(peer_blacklist);
    }
}

impl ClientCertVerifier for CertificateVerifier {
    fn client_auth_root_subjects(&self, _: Option<&DNSName>) -> Option<DistinguishedNames> {
        Some(Default::default())
    }
    fn verify_client_cert(
        &self,
        presented_certs: &[rustls::Certificate],
        _: Option<&DNSName>,
    ) -> Result<ClientCertVerified, TLSError> {
        self.verify(presented_certs)
            .map(|_| ClientCertVerified::assertion())
    }
}

impl ServerCertVerifier for CertificateVerifier {
    fn verify_server_cert(
        &self,
        _: &RootCertStore,
        presented_certs: &[rustls::Certificate],
        _: DNSNameRef,
        _: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        self.verify(presented_certs)
            .map(|_| ServerCertVerified::assertion())
    }
}
