use crate::endpoint::ConnectionInfo;
use crate::packet::ResponseWindow;
use crate::proto::Request;
use crate::proto::Response;
use crate::Database;
use std::sync::Arc;
use thiserror::Error;

/// Error when handling a [Request].
///
/// If such error occurred, no response should be sent to the peer; the connection must be closed
/// immediately.
#[derive(Error, Debug)]
#[error("Unrecoverable error occured during request handling")]
pub enum Error {
    PeerIdAbsent,
}

pub trait Handler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error>;
}

pub struct PeerHandler {
    pub database: Arc<dyn Database>,
}

impl Handler for PeerHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request {
            Request::Ping => Ok(Default::default()),
            Request::Message(message) => match window.account_id() {
                Some(id) => {
                    self.database.accept_message(&message, id.as_bytes());
                    Ok(Default::default())
                }
                None => Err(Error::PeerIdAbsent),
            },
        }
    }
}

pub struct DeviceHandler;

impl Handler for DeviceHandler {
    fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request {
            Request::Ping => Ok(Default::default()),
            _ => Ok(Response::forbidden()),
        }
    }
}
