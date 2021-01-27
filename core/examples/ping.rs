//! Pings a Node.
//!
//! TODO: Turn this into an integration test

use std::net::SocketAddr;
use std::time::Instant;
use structopt::StructOpt;
use tokio::time::Duration;
use viska::database::ProfileConfig;
use viska::proto::request::Payload;
use viska::proto::Request;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::from_args();

    let tmp_dir = tempfile::tempdir()?;
    let account_id = viska::database::create_standard_profile(tmp_dir.path().to_path_buf())?;
    let profile_config = ProfileConfig {
        dir_data: tmp_dir.path().to_path_buf(),
    };
    let node_grpc_port = viska::util::random_port();

    let (node, _) = Node::start(&account_id, &profile_config, node_grpc_port).await?;

    let connection = node.connect(&cli.destination).await?;
    let mut counter = 0_u32;
    let request = Request {
        payload: Some(Payload::Ping(())),
    };
    loop {
        let earlier = Instant::now();
        counter += 1;
        match connection.request(&request).await {
            Ok(_) => println!(
                "Received pong ({}) after {} ms from {:?}",
                counter,
                (Instant::now() - earlier).as_millis(),
                &connection
            ),
            Err(err) => println!("  ERROR: {:?}", err),
        }
        tokio::time::delay_for(Duration::from_secs(1)).await;
    }
}

#[derive(StructOpt)]
struct Cli {
    destination: SocketAddr,
}
