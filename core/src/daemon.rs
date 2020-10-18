tonic::include_proto!("viska.daemon");

use crate::changelog::ChangelogMerger;
use crate::database::Chatroom;
use crate::database::Message;
use crate::database::Peer;
use crate::database::TransactionPayload;
use crate::endpoint::CertificateVerifier;
use crate::pki::CertificateId;
use async_trait::async_trait;
use node_client::NodeClient;
use node_server::NodeServer;
use platform_client::PlatformClient;
use platform_server::Platform;
use platform_server::PlatformServer;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    fn spawn_server(service: S, port: u16) -> u16 {
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
        port
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
        Self::spawn_server(PlatformServer::new(Self), crate::util::random_port())
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
    platform: Arc<Mutex<PlatformClient<Channel>>>,
    account_id: CertificateId,
    changelog_merger: Arc<ChangelogMerger>,
}

impl<T: node_server::Node> GrpcService<NodeServer<T>> for StandardNode {}

impl StandardNode {
    pub fn start(
        verifier: Arc<CertificateVerifier>,
        platform: Arc<Mutex<PlatformClient<Channel>>>,
        account_id: CertificateId,
        node_grpc_port: u16,
    ) -> u16 {
        let instance = Self {
            changelog_merger: Arc::new(ChangelogMerger::new(platform.clone())),
            verifier,
            platform,
            account_id,
        };
        Self::spawn_server(NodeServer::new(instance), node_grpc_port)
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

    #[cfg(not(debug_assertions))]
    async fn populate_mock_data(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, Status> {
        Err(Status::unimplemented("Only in debug mode"))
    }

    #[cfg(debug_assertions)]
    async fn populate_mock_data(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, Status> {
        let account_id_bytes = crate::database::bytes_from_hash(self.account_id.clone());
        let (transaction, changelog) = crate::mock_profile::populate_data(&account_id_bytes);
        log::info!(
            "Generated a transaction of {} entries and a changelog of {} entries",
            transaction.len(),
            changelog.len()
        );
        let mut platform = self.platform.lock().await;

        log::info!("Committing the mock Vcards as a transaction");
        platform
            .commit_transaction(futures::stream::iter(transaction))
            .await?;

        log::info!("Merging changelog generated from `mock_profile`");
        let transaction_after_changelog =
            self.changelog_merger.commit(changelog.into_iter()).await?;
        log::info!(
            "Generated a transaction of {} entries after a changelog merge",
            transaction_after_changelog.len()
        );
        platform
            .commit_transaction(futures::stream::iter(transaction_after_changelog))
            .await?;

        Ok(tonic::Response::new(()))
    }
}
