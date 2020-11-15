tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use changelog_payload::Content;
use rusqlite::Connection;

pub(crate) struct ChangelogMerger;

impl ChangelogMerger {
    pub fn commit(
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
                    PeerService::save(connection, peer)?;
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
