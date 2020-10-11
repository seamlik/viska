tonic::include_proto!("viska.changelog");

use crate::database::chatroom::ChatroomId;
use blake3::Hash;
use blake3::Hasher;

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
        crate::database::chatroom::chatroom_id(members)
    }
}

impl Chatroom {
    pub fn id(&self) -> ChatroomId {
        crate::database::chatroom::chatroom_id(self.members.iter().cloned())
    }
}
