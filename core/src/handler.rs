use crate::daemon::platform_client::PlatformClient;
use crate::daemon::GrpcClient;
use crate::packet::ResponseWindow;
use crate::proto::request::Payload;
use crate::proto::Response;
use async_trait::async_trait;
use thiserror::Error;
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

pub struct PeerHandler {
    pub platform_grpc_port: u16,
}

#[async_trait]
impl Handler for PeerHandler {
    async fn handle(&self, window: &ResponseWindow) -> Result<Response, Error> {
        match &window.request.payload {
            Some(Payload::Message(message)) => {
                let mut platform = PlatformClient::create(self.platform_grpc_port).await?;
                platform
                    .accept_message(message.clone())
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
