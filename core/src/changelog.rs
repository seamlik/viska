tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use changelog_payload::Content;
use diesel::prelude::*;
use std::sync::Arc;

pub(crate) struct ChangelogMerger {
    pub chatroom_service: Arc<ChatroomService>,
    pub message_service: Arc<MessageService>,
    pub peer_service: Arc<PeerService>,
}

impl ChangelogMerger {
    pub fn commit(
        &self,
        connection: &'_ SqliteConnection,
        payloads: impl Iterator<Item = ChangelogPayload>,
    ) -> QueryResult<()> {
        for payload in payloads {
            log::debug!("Committing {:?}", &payload.content);
            match payload.content {
                Some(Content::AddChatroom(chatroom)) => {
                    self.chatroom_service.save(connection, &chatroom)?;
                }
                Some(Content::AddPeer(peer)) => {
                    self.peer_service.save(connection, peer)?;
                }
                Some(Content::AddMessage(message)) => {
                    self.message_service.update(connection, &message)?;
                }
                None => todo!("Empty transaction payload"),
            }
        }
        Ok(())
    }
}
