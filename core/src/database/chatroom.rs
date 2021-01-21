use super::schema::chatroom as Schema;
use super::BytesArray;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use chrono::Utc;
use diesel::prelude::*;
use prost::Message as _;
use std::collections::BTreeSet;

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

        let chatroom_id: [u8; 32] = chatroom_inner.chatroom_id().into();
        let latest_message_id: [u8; 32] = message.canonical_id().into();

        super::Chatroom {
            inner: Some(chatroom_inner),
            time_updated: message.time,
            chatroom_id: chatroom_id.to_vec(),
            latest_message_id: latest_message_id.to_vec(),
        }
    }

    /// Updates the [Chatroom](super::Chatroom) that is supposed to hold a new [Message].
    pub fn update_for_message(
        connection: &'_ SqliteConnection,
        message: &crate::changelog::Message,
    ) -> QueryResult<()> {
        let chatroom_id = message.chatroom_id();
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
            ChatroomService::save(connection, ChatroomService::create_for_message(message))?;
        }
        Ok(())
    }

    fn save(connection: &'_ SqliteConnection, payload: super::Chatroom) -> QueryResult<()> {
        let inner = payload.inner.unwrap();

        let mut members = Vec::<u8>::default();
        BytesArray {
            array: inner.members,
        }
        .encode(&mut members)
        .unwrap();

        diesel::replace_into(Schema::table)
            .values((
                Schema::chatroom_id.eq(payload.chatroom_id),
                Schema::latest_message_id.eq(payload.latest_message_id),
                Schema::time_updated.eq(payload.time_updated),
                Schema::name.eq(inner.name),
                Schema::members.eq(members),
            ))
            .execute(connection)?;
        Ok(())
    }

    pub fn update(
        connection: &'_ SqliteConnection,
        payload: crate::changelog::Chatroom,
    ) -> QueryResult<()> {
        let chatroom_id = super::bytes_from_hash(payload.chatroom_id());
        let mut row: super::Chatroom = payload.into();

        if let Some(latest_message_id) = Schema::table
            .find(&chatroom_id)
            .select(Schema::latest_message_id)
            .first(connection)
            .optional()?
        {
            row.latest_message_id = latest_message_id
        }

        ChatroomService::save(connection, row)
    }
}

impl From<crate::changelog::Chatroom> for super::Chatroom {
    fn from(src: crate::changelog::Chatroom) -> Self {
        Self {
            chatroom_id: super::bytes_from_hash(src.chatroom_id()),
            latest_message_id: vec![],
            time_updated: super::float_from_time(Utc::now()),
            inner: src.into(),
        }
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
