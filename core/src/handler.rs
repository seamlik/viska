use crate::daemon::platform_client::PlatformClient;
use crate::packet::ResponseWindow;
use crate::proto::request::Payload;
use crate::proto::Response;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Status;

#[derive(Error, Debug)]
#[error("Error during request handling")]
pub enum Error {
    GrpcOperation(#[from] Status),
    GrpcConnection(#[from] tonic::transport::Error),
}

#[async_trait]
pub trait Handler {
    async fn handle(&self, window: &ResponseWindow) -> Result<Response, Error>;
}

pub(crate) struct PeerHandler {
    pub platform: Arc<Mutex<PlatformClient<Channel>>>,
}

#[async_trait]
impl Handler for PeerHandler {
    async fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(Payload::Message(message)) => {
                let message_id: [u8; 32] = message.message_id().into();
                self.platform
                    .lock()
                    .await
                    .notify_message(message_id.to_vec())
                    .await
                    .map(|_| Response::default())
                    .map_err(From::from)
            }
            _ => DefaultHandler.handle(window).await,
        }
    }
}

pub struct DeviceHandler;

#[async_trait]
impl Handler for DeviceHandler {
    async fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request {
            _ => DefaultHandler.handle(window).await,
        }
    }
}

struct DefaultHandler;

#[async_trait]
impl Handler for DefaultHandler {
    async fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(payload) => match payload {
                Payload::Ping(()) => Ok(Default::default()),
                _ => Ok(Response::forbidden()),
            },
            None => Ok(Response::bad_request("No payload".into())),
        }
    }
}
