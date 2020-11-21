//! Pings a Node.
//!
//! TODO: Turn this into an integration test

use std::net::SocketAddr;
use std::time::Instant;
use structopt::StructOpt;
use tokio::time::Duration;
use viska::proto::request::Payload;
use viska::proto::Request;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::from_args();

    let dummy_cert_bundle = viska::pki::new_certificate();
    let node_grpc_port = viska::util::random_port();
    let (node, _) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.key,
        node_grpc_port,
        Default::default(),
    )
    .await?;

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
