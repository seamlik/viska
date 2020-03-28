use futures::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
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
    let (node, task) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.keypair,
        database,
    )?;

    let connection = node.connect(&cli.destination).await?;
    tokio::time::interval(Duration::from_secs(1))
        .for_each(|_| async {
            println!("Pinging remote node at {}", cli.destination);
            match connection.request(&Request::Ping).await {
                Ok(_) => println!("  SUCCESS"),
                Err(err) => println!("  ERROR: {}", err),
            }
        })
        .await;

    task.await;
    Ok(())
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
