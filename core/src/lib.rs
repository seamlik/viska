#![feature(once_cell)]
#![feature(proc_macro_hygiene)]

#[path = "../../target/riko/viska.rs"]
#[riko::ignore]
pub mod bridge;

mod changelog;
pub mod daemon;
pub mod database;
mod endpoint;
mod handler;
mod mock_profile;
mod packet;
pub mod pki;
pub mod proto;
pub mod util;

use crate::daemon::platform_client::PlatformClient;
use crate::daemon::GrpcClient;
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
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::lazy::SyncLazy;
use std::net::SocketAddr;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tonic::transport::Channel;
use uuid::Uuid;

static CURRENT_NODE_HANDLE: AtomicI32 = AtomicI32::new(0);
static NODES: SyncLazy<Mutex<HashMap<i32, Node>>> = SyncLazy::new(|| Default::default());
static TOKIO: SyncLazy<Mutex<Runtime>> = SyncLazy::new(|| Mutex::new(Runtime::new().unwrap()));

/// Starts a [Node].
///
/// # Returns
///
/// The handle for use with [stop].
#[riko::fun]
pub fn start(
    certificate: ByteBuf,
    key: ByteBuf,
    platform_grpc_port: u16,
    node_grpc_port: u16,
) -> Result<i32, EndpointError> {
    let handle = CURRENT_NODE_HANDLE.fetch_add(1, Ordering::SeqCst);
    let (node, _) = TOKIO.lock().unwrap().block_on(Node::start(
        &certificate,
        &key,
        platform_grpc_port,
        node_grpc_port,
        true,
    ))?;
    NODES.lock().unwrap().insert(handle, node);
    Ok(handle)
}

/// Stops a [Node].
///
/// # Parameters
///
/// * `handle`: The handle returned from [start].
#[riko::fun]
pub fn stop(handle: i32) {
    log::info!("Stopping Node");
    NODES.lock().unwrap().remove(&handle);
}

/// The protagonist.
pub struct Node {
    connection_manager: Arc<ConnectionManager>,
}

impl Node {
    /// Constructor.
    ///
    /// The returned [Future] is a handle for awaiting all operations. Everything is running once
    /// this method finishes.
    pub async fn start(
        certificate: &[u8],
        key: &[u8],
        platform_grpc_port: u16,
        node_grpc_port: u16,
        enable_certificate_verification: bool,
    ) -> Result<(Self, JoinHandle<()>), EndpointError> {
        let account_id = certificate.id();
        let certificate_verifier = Arc::new(CertificateVerifier {
            enabled: enable_certificate_verification,
            account_id,
            peer_whitelist: Default::default(),
        });

        let platform = Arc::new(tokio::sync::Mutex::new(
            PlatformClient::<Channel>::create(platform_grpc_port).await?,
        ));

        // Start gRPC server
        daemon::StandardNode::start(certificate_verifier.clone(), account_id, node_grpc_port)?;

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
                Box::new(PeerHandler {
                    platform: platform.clone(),
                })
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
        Ok((Self { connection_manager }, task))
    }

    pub async fn connect(&self, addr: &SocketAddr) -> Result<Arc<Connection>, ConnectionError> {
        self.connection_manager.clone().connect(addr).await
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
    Grpc(#[from] tonic::transport::Error),
    Quic(#[from] quinn::EndpointError),
    Socket(#[from] std::io::Error),
    Sqlite(#[from] rusqlite::Error),
    TlsConfiguration(#[from] rustls::TLSError),
}
