//! Runs a Node that does nothing.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let (_node, future) = viska_dev::start_dummy_node().await?;
    future.await?;
    Ok(())
}
