use super::chatroom::ChatroomService;
use super::transaction_payload::Content;
use super::TransactionPayload;
use crate::daemon::platform_client::PlatformClient;
use crate::daemon::NullableResponse;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::IntoRequest;
use tonic::Status;

pub struct MessageService {
    pub platform: Arc<Mutex<PlatformClient<Channel>>>,
    pub chatroom_service: Arc<ChatroomService>,
}

impl MessageService {
    pub async fn update(
        &self,
        payload: crate::changelog::Message,
    ) -> Result<Vec<TransactionPayload>, Status> {
        let mut transaction = Vec::<TransactionPayload>::new();

        // Update chatroom
        if let Some(chatroom) = self.chatroom_service.update_for_message(&payload).await? {
            transaction.push(TransactionPayload {
                content: Some(Content::AddChatroom(chatroom)),
            });
        }

        // Update message
        let message_id = crate::database::bytes_from_hash(payload.message_id());
        let message = if let Some(mut message) = self
            .platform
            .lock()
            .await
            .find_message_by_id(message_id.clone().into_request())
            .await
            .unwrap_response()?
        {
            message.inner = payload.into();
            message
        } else {
            super::Message {
                message_id,
                chatroom_id: crate::database::bytes_from_hash(payload.chatroom_id()),
                inner: payload.into(),
            }
        };
        transaction.push(TransactionPayload {
            content: Some(Content::AddMessage(message)),
        });

        Ok(transaction)
    }
}
