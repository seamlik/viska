tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use blake3::Hash;
use blake3::Hasher;
use changelog_payload::Content;
use rusqlite::Transaction;
use std::collections::BTreeSet;

type MessageId = Hash;

impl Message {
    /// Calculates the ID of a [Message].
    pub fn message_id(&self) -> MessageId {
        let mut hasher = Hasher::default();

        hasher.update(&self.sender);
        for account in self.recipients.iter() {
            hasher.update(&account);
        }

        hasher.update(self.time.to_be_bytes().as_ref());
        hasher.update(self.content.as_bytes());

        if let Some(attachment) = &self.attachment {
            hasher.update(attachment.mime.as_bytes());
            hasher.update(&attachment.content);
        }

        hasher.finalize()
    }

    pub fn chatroom_id(&self) -> ChatroomId {
        let recipients = self.recipients.iter().cloned();
        let members = recipients.chain(std::iter::once(self.sender.clone()));
        chatroom_id(members)
    }
}

pub type ChatroomId = Hash;

impl Chatroom {
    pub fn id(&self) -> ChatroomId {
        chatroom_id(self.members.iter().cloned())
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

pub(crate) struct ChangelogMerger;

impl ChangelogMerger {
    pub fn commit<'t>(
        transaction: &'t Transaction,
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
