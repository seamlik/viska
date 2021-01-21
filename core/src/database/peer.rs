use super::schema::peer as Schema;
use crate::changelog::PeerRole;
use crate::daemon::Roster;
use crate::daemon::RosterItem;
use crate::endpoint::CertificateVerifier;
use crate::event::Event;
use diesel::prelude::*;
use futures::channel::mpsc::UnboundedSender;
use std::sync::Arc;

#[derive(Default)]
pub struct PeerService {
    pub event_sink: Option<UnboundedSender<Event>>,
    pub verifier: Option<Arc<CertificateVerifier>>,
}

impl PeerService {
    pub fn save(
        &self,
        connection: &'_ SqliteConnection,
        payload: crate::changelog::Peer,
    ) -> QueryResult<()> {
        diesel::replace_into(Schema::table)
            .values((
                Schema::columns::account_id.eq(payload.account_id),
                Schema::columns::name.eq(payload.name),
                Schema::columns::role.eq(payload.role),
            ))
            .execute(connection)?;

        // Publish event
        if let Some(sink) = &self.event_sink {
            let _ = sink.unbounded_send(Event::Roster);
        }

        // Update certificate verifier rules
        if let Some(verifier) = &self.verifier {
            let blacklist = Self::blacklist(connection)?;
            log::info!(
                "Updating certificate blacklist to: {:?}",
                blacklist
                    .iter()
                    .map(|id| hex::encode_upper(id))
                    .collect::<Vec<_>>()
            );
            verifier.set_rules(std::iter::empty(), blacklist)
        }

        Ok(())
    }

    pub fn blacklist(connection: &'_ SqliteConnection) -> QueryResult<Vec<Vec<u8>>> {
        let blocked_i32: i32 = PeerRole::Blocked.into();
        Schema::table
            .select(Schema::account_id)
            .filter(Schema::role.eq(blocked_i32))
            .load(connection)
    }

    pub fn roster(connection: &'_ SqliteConnection) -> QueryResult<Roster> {
        let result = Schema::table
            .left_join(
                super::schema::vcard::table
                    .on(super::schema::vcard::account_id.eq(Schema::account_id)),
            )
            .select((Schema::name, super::schema::vcard::name.nullable()))
            .load::<(String, Option<String>)>(connection)?
            .into_iter()
            .map(|(peer_name, vcard_name)| RosterItem {
                name: if peer_name.is_empty() {
                    vcard_name.unwrap_or_default()
                } else {
                    peer_name
                },
            })
            .collect();
        Ok(Roster { roster: result })
    }

    pub fn is_in_roster(
        connection: &'_ SqliteConnection,
        account_id: &'_ [u8],
    ) -> QueryResult<bool> {
        let friend_i32: i32 = PeerRole::Friend.into();
        diesel::select(diesel::dsl::exists(
            Schema::table.filter(
                Schema::account_id
                    .eq(account_id)
                    .and(Schema::role.eq(friend_i32)),
            ),
        ))
        .first(connection)
    }
}
