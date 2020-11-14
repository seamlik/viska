//! Generates a mock profile and writes the database file to somewhere.

use std::path::PathBuf;
use structopt::StructOpt;
use viska::database::Storage;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::from_args();

    let dummy_cert_bundle = viska::pki::new_certificate();
    let node_grpc_port = viska::util::random_port();
    let database_config = viska::database::Config {
        storage: Storage::OnDisk(cli.destination),
    };
    let (node, _) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.key,
        node_grpc_port,
        false,
        database_config,
    )
    .await?;

    node.populate_mock_data()?;

    Ok(())
}

#[derive(StructOpt)]
struct Cli {
    destination: PathBuf,
}
