use super::chatroom::ChatroomService;
use super::object::ObjectService;
use super::schema::message as Schema;
use super::schema::message_recipients as SchemaRecipients;
use super::Event;
use crate::changelog::Message;
use crate::daemon::ChatroomMessagesSubscription;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use diesel::prelude::*;
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) struct MessageService;

impl MessageService {
    fn save(connection: &'_ SqliteConnection, payload: &Message) -> QueryResult<Event> {
        let message_id = super::bytes_from_hash(payload.canonical_id());
        let chatroom_id = super::bytes_from_hash(payload.chatroom_id());
        let attachment_id: Option<Vec<u8>> = payload
            .attachment
            .as_ref()
            .map(|obj| ObjectService::save(connection, obj))
            .transpose()?
            .map(|id| id.as_bytes().as_ref().into());

        diesel::replace_into(Schema::table)
            .values((
                Schema::message_id.eq(&message_id),
                Schema::chatroom_id.eq(&chatroom_id),
                Schema::attachment.eq(attachment_id),
                Schema::content.eq(&payload.content),
                Schema::sender.eq(&payload.sender),
                Schema::time.eq(&payload.time),
            ))
            .execute(connection)?;
        Self::replace_recipients(connection, &message_id, payload.recipients.iter())?;
        Ok(Event::Message { chatroom_id })
    }

    pub fn update(connection: &'_ SqliteConnection, payload: &Message) -> QueryResult<Event> {
        // Update chatroom
        ChatroomService::update_for_message(connection, &payload)?;

        // Update message
        Self::save(connection, &payload)
    }

    fn replace_recipients<'m>(
        connection: &'_ SqliteConnection,
        message_id: &[u8],
        recipients: impl Iterator<Item = &'m Vec<u8>>,
    ) -> QueryResult<()> {
        diesel::delete(SchemaRecipients::table.filter(SchemaRecipients::message_id.eq(message_id)))
            .execute(connection)?;
        let recipients_sorted: BTreeSet<_> = recipients.collect();
        let rows: Vec<_> = recipients_sorted
            .into_iter()
            .map(|recipient| {
                (
                    SchemaRecipients::id.eq(Uuid::new_v4().as_bytes().to_vec()),
                    SchemaRecipients::message_id.eq(message_id),
                    SchemaRecipients::recipient_account_id.eq(recipient),
                )
            })
            .collect();
        diesel::insert_into(SchemaRecipients::table)
            .values(rows)
            .execute(connection)?;
        Ok(())
    }

    pub fn find_by_chatroom(
        connection: &SqliteConnection,
        chatroom_id: &[u8],
    ) -> QueryResult<ChatroomMessagesSubscription> {
        todo!()
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
