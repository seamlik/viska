use crate::database::message::MessageService;
use crate::database::Database;
use crate::packet::ResponseWindow;
use crate::proto::request::Payload;
use crate::proto::Response;
use diesel::prelude::*;
use std::sync::Arc;
use thiserror::Error;
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
}

impl Handler for PeerHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(Payload::Message(message)) => {
                // TODO: Send events
                let connection = self.database.connection.lock().unwrap();
                connection.transaction::<_, diesel::result::Error, _>(|| {
                    MessageService::update(&connection, &message)
                })?;
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
