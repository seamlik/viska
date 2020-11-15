tonic::include_proto!("viska.daemon");

use crate::database::Chatroom;
use crate::database::Message;
use crate::database::Peer;
use crate::database::TransactionPayload;
use crate::endpoint::CertificateVerifier;
use async_trait::async_trait;
use node_client::NodeClient;
use node_server::NodeServer;
use platform_client::PlatformClient;
use platform_server::Platform;
use platform_server::PlatformServer;
use std::error::Error;
use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tonic::transport::NamedService;
use tonic::transport::Server;
use tonic::Code;
use tonic::Status;
use tonic::Streaming;
use tower::Service;

trait GrpcService<S>
where
    S: Service<http::Request<Body>, Response = http::Response<BoxBody>>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn Error + Send + Sync>> + Send,
{
    fn spawn_server(service: S, port: u16) {
        // TODO: TLS
        log::info!("{} serving at port {}", std::any::type_name::<S>(), port);
        let task = async move {
            Server::builder()
                .add_service(service)
                .serve(format!("[::1]:{}", port).parse().unwrap())
                .await
                .expect("Failed to spawn gRPC server")
        };
        tokio::spawn(task);
    }
}

#[async_trait]
pub(crate) trait GrpcClient: Sized {
    async fn create(port: u16) -> Result<Self, tonic::transport::Error>;
}

#[async_trait]
impl GrpcClient for PlatformClient<Channel> {
    async fn create(port: u16) -> Result<Self, tonic::transport::Error> {
        Self::connect(format!("http://[::1]:{}", port)).await
    }
}

#[async_trait]
impl GrpcClient for NodeClient<Channel> {
    async fn create(port: u16) -> Result<Self, tonic::transport::Error> {
        Self::connect(format!("http://[::1]:{}", port)).await
    }
}

/// Nullable gRPC response.
///
/// If error status is [NotFound](Code::NotFound), the resposne can be seen as [None].
pub trait NullableResponse<T: prost::Message> {
    fn unwrap_response(self) -> Result<Option<T>, Status>;
}

impl<T: prost::Message> NullableResponse<T> for Result<tonic::Response<T>, Status> {
    fn unwrap_response(self) -> Result<Option<T>, Status> {
        match self {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) => {
                if let Code::NotFound = status.code() {
                    Ok(None)
                } else {
                    Err(status)
                }
            }
        }
    }
}

/// [Platform] implementation without actual functionality for test purposes.
pub struct DummyPlatform;

impl<T: Platform> GrpcService<PlatformServer<T>> for DummyPlatform {}

impl DummyPlatform {
    /// Starts the gRPC server in the background and returns the port to access it.
    pub fn start() -> u16 {
        let port = crate::util::random_port();
        Self::spawn_server(PlatformServer::new(Self), port);
        port
    }
}

#[async_trait]
impl Platform for DummyPlatform {
    async fn commit_transaction(
        &self,
        _: tonic::Request<Streaming<TransactionPayload>>,
    ) -> Result<tonic::Response<()>, Status> {
        log::info!("Committing a transaction");
        Ok(tonic::Response::new(()))
    }

    async fn notify_message(
        &self,
        message_id: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<()>, Status> {
        log::info!(
            "Received a message {}",
            hex::encode_upper(message_id.get_ref())
        );
        Ok(tonic::Response::new(()))
    }

    async fn find_chatroom_by_id(
        &self,
        chatroom_id: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Chatroom>, Status> {
        log::info!(
            "Finding chatroom {}",
            hex::encode_upper(chatroom_id.get_ref())
        );
        Ok(tonic::Response::new(Default::default()))
    }

    async fn find_peer_by_id(
        &self,
        peer_id: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Peer>, Status> {
        log::info!("Finding peer {}", hex::encode_upper(peer_id.get_ref()));
        Ok(tonic::Response::new(Default::default()))
    }

    async fn find_message_by_id(
        &self,
        message_id: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Message>, Status> {
        log::info!(
            "Finding message {}",
            hex::encode_upper(message_id.get_ref())
        );
        Ok(tonic::Response::new(Default::default()))
    }
}

pub(crate) struct StandardNode {
    verifier: Arc<CertificateVerifier>,
}

impl<T: node_server::Node> GrpcService<NodeServer<T>> for StandardNode {}

impl StandardNode {
    pub fn start(verifier: Arc<CertificateVerifier>, node_grpc_port: u16) -> rusqlite::Result<()> {
        let instance = Self { verifier };
        Self::spawn_server(NodeServer::new(instance), node_grpc_port);
        Ok(())
    }
}

#[async_trait]
impl node_server::Node for StandardNode {
    async fn update_peer_whitelist(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, Status> {
        todo!()
    }
}

trait IntoTonicStatus {
    fn into_tonic_status(self) -> Status;
}

impl IntoTonicStatus for rusqlite::Error {
    fn into_tonic_status(self) -> Status {
        match self {
            rusqlite::Error::QueryReturnedNoRows => Status::not_found(""),
            _ => Status::internal(self.to_string()),
        }
    }
}
