//! Shared code for development purposes.

use rand::prelude::*;
use std::future::Future;
use viska::database::ProfileConfig;
use viska::Node;

/// Runs a [Node] that does nothing.
pub async fn start_dummy_node() -> Result<(Node, impl Future<Output = ()>), anyhow::Error> {
    let tmp_dir = tempfile::tempdir()?;
    let account_id = viska::database::create_standard_profile(tmp_dir.path().to_path_buf()).await?;
    let profile_config = ProfileConfig {
        dir_data: tmp_dir.path().to_path_buf(),
    };
    let node_grpc_port = random_port();

    Node::start(&account_id, &profile_config, node_grpc_port)
        .await
        .map_err(Into::into)
}

/// Generates a random port within the private range untouched by IANA.
fn random_port() -> u16 {
    thread_rng().gen_range(49152..u16::MAX)
}
