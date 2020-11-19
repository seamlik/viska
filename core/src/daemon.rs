tonic::include_proto!("viska.daemon");

use crate::database::Chatroom;
use crate::database::Vcard;
use crate::endpoint::CertificateVerifier;
use async_trait::async_trait;
use futures::channel::mpsc::Receiver;
use node_client::NodeClient;
use node_server::NodeServer;
use std::error::Error;
use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tonic::transport::NamedService;
use tonic::transport::Server;
use tonic::Code;
use tonic::Status;
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

    type WatchVcardByIdStream = Receiver<Result<Vcard, Status>>;

    async fn watch_vcard_by_id(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchVcardByIdStream>, Status> {
        todo!()
    }

    type WatchChatroomMessagesStream = Receiver<Result<ChatroomMessagesSubscription, Status>>;

    async fn watch_chatroom_messages(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomMessagesStream>, Status> {
        todo!()
    }

    type WatchChatroomStream = Receiver<Result<Chatroom, Status>>;

    async fn watch_chatroom(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomStream>, Status> {
        todo!()
    }

    type WatchChatroomsStream = Receiver<Result<ChatroomsSubscription, Status>>;

    async fn watch_chatrooms(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchChatroomsStream>, Status> {
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
