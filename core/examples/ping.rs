use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;
use tokio::time::Duration;
use viska::proto::Message;
use viska::proto::Request;
use viska::Database;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::from_args();

    let dummy_cert_bundle = viska::pki::new_certificate();
    let database: Arc<dyn Database> = Arc::new(DummyDatabase::default());
    let (node, _) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.keypair,
        database,
    )?;

    let connection = node.connect(&cli.destination).await?;
    let mut counter = 0_u32;
    loop {
        let earlier = Instant::now();
        counter += 1;
        match connection.request(&Request::Ping).await {
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

#[derive(Default)]
struct DummyDatabase;

impl viska::Database for DummyDatabase {
    fn is_peer(&self, _: &[u8]) -> bool {
        true
    }
    fn accept_message(&self, _: &Message, _: &[u8]) {
        println!("Received a message");
    }
}
