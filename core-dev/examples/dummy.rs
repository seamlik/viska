//! Runs a Node that does nothing.

fn main() -> anyhow::Result<()> {
    env_logger::init();
    futures_executor::block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let (_node, future) = viska_dev::start_dummy_node().await?;
    Ok(future.await)
}
