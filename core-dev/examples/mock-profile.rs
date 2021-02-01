//! Generates a mock profile and writes the database file to somewhere.

use std::path::PathBuf;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::from_args();
    viska::database::create_mock_profile(cli.destination)?;
    Ok(())
}

#[derive(StructOpt)]
struct Cli {
    destination: PathBuf,
}
