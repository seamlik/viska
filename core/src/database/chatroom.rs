use super::schema::chatroom as Schema;
use super::schema::chatroom_members as SchemaMembers;
use super::Event;
use crate::changelog::Chatroom;
use crate::changelog::Message;
use blake3::Hash;
use blake3::Hasher;
use chrono::Utc;
use diesel::prelude::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

pub(crate) struct ChatroomService {
    pub event_sink: Sender<Arc<Event>>,
}

impl ChatroomService {
    /// Creates a [Chatroom](super::Chatroom) when receiving or sending a message belonging to a
    /// non-existing [Chatroom](super::Chatroom).
    fn create_for_message(message: &Message) -> Chatroom {
        let mut members = message.recipients.clone();
        members.push(message.sender.clone());

        // TODO: New name by fetching Vcard
        let name = "New chatroom".to_string();

        Chatroom { members, name }
    }

    /// Updates the [Chatroom](super::Chatroom) that is supposed to hold a new [Message].
    pub fn update_for_message(
        &self,
        connection: &'_ SqliteConnection,
        message: &crate::changelog::Message,
    ) -> QueryResult<()> {
        let chatroom_id = message.chatroom_id();
        // TODO: Simplify to 1 SQL
        if let Some(time_updated) = Schema::table
            .find(chatroom_id.as_bytes().as_ref())
            .select(Schema::time_updated)
            .first(connection)
            .optional()?
        {
            if message.time > time_updated {
                diesel::update(Schema::table.find(chatroom_id.as_bytes().as_ref()))
                    .set(Schema::time_updated.eq(time_updated))
                    .execute(connection)?;
            }
        } else {
            self.save(connection, &ChatroomService::create_for_message(message))?;
        }
        Ok(())
    }

    pub fn save(&self, connection: &'_ SqliteConnection, payload: &Chatroom) -> QueryResult<()> {
        let chatroom_id = super::bytes_from_hash(payload.chatroom_id());
        diesel::replace_into(Schema::table)
            .values((
                Schema::chatroom_id.eq(&chatroom_id),
                Schema::time_updated.eq(super::float_from_time(Utc::now())),
                Schema::name.eq(&payload.name),
            ))
            .execute(connection)?;
        Self::replace_members(connection, &chatroom_id, payload.members.iter())?;
        let _ = self.event_sink.send(Event::Chatroom { chatroom_id }.into());

        Ok(())
    }

    fn replace_members<'m>(
        connection: &'_ SqliteConnection,
        chatroom_id: &[u8],
        members: impl Iterator<Item = &'m Vec<u8>>,
    ) -> QueryResult<()> {
        diesel::delete(SchemaMembers::table.filter(SchemaMembers::chatroom_id.eq(chatroom_id)))
            .execute(connection)?;
        let members_sorted: BTreeSet<_> = members.collect();
        let rows: Vec<_> = members_sorted
            .into_iter()
            .map(|member| {
                (
                    SchemaMembers::id.eq(Uuid::new_v4().as_bytes().to_vec()),
                    SchemaMembers::chatroom_id.eq(chatroom_id),
                    SchemaMembers::member_account_id.eq(member),
                )
            })
            .collect();
        diesel::insert_into(SchemaMembers::table)
            .values(rows)
            .execute(connection)?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &SqliteConnection,
        id: &[u8],
    ) -> QueryResult<Option<crate::daemon::Chatroom>> {
        Schema::table
            .find(id)
            .select(Schema::columns::name)
            .first::<String>(connection)
            .map(|name| crate::daemon::Chatroom { name })
            .optional()
    }
}

impl crate::changelog::Chatroom {
    pub fn chatroom_id(&self) -> Hash {
        chatroom_id(self.members.iter())
    }
}

pub fn chatroom_id<'a>(members: impl Iterator<Item = &'a Vec<u8>>) -> Hash {
    let mut hasher = Hasher::default();
    hasher.update(b"Viska chatroom ID");

    let members_sorted: BTreeSet<&'a Vec<u8>> = members.collect();
    let length = members_sorted.iter().fold(0, |sum, x| sum + x.len());
    hasher.update(&length.to_be_bytes());
    for id in members_sorted {
        hasher.update(id);
    }

    hasher.finalize()
}
