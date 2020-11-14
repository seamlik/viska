use crate::database::message::MessageService;
use crate::database::Database;
use crate::packet::ResponseWindow;
use crate::proto::request::Payload;
use crate::proto::Response;
use std::sync::Arc;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
#[error("Error during request handling")]
pub enum Error {
    Database(#[from] rusqlite::Error),
    GrpcOperation(#[from] Status),
    GrpcConnection(#[from] tonic::transport::Error),
}

pub trait Handler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error>;
}

pub(crate) struct PeerHandler {
    pub database: Arc<Database>,
}

impl Handler for PeerHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(Payload::Message(message)) => {
                let mut sqlite = self.database.connection.lock().unwrap();
                let transaction = sqlite.transaction()?;
                MessageService::update(&transaction, message.clone())?;
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
