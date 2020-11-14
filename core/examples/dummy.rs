//! Runs a Node that does nothing.

use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let dummy_cert_bundle = viska::pki::new_certificate();
    let node_grpc_port = viska::util::random_port();
    let (_, future) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.key,
        node_grpc_port,
        false,
        Default::default(),
    )
    .await?;
    future.await?;
    Ok(())
}
