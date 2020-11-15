tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use changelog_payload::Content;
use rusqlite::Transaction;

pub(crate) struct ChangelogMerger;

impl ChangelogMerger {
    pub fn commit(
        transaction: &'_ Transaction,
        payloads: impl Iterator<Item = ChangelogPayload>,
    ) -> rusqlite::Result<()> {
        for payload in payloads {
            match payload.content {
                Some(Content::AddChatroom(chatroom)) => {
                    ChatroomService::update(transaction, chatroom)?;
                }
                Some(Content::AddPeer(peer)) => {
                    PeerService::save(transaction, peer)?;
                }
                Some(Content::AddMessage(message)) => {
                    MessageService::update(transaction, message)?;
                }
                None => todo!("Empty transaction payload"),
            }
        }
        Ok(())
    }
}
