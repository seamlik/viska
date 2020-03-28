use std::sync::Arc;
use viska::proto::Message;
use viska::Database;
use viska::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let dummy_cert_bundle = viska::pki::new_certificate();
    let database: Arc<dyn Database> = Arc::new(DummyDatabase::default());
    let (_, future) = Node::start(
        &dummy_cert_bundle.certificate,
        &dummy_cert_bundle.keypair,
        database,
    )?;
    future.await;
    Ok(())
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
