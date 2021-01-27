//! Runs a Node that does nothing.

use viska::database::ProfileConfig;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let tmp_dir = tempfile::tempdir()?;
    let account_id = viska::database::create_standard_profile(tmp_dir.path().to_path_buf())?;
    let profile_config = ProfileConfig {
        dir_data: tmp_dir.path().to_path_buf(),
    };
    let node_grpc_port = viska::util::random_port();

    let (_node, future) = Node::start(&account_id, &profile_config, node_grpc_port).await?;
    future.await?;
    Ok(())
}
