use crate::daemon::event::Content;
use crate::daemon::Event as DaemonEvent;
use crate::database::message::MessageService;
use crate::database::Database;
use crate::database::Event as DatabaseEvent;
use crate::packet::ResponseWindow;
use crate::pki::CanonicalId;
use crate::proto::request::Payload;
use crate::proto::Response;
use diesel::prelude::*;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast::Sender;
use tonic::Status;

#[derive(Error, Debug)]
#[error("Error during request handling")]
pub enum Error {
    Database(#[from] diesel::result::Error),
    GrpcOperation(#[from] Status),
    GrpcConnection(#[from] tonic::transport::Error),
}

pub trait Handler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error>;
}

pub(crate) struct PeerHandler {
    pub database: Arc<Database>,
    pub event_sink_database: Sender<Arc<DatabaseEvent>>,
    pub event_sink_daemon: Sender<Arc<DaemonEvent>>,
}

impl Handler for PeerHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(Payload::Message(message)) => {
                let connection = self.database.connection.lock().unwrap();

                let datbase_event =
                    connection.transaction::<_, diesel::result::Error, _>(|| {
                        MessageService::update(&connection, &message)
                    })?;
                let _ = self.event_sink_database.send(datbase_event.into());

                let daemon_event = DaemonEvent {
                    content: Content::Message(message.canonical_id().as_bytes().to_vec()).into(),
                };
                let _ = self.event_sink_daemon.send(daemon_event.into());

                Ok(Default::default())
            }
            _ => DefaultHandler.handle(window),
        }
    }
}

pub struct DeviceHandler;

impl Handler for DeviceHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request {
            _ => DefaultHandler.handle(window),
        }
    }
}

struct DefaultHandler;

impl Handler for DefaultHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(payload) => match payload {
                Payload::Ping(()) => Ok(Default::default()),
                _ => Ok(Response::forbidden()),
            },
            None => Ok(Response::bad_request("No payload".into())),
        }
    }
}
