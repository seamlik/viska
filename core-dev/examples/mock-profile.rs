//! Generates a mock profile and writes the database file to somewhere.

use std::path::PathBuf;
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    futures_executor::block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::from_args();
    viska::database::create_mock_profile(cli.destination).await?;
    Ok(())
}

#[derive(StructOpt)]
struct Cli {
    destination: PathBuf,
}
