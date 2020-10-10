use crate::changelog::Message;
use crate::daemon::PlatformConnector;
use blake3::Hash;
use blake3::Hasher;
use std::collections::BTreeSet;
use std::sync::Arc;
use tonic::Request;
use tonic::Status;

pub type ChatroomId = Hash;

pub struct ChatroomService {
    platform: Arc<PlatformConnector>,
}

impl ChatroomService {
    /// Creates a [Chatroom](super::Chatroom) when receiving or sending a message belonging to a
    /// non-existing [Chatroom](super::Chatroom).
    pub fn create_for_message(&self, message: &Message) -> super::Chatroom {
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
        let mut chatroom = self
            .platform
            .connect()
            .await
            .find_chatroom_by_id(Request::new(chatroom_id.to_vec()))
            .await?
            .into_inner();
        if message.time > chatroom.time_updated {
            chatroom.time_updated = message.time;
            Ok(Some(chatroom))
        } else {
            Ok(None)
        }
    }
}

pub fn chatroom_id(members: impl Iterator<Item = Vec<u8>>) -> ChatroomId {
    let members_sorted: BTreeSet<Vec<u8>> = members.collect();
    let mut hasher = Hasher::default();
    for id in members_sorted {
        hasher.update(id.as_slice());
    }
    hasher.finalize()
}
