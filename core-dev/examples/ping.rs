//! Pings a Node.

use std::net::SocketAddr;
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;
use viska::proto::request::Payload;
use viska::proto::Request;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    futures_executor::block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::from_args();

    let (node, _) = viska_dev::start_dummy_node().await?;

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
        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}

#[derive(StructOpt)]
struct Cli {
    destination: SocketAddr,
}
