tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use changelog_payload::Content;
use rusqlite::Connection;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct ChangelogMerger {
    pub peer_service: Arc<PeerService>,
}

impl ChangelogMerger {
    pub fn commit(
        &self,
        connection: &'_ Connection,
        payloads: impl Iterator<Item = ChangelogPayload>,
    ) -> rusqlite::Result<()> {
        for payload in payloads {
            log::debug!("Committing {:?}", &payload.content);
            match payload.content {
                Some(Content::AddChatroom(chatroom)) => {
                    ChatroomService::update(connection, chatroom)?;
                }
                Some(Content::AddPeer(peer)) => {
                    self.peer_service.save(connection, peer)?;
                }
                Some(Content::AddMessage(message)) => {
                    MessageService::update(connection, message)?;
                }
                None => todo!("Empty transaction payload"),
            }
        }
        Ok(())
    }
}
