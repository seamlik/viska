#[path = "../../target/riko/bridge.rs"]
mod bridge;

pub mod android;
mod endpoint;
mod handler;
mod packet;
pub mod pki;
pub mod proto;

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
use proto::Message;
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
}

impl Node {
    /// Constructor.
    ///
    /// The returned [Future] is a handle for awaiting all operations. Everything is running once
    /// this method finishes.
    pub fn start(
        certificate: &[u8],
        key: &[u8],
        database: Arc<dyn Database>,
    ) -> Result<(Self, JoinHandle<()>), EndpointError> {
        let config = endpoint::Config {
            certificate,
            key,
            database: &database,
        };
        let (window_sender, window_receiver) =
            futures::channel::mpsc::unbounded::<ResponseWindow>();

        let database_cloned = database.clone();
        let account_id = certificate.id();
        tokio::spawn(window_receiver.for_each(move |window| {
            let response = if window.account_id() == Some(account_id) {
                DeviceHandler.handle(&window)
            } else {
                PeerHandler {
                    database: database_cloned.clone(),
                }
                .handle(&window)
            };
            async {
                match response {
                    Ok(r) => window
                        .send_response(r)
                        .await
                        .unwrap_or_else(|err| log::error!("Error sending a response: {}", err)),
                    Err(err) => window.disconnect(err).await,
                }
            }
        }));

        let (endpoint, incomings) = LocalEndpoint::start(&config)?;
        let connection_manager = Arc::new(ConnectionManager::new(endpoint, window_sender));

        let connection_manager_cloned = connection_manager.clone();
        let task = tokio::spawn(incomings.for_each(move |connecting| {
            let connection_manager_cloned = connection_manager_cloned.clone();
            tokio::spawn(async {
                match connecting.await {
                    Ok(new_connection) => {
                        connection_manager_cloned.add(new_connection).await;
                    }
                    Err(err) => log::error!("Failed to accept an incoming connection: {}", err),
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
    pub id: Uuid,
    quic: quinn::Connection,
    manager: Arc<ConnectionManager>,
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
        let raw_request = serde_cbor::to_vec(request)
            .unwrap_or_else(|err| panic!("Failed to encode a request: {}", err));

        sender.write_all(&raw_request).await?;
        sender.finish().await?;
        drop(sender);

        match receiver.read_to_end(packet::MAX_PACKET_SIZE_BYTES).await {
            Ok(raw_response) => {
                let response = serde_cbor::from_slice(&raw_response)?;
                log::debug!("Received response: {:?}", &response);
                Ok(response)
            }
            Err(err) => match err {
                ReadToEndError::TooLong => {
                    self.manager
                        .close(&self.id, StatusCode::PAYLOAD_TOO_LARGE)
                        .await;
                    Err(RequestError::ResponseTooLong)
                }
                ReadToEndError::Read(inner) => Err(inner.into()),
            },
        }
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
    BadResponse(#[from] serde_cbor::Error),
    Connection(#[from] quinn::ConnectionError),
    Read(#[from] quinn::ReadError),
    ResponseTooLong,
    Write(#[from] quinn::WriteError),
}

/// Access point for Viska's database.
pub trait Database: Send + Sync {
    /// Checks if an account of the provided `account_id` is in the roster.
    fn is_peer(&self, account_id: &[u8]) -> bool;

    /// Accepts an incoming [Message].
    fn accept_message(&self, message: &Message, sender: &[u8]);
}

/// Error when connecting with a remote [Node].
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
