use super::chatroom::ChatroomService;
use super::BytesArray;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use prost::Message as _;
use rusqlite::Connection;

pub(crate) struct MessageService;

impl MessageService {
    fn save(connection: &'_ Connection, payload: super::Message) -> rusqlite::Result<()> {
        let inner = payload.inner.unwrap();
        let (attachment, attachment_mime) = inner
            .attachment
            .map(|blob| (blob.content, blob.mime))
            .unwrap_or_default();

        let mut recipients = Vec::<u8>::default();
        BytesArray {
            array: inner.recipients,
        }
        .encode(&mut recipients)
        .unwrap();

        let sql = r#"
            REPLACE INTO message (
                message_id,
                chatroom_id,
                attachment,
                attachment_mime,
                content,
                recipients,
                sender,
                time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
        "#;
        let mut stmt = connection.prepare_cached(sql)?;
        stmt.execute(rusqlite::params![
            payload.message_id,
            payload.chatroom_id,
            attachment,
            attachment_mime,
            inner.content,
            recipients,
            inner.sender,
            inner.time,
        ])?;
        Ok(())
    }

    pub fn update(
        connection: &'_ Connection,
        payload: crate::changelog::Message,
    ) -> rusqlite::Result<()> {
        // Update chatroom
        ChatroomService::update_for_message(connection, &payload)?;

        // Update message
        let message_id = super::bytes_from_hash(payload.canonical_id());
        let chatroom_id = super::bytes_from_hash(payload.chatroom_id());
        let message = super::Message {
            message_id,
            chatroom_id,
            inner: payload.into(),
        };
        MessageService::save(connection, message)?;
        Ok(())
    }
}

impl crate::changelog::Message {
    pub fn chatroom_id(&self) -> Hash {
        let recipients = self.recipients.iter();
        let members = recipients.chain(std::iter::once(&self.sender));
        super::chatroom::chatroom_id(members)
    }
}

impl CanonicalId for crate::changelog::Message {
    fn canonical_id(&self) -> Hash {
        let mut hasher = Hasher::default();

        hasher.update(b"Viska message");

        hasher.update(&self.sender.len().to_be_bytes());
        hasher.update(&self.sender);

        let length = self.recipients.iter().fold(0, |sum, x| sum + x.len());
        hasher.update(&length.to_be_bytes());
        for account in self.recipients.iter() {
            hasher.update(&account);
        }

        hasher.update(self.time.to_be_bytes().as_ref());

        hasher.update(&self.content.len().to_be_bytes());
        hasher.update(self.content.as_bytes());

        if let Some(attachment) = &self.attachment {
            hasher.update(&blake3::OUT_LEN.to_be_bytes());
            hasher.update(attachment.canonical_id().as_bytes());
        }

        hasher.finalize()
    }
}
