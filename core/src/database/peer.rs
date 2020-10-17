use super::transaction_payload::Content;
use super::TransactionPayload;
use crate::daemon::platform_client::PlatformClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Status;

pub struct PeerService {
    pub platform: Arc<Mutex<PlatformClient<Channel>>>,
}

impl PeerService {
    pub async fn update(
        &self,
        payload: crate::changelog::Peer,
    ) -> Result<Vec<TransactionPayload>, Status> {
        Ok(vec![TransactionPayload {
            content: Some(Content::AddPeer(super::Peer {
                inner: Some(payload),
            })),
        }])
    }
}
