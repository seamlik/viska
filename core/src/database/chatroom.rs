use super::transaction_payload::Content;
use super::TransactionPayload;
use crate::changelog::Message;
use crate::daemon::platform_client::PlatformClient;
use crate::daemon::NullableResponse;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Request;
use tonic::Status;

pub struct ChatroomService {
    pub platform: Arc<Mutex<PlatformClient<Channel>>>,
}

impl ChatroomService {
    /// Creates a [Chatroom](super::Chatroom) when receiving or sending a message belonging to a
    /// non-existing [Chatroom](super::Chatroom).
    fn create_for_message(&self, message: &Message) -> super::Chatroom {
        let mut members = message.recipients.clone();
        members.push(message.sender.clone());

        // TODO: New name by fetching Vcard
        let name = "New chatroom".to_string();

        let chatroom_inner = crate::changelog::Chatroom { members, name };

        let chatroom_id: [u8; 32] = chatroom_inner.id().into();
        let latest_message_id: [u8; 32] = message.message_id().into();

        super::Chatroom {
            inner: Some(chatroom_inner),
            time_updated: message.time,
            chatroom_id: chatroom_id.to_vec(),
            latest_message_id: latest_message_id.to_vec(),
        }
    }

    /// Updates the [Chatroom](super::Chatroom) that is supposed to hold a new [Message].
    ///
    /// If no update should be written, returns a [None].
    pub async fn update_for_message(
        &self,
        message: &Message,
    ) -> Result<Option<super::Chatroom>, Status> {
        let chatroom_id: [u8; 32] = message.chatroom_id().into();
        if let Some(mut chatroom) = self
            .platform
            .lock()
            .await
            .find_chatroom_by_id(Request::new(chatroom_id.to_vec()))
            .await
            .unwrap_response()?
        {
            if message.time > chatroom.time_updated {
                chatroom.time_updated = message.time;
                Ok(Some(chatroom))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(self.create_for_message(&message)))
        }
    }

    pub async fn update(
        &self,
        payload: crate::changelog::Chatroom,
    ) -> Result<Vec<TransactionPayload>, Status> {
        let chatroom_id = crate::database::bytes_from_hash(payload.id());
        let chatroom = if let Some(mut chatroom) = self
            .platform
            .lock()
            .await
            .find_chatroom_by_id(Request::new(chatroom_id.clone()))
            .await
            .unwrap_response()?
        {
            chatroom.inner = payload.into();
            chatroom
        } else {
            crate::database::Chatroom {
                chatroom_id,
                time_updated: crate::database::float_from_time(Utc::now()),
                inner: payload.into(),
                ..Default::default()
            }
        };
        Ok(vec![TransactionPayload {
            content: Some(Content::AddChatroom(chatroom)),
        }])
    }
}
