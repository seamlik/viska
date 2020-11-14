use super::chatroom::ChatroomService;
use super::BytesArray;
use prost::Message as _;
use rusqlite::Transaction;

pub(crate) struct MessageService;

impl MessageService {
    fn save(transaction: &'_ Transaction, payload: super::Message) -> rusqlite::Result<()> {
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
            ) VALUES (?);
        "#;
        let params = rusqlite::params![
            payload.message_id,
            payload.chatroom_id,
            attachment,
            attachment_mime,
            inner.content,
            recipients,
            inner.sender,
            inner.time,
        ];
        transaction.execute(sql, params)?;
        Ok(())
    }

    pub fn update(
        transaction: &'_ Transaction,
        payload: crate::changelog::Message,
    ) -> rusqlite::Result<()> {
        // Update chatroom
        ChatroomService::update_for_message(transaction, &payload)?;

        // Update message
        let message_id = super::bytes_from_hash(payload.message_id());
        let chatroom_id = super::bytes_from_hash(payload.chatroom_id());
        let message = super::Message {
            message_id,
            chatroom_id,
            inner: payload.into(),
        };
        MessageService::save(transaction, message)?;
        Ok(())
    }
}
