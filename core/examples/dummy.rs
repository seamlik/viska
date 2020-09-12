//! Runs a Node that does nothing.

use viska::daemon::DummyPlatform;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let dummy_cert_bundle = viska::pki::new_certificate();
    let platform_grpc_port = DummyPlatform::start();
    let (_, future) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.key,
        platform_grpc_port,
        false,
    )?;
    future.await?;
    Ok(())
}
