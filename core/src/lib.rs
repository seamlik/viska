#![feature(drain_filter)]
#![feature(once_cell)]
#![feature(proc_macro_hygiene)]

// (https://github.com/diesel-rs/diesel/issues/1894)
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

#[path = "../../target/riko/viska.rs"]
#[riko::ignore]
pub mod bridge;

mod changelog;
mod daemon;
pub mod database;
mod endpoint;
mod handler;
mod mock_profile;
mod packet;
pub mod pki;
pub mod proto;
pub mod util;

use self::daemon::node_client::NodeClient;
use self::daemon::Event;
use self::database::ProfileConfig;
use crate::database::peer::PeerService;
use crate::database::Database;
use crate::endpoint::CertificateVerifier;
use blake3::Hash;
use database::DatabaseInitializationError;
use database::Storage;
use endpoint::ConnectionInfo;
use endpoint::ConnectionManager;
use futures_util::FutureExt;
use http::StatusCode;
use packet::ResponseWindow;
use pki::CanonicalId;
use prost::DecodeError;
use prost::Message as _;
use proto::Request;
use proto::Response;
use quinn::ReadToEndError;
use serde_bytes::ByteBuf;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::future::Future;
use std::lazy::SyncLazy;
use std::net::SocketAddr;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use tokio::sync::broadcast::Sender;
use tonic::transport::Channel;
use uuid::Uuid;

static CURRENT_NODE_HANDLE: AtomicI32 = AtomicI32::new(0);
static NODES: SyncLazy<Mutex<HashMap<i32, Node>>> = SyncLazy::new(Default::default);

/// Starts a [Node].
///
/// # Returns
///
/// The handle for use with [stop].
#[riko::fun]
pub async fn start(
    account_id: ByteBuf,
    profile_config: ProfileConfig,
    node_grpc_port: u16,
) -> Result<i32, NodeStartError> {
    let handle = CURRENT_NODE_HANDLE.fetch_add(1, Ordering::SeqCst);
    let (node, task) = Node::new(&account_id, &profile_config, node_grpc_port).await?;
    self::util::spawn(task);
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
    connection_manager: ConnectionManager,
    _node_grpc_shutdown_token: Box<dyn Any + Send>,
    grpc_port: u16,
    event_sink_daemon: Sender<Arc<Event>>,
}

impl Node {
    /// Constructor.
    ///
    /// Port to serve ths gRPC service must be manually chosen because Tonic currently does not
    /// support getting the port number from a started service.
    ///
    /// The returned [Future] is for driving all operations. Nothing runs until it is run. It will
    /// run to completion once [Node] is dropped.
    pub async fn new(
        account_id: &[u8],
        profile_config: &ProfileConfig,
        grpc_port: u16,
    ) -> Result<(Self, impl Future<Output = ()>), NodeStartError> {
        let database = Arc::new(Database::create(&Storage::OnDisk(
            profile_config.path_database(account_id).await?,
        ))?);

        let (event_sink_database, _) = tokio::sync::broadcast::channel(8);

        let certificate =
            async_fs::read(profile_config.path_certificate(account_id).await?).await?;
        let key = async_fs::read(profile_config.path_key(account_id).await?).await?;

        let account_id_calculated = certificate.canonical_id();
        if account_id_calculated.as_bytes() != account_id {
            return Err(NodeStartError::IncorrectAccountId);
        }

        let certificate_verifier: Arc<_> = CertificateVerifier::new(account_id_calculated).into();
        certificate_verifier.set_rules(
            std::iter::empty(),
            PeerService::blacklist(&database.connection.lock().unwrap())?,
        );

        // Start gRPC server
        let (event_sink_daemon, _) = tokio::sync::broadcast::channel(8);
        let (grpc_task, node_grpc_shutdown_token) = daemon::StandardNode::create(
            grpc_port,
            event_sink_database.clone(),
            event_sink_daemon.clone(),
            database.clone(),
        );

        // QUIC endpoint and connection manager
        let endpoint_config = self::endpoint::Config {
            certificate: &certificate,
            key: &key,
        };
        let (window_sender, window_receiver) = futures_channel::mpsc::unbounded::<ResponseWindow>();
        let request_handler_task = ResponseWindow::consumer_task(
            account_id_calculated,
            window_receiver,
            database.clone(),
            event_sink_database,
            event_sink_daemon.clone(),
        );
        let (connection_manager, connection_manager_task) =
            ConnectionManager::new(&endpoint_config, certificate_verifier, window_sender)?;

        let task = async move {
            futures_util::join!(
                grpc_task.boxed(),
                request_handler_task.boxed(),
                connection_manager_task.boxed(),
            );
        };

        log::info!(
            "Started Viska node with account {}",
            account_id_calculated.to_hex()
        );
        Ok((
            Self {
                connection_manager,
                _node_grpc_shutdown_token: Box::new(node_grpc_shutdown_token),
                grpc_port,
                event_sink_daemon,
            },
            task,
        ))
    }

    /// Connects to a remote [Node].
    pub async fn connect(&self, addr: &SocketAddr) -> Result<Arc<Connection>, ConnectionError> {
        self.connection_manager.connect(addr).await
    }

    /// Gets the local port.
    pub fn local_port(&self) -> std::io::Result<u16> {
        self.connection_manager.local_port()
    }

    #[cfg(test)]
    async fn grpc_client(&self) -> Result<NodeClient<Channel>, tonic::transport::Error> {
        NodeClient::<Channel>::connect(format!("http://[::1]:{}", self.grpc_port)).await
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

    /// Closes the connection with a reason code.
    pub fn close(&self, code: StatusCode) {
        log::info!("Closing connection to {}", self.remote_address());
        self.quic.close(code.as_u16().into(), &[]);
    }
}

impl ConnectionInfo for Connection {
    fn remote_address(&self) -> SocketAddr {
        self.quic.remote_address()
    }

    fn account_id(&self) -> Option<Hash> {
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

#[derive(Error, Debug)]
#[error("Failed to start a Viska node")]
pub enum NodeStartError {
    DataFile(#[from] std::io::Error),
    DatabaseInitialization(#[from] DatabaseInitializationError),
    DatabaseQuery(#[from] diesel::result::Error),
    Endpoint(#[from] self::endpoint::Error),

    #[error("Account ID does not match with the certificate")]
    IncorrectAccountId,
}
