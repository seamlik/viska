tonic::include_proto!("viska.daemon");

use crate::endpoint::CertificateVerifier;
use crate::transaction::Message;
use crate::transaction::Transaction;
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
    fn spawn_server(service: S) -> u16 {
        // TODO: TLS
        let port = crate::util::random_port();
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

/// [Platform] implementation without actual functionality for test purposes.
pub struct DummyPlatform;

impl<T: Platform> GrpcService<PlatformServer<T>> for DummyPlatform {}

impl DummyPlatform {
    /// Starts the gRPC server in the background and returns the port to access it.
    pub fn start() -> u16 {
        Self::spawn_server(PlatformServer::new(Self))
    }
}

#[async_trait]
impl Platform for DummyPlatform {
    async fn commit(
        &self,
        _: tonic::Request<Streaming<Transaction>>,
    ) -> Result<tonic::Response<()>, Status> {
        log::info!("Committing a transaction");
        Ok(tonic::Response::new(()))
    }

    async fn accept_message(
        &self,
        _: tonic::Request<Message>,
    ) -> Result<tonic::Response<()>, Status> {
        log::info!("Received a message");
        Ok(tonic::Response::new(()))
    }
}

pub(crate) struct StandardNode {
    verifier: Arc<CertificateVerifier>,
}

impl<T: node_server::Node> GrpcService<NodeServer<T>> for StandardNode {}

impl StandardNode {
    pub fn start(verifier: Arc<CertificateVerifier>) -> u16 {
        let instance = Self { verifier };
        Self::spawn_server(NodeServer::new(instance))
    }
}

#[async_trait]
impl node_server::Node for StandardNode {
    async fn update_peer_whitelist(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, Status> {
        unimplemented!()
    }
}