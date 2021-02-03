use std::net::SocketAddrV6;
use std::str::FromStr;
use viska::proto::request::Payload;
use viska::proto::Request;

#[tokio::test]
async fn ping() -> anyhow::Result<()> {
    let (dummy, _) = viska_dev::start_dummy_node().await?;
    let dummy_port = dummy.local_port()?;

    let (prober, _) = viska_dev::start_dummy_node().await?;
    let addr = SocketAddrV6::from_str(&format!("[::1]:{}", dummy_port))?;
    let connection = prober.connect(&addr.into()).await?;
    let request = Request {
        payload: Some(Payload::Ping(())),
    };
    connection
        .request(&request)
        .await
        .expect("Should receive pong");

    Ok(())
}
