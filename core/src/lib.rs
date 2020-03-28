#[path = "../../target/riko/bridge.rs"]
mod bridge;

pub mod android;
mod endpoint;
mod handler;
mod packet;
pub mod pki;
pub mod proto;
pub(crate) mod util;

use endpoint::ConnectionInfo;
use endpoint::ConnectionManager;
pub use endpoint::Error as EndpointError;
use endpoint::LocalEndpoint;
use futures::prelude::*;
use handler::DeviceHandler;
use handler::Handler;
use handler::PeerHandler;
use packet::ResponseWindow;
use pki::Certificate;
use proto::Message;
use std::sync::Arc;

/// The protagonist.
pub struct Node {
    connections: Arc<ConnectionManager>,
    endpoint: LocalEndpoint,
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
    ) -> Result<(Self, impl Future<Output = ()>), EndpointError> {
        let connections = Arc::<ConnectionManager>::default();
        let config = endpoint::Config {
            certificate,
            key,
            database: &database,
        };
        let (endpoint, new_connections) = LocalEndpoint::start(&config)?;
        let (window_sender, window_receiver) =
            futures::channel::mpsc::unbounded::<ResponseWindow>();

        let connections_cloned = connections.clone();
        let connection_task = new_connections.for_each(move |new_quic_connection| {
            let connections_cloned = connections_cloned.clone();
            connections_cloned
                .clone()
                .add(new_quic_connection, window_sender.clone());
            async {}
        });

        let account_id = certificate.id();
        let request_task = window_receiver.for_each(move |window| {
            let response = if window.account_id() == Some(account_id) {
                DeviceHandler.handle(&window)
            } else {
                PeerHandler {
                    database: database.clone(),
                }
                .handle(&window)
            };
            async {
                match response {
                    Ok(r) => window
                        .send_response(r)
                        .await
                        .unwrap_or_else(|err| log::error!("Error sending a response: {}", err)),
                    Err(err) => window.disconnect(err),
                }
            }
        });

        let all_tasks = async {
            connection_task.await;
            tokio::spawn(request_task)
                .await
                .unwrap_or_else(|err| log::error!("Error when processing requests: {}", err))
        };
        Ok((
            Self {
                connections,
                endpoint,
            },
            all_tasks,
        ))
    }
}

/// Access point for Viska's database.
pub trait Database: Send + Sync {
    /// Checks if an account of the provided `account_id` is in the roster.
    fn is_peer(&self, account_id: &[u8]) -> bool;

    /// Accepts an incoming [Message].
    fn accept_message(&self, message: &Message, sender: &[u8]);
}
