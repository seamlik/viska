use super::BytesArray;
use chrono::Utc;
use prost::Message as _;
use rusqlite::types::FromSql;
use rusqlite::Connection;
use rusqlite::Transaction;

pub(crate) struct ChatroomService;

impl ChatroomService {
    /// Creates a [Chatroom](super::Chatroom) when receiving or sending a message belonging to a
    /// non-existing [Chatroom](super::Chatroom).
    fn create_for_message(message: &crate::changelog::Message) -> super::Chatroom {
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
    pub fn update_for_message(
        transaction: &Transaction,
        message: &crate::changelog::Message,
    ) -> rusqlite::Result<()> {
        let chatroom_id = message.chatroom_id();
        if let Some(time_updated) = ChatroomService::select_column_by_id(
            transaction,
            chatroom_id.as_bytes().as_ref(),
            "time_updated",
        )? {
            if message.time > time_updated {
                ChatroomService::update_time_updated_by_id(
                    transaction,
                    chatroom_id.as_bytes().as_ref(),
                    time_updated,
                )?;
            }
        } else {
            ChatroomService::save(transaction, ChatroomService::create_for_message(message))?;
        }
        Ok(())
    }

    fn update_time_updated_by_id(
        transaction: &Transaction,
        chatroom_id: &[u8],
        time_updated: f64,
    ) -> rusqlite::Result<()> {
        let sql = "UPDATE chatroom SET time_updated = ? WHERE chatroom_id = ?;";
        transaction.execute(sql, rusqlite::params![time_updated, chatroom_id])?;
        Ok(())
    }

    fn save<'t>(transaction: &'t Transaction, payload: super::Chatroom) -> rusqlite::Result<()> {
        let inner = payload.inner.unwrap();

        let mut members = Vec::<u8>::default();
        BytesArray {
            array: inner.members,
        }
        .encode(&mut members)
        .unwrap();

        let sql = r#"
            REPLACE INTO chatroom (
                chatroom_id,
                latest_message_id,
                time_updated,
                name,
                members
            ) VALUES (?);
        "#;
        let params = rusqlite::params![
            payload.chatroom_id,
            payload.time_updated,
            inner.name,
            members,
        ];
        transaction.execute(sql, params)?;
        Ok(())
    }

    pub fn update<'t>(
        transaction: &'t Transaction,
        payload: crate::changelog::Chatroom,
    ) -> rusqlite::Result<()> {
        let chatroom_id = super::bytes_from_hash(payload.id());
        let mut row: super::Chatroom = payload.into();

        if let Some(latest_message_id) =
            ChatroomService::select_column_by_id(transaction, &chatroom_id, "latest_message_id")?
        {
            row.latest_message_id = latest_message_id
        }

        ChatroomService::save(transaction, row)
    }

    fn select_column_by_id<T: FromSql>(
        connection: &Connection,
        chatroom_id: &[u8],
        column: &str,
    ) -> rusqlite::Result<Option<T>> {
        let sql = format!(
            "SELECT {} FROM chatroom WHERE chatroom_id = ? LIMIT 1",
            column
        );
        super::unwrap_optional_row(connection.query_row(
            &sql,
            rusqlite::params![chatroom_id],
            |row| row.get(0),
        ))
    }
}

impl From<crate::changelog::Chatroom> for super::Chatroom {
    fn from(src: crate::changelog::Chatroom) -> Self {
        Self {
            chatroom_id: super::bytes_from_hash(src.id()),
            latest_message_id: vec![],
            time_updated: super::float_from_time(Utc::now()),
            inner: src.into(),
        }
    }
}
