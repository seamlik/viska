#![feature(proc_macro_hygiene)]

#[path = "../../target/riko/viska.rs"]
#[riko::ignore]
pub mod bridge;

pub mod daemon;
mod endpoint;
mod handler;
mod packet;
pub mod pki;
pub mod proto;
mod util;

pub mod transaction {
    tonic::include_proto!("viska.transaction");
}

use crate::endpoint::CertificateVerifier;
use endpoint::ConnectionInfo;
use endpoint::ConnectionManager;
use endpoint::LocalEndpoint;
use futures::prelude::*;
use handler::DeviceHandler;
use handler::Handler;
use handler::PeerHandler;
use http::StatusCode;
use packet::ResponseWindow;
use pki::Certificate;
use pki::CertificateId;
use prost::DecodeError;
use prost::Message as _;
use proto::Request;
use proto::Response;
use quinn::ReadToEndError;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// The protagonist.
pub struct Node {
    connection_manager: Arc<ConnectionManager>,
    grpc_port: u16,
}

impl Node {
    /// Constructor.
    ///
    /// The returned [Future] is a handle for awaiting all operations. Everything is running once
    /// this method finishes.
    pub fn start(
        certificate: &[u8],
        key: &[u8],
        platform_grpc_port: u16,
        enable_certificate_verification: bool,
    ) -> Result<(Self, JoinHandle<()>), EndpointError> {
        let certificate_verifier = Arc::new(CertificateVerifier {
            enabled: enable_certificate_verification,
            account_id: certificate.id(),
            peer_whitelist: Default::default(),
        });

        // Start gRPC server
        let node_grpc_port = daemon::StandardNode::start(certificate_verifier.clone());

        let config = endpoint::Config { certificate, key };
        let (window_sender, window_receiver) =
            futures::channel::mpsc::unbounded::<ResponseWindow>();

        // Handle requests
        let account_id = certificate.id();
        tokio::spawn(window_receiver.for_each(move |window| {
            let handler: Box<dyn Handler + Send + Sync> = if window.account_id() == Some(account_id)
            {
                Box::new(DeviceHandler)
            } else {
                Box::new(PeerHandler { platform_grpc_port })
            };
            async move {
                let response = match handler.handle(&window).await {
                    Ok(r) => r,
                    Err(err) => err.into(),
                };
                window
                    .send_response(response)
                    .await
                    .unwrap_or_else(|err| log::error!("Error sending a response: {:?}", err));
            }
        }));

        let (endpoint, incomings) = LocalEndpoint::start(&config, certificate_verifier)?;
        let connection_manager = Arc::new(ConnectionManager::new(endpoint, window_sender));

        // Process incoming connections
        let connection_manager_cloned = connection_manager.clone();
        let task = tokio::spawn(incomings.for_each(move |connecting| {
            let connection_manager_cloned = connection_manager_cloned.clone();
            tokio::spawn(async {
                match connecting.await {
                    Ok(new_connection) => {
                        connection_manager_cloned.add(new_connection).await;
                    }
                    Err(err) => log::error!("Failed to accept an incoming connection: {:?}", err),
                }
            });
            async {}
        }));

        println!("Started Viska node with account {}", account_id.to_hex());
        Ok((
            Self {
                connection_manager,
                grpc_port: node_grpc_port,
            },
            task,
        ))
    }

    pub async fn connect(&self, addr: &SocketAddr) -> Result<Arc<Connection>, ConnectionError> {
        self.connection_manager.clone().connect(addr).await
    }

    /// Gets the port to its gRPC server.
    pub fn grpc_port(&self) -> u16 {
        self.grpc_port
    }
}

/// Connection to a remote [Node].
pub struct Connection {
    id: Uuid,
    quic: quinn::Connection,
}

impl Connection {
    /// Sends a [Request] and awaits for its [Response].
    ///
    /// # Note
    ///
    /// If the remote [Node] is sending a packet larger than a threashold, this [Connection] will
    /// be closed immediately.
    pub async fn request(&self, request: &Request) -> Result<Response, RequestError> {
        let (mut sender, receiver) = self.quic.open_bi().await?;
        let mut raw_request = Vec::<u8>::new();
        request
            .encode(&mut raw_request)
            .unwrap_or_else(|err| panic!("Failed to encode a request: {}", err));

        sender.write_all(&raw_request).await?;
        sender.finish().await?;

        match receiver.read_to_end(packet::MAX_PACKET_SIZE_BYTES).await {
            Ok(raw_response) => {
                let response = Response::decode(raw_response.as_slice())?;
                log::debug!("Received response: {:?}", &response);
                Ok(response)
            }
            Err(err) => match err {
                ReadToEndError::TooLong => {
                    self.close(StatusCode::PAYLOAD_TOO_LARGE);
                    Err(RequestError::ResponseTooLong)
                }
                ReadToEndError::Read(inner) => Err(inner.into()),
            },
        }
    }

    pub fn close(&self, code: StatusCode) {
        log::info!("Closing connection to {}", self.remote_address());
        self.quic.close(code.as_u16().into(), &[]);
    }
}

impl ConnectionInfo for Connection {
    fn remote_address(&self) -> SocketAddr {
        self.quic.remote_address()
    }

    fn account_id(&self) -> Option<CertificateId> {
        self.quic.account_id()
    }
}

impl Debug for Connection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Connection")
            .field("connection_id", &self.id)
            .field("remote_address", &self.remote_address())
            .field(
                "account_id",
                &self
                    .account_id()
                    .map(|hash| hash.to_hex().to_string())
                    .unwrap_or_else(|| "None".into()),
            )
            .finish()
    }
}

/// Error when sending a [Request] and waiting for its [Response].
#[derive(Error, Debug)]
#[error("Failed to complete a request-response lifecycle")]
pub enum RequestError {
    BadResponse(#[from] DecodeError),
    Connection(#[from] quinn::ConnectionError),
    Read(#[from] quinn::ReadError),
    ResponseTooLong,
    Write(#[from] quinn::WriteError),
}

/// Error when connecting to a remote [Node].
#[derive(Error, Debug)]
#[error("Failed to connect to a remote node")]
pub enum ConnectionError {
    Start(#[from] quinn::ConnectError),
    Handshake(#[from] quinn::ConnectionError),
}

/// Error when starting a QUIC endpoint.
#[derive(Error, Debug)]
#[error("Failed to start a QUIC endpoint")]
pub enum EndpointError {
    CryptoMaterial(#[from] quinn::ParseError),
    TlsConfiguration(#[from] rustls::TLSError),
    Quic(#[from] quinn::EndpointError),
    Socket(#[from] std::io::Error),
}
