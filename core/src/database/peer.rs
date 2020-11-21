use crate::changelog::PeerRole;
use crate::daemon::Roster;
use crate::daemon::RosterItem;
use crate::endpoint::CertificateVerifier;
use crate::event::Event;
use futures::channel::mpsc::UnboundedSender;
use rusqlite::Connection;
use std::sync::Arc;

#[derive(Default)]
pub struct PeerService {
    pub event_sink: Option<UnboundedSender<Event>>,
    pub verifier: Option<Arc<CertificateVerifier>>,
}

impl PeerService {
    pub fn save(
        &self,
        connection: &'_ Connection,
        payload: crate::changelog::Peer,
    ) -> rusqlite::Result<()> {
        let sql = r#"
            REPLACE INTO peer (
                account_id,
                name,
                role
            ) VALUES (?1, ?2, ?3);
        "#;
        let mut stmt = connection.prepare_cached(sql)?;
        stmt.execute(rusqlite::params![
            payload.account_id,
            payload.name,
            payload.role
        ])?;

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

    pub fn blacklist(connection: &'_ Connection) -> rusqlite::Result<Vec<Vec<u8>>> {
        let blocked_i32: i32 = PeerRole::Blocked.into();
        connection
            .prepare_cached("SELECT account_id FROM peer WHERE role = ?")?
            .query_map(rusqlite::params![blocked_i32], |row| {
                let result: Vec<u8> = row.get(0)?;
                Ok(result)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
    }

    pub fn roster(connection: &'_ Connection) -> rusqlite::Result<Roster> {
        let sql = r#"
            SELECT peer.name, vcard.name
            FROM peer
            JOIN vcard ON peer.account_id = vcard.account_id;"#;
        let mut stmt = connection.prepare_cached(sql)?;
        let roster_list: rusqlite::Result<Vec<_>> = stmt
            .query_map(rusqlite::NO_PARAMS, |row| {
                let peer_name: String = row.get(0)?;
                let display_name = if peer_name.is_empty() {
                    row.get(1)?
                } else {
                    peer_name
                };
                Ok(RosterItem { name: display_name })
            })?
            .collect();
        Ok(Roster {
            roster: roster_list?,
        })
    }

    pub fn is_in_roster(
        connection: &'_ Connection,
        account_id: &'_ [u8],
    ) -> rusqlite::Result<bool> {
        let sql = r#"
            SELECT EXISTS (
                SELECT 1 FROM peer WHERE (peer.account_id = ?1 AND peer.role = ?2) LIMIT 1
            );
        "#;
        let mut stmt = connection.prepare_cached(sql)?;

        let friend_i32: i32 = PeerRole::Friend.into();
        stmt.query_row(rusqlite::params![account_id, friend_i32], |row| {
            let result: u8 = row.get(0)?;
            Ok(result == 1)
        })
    }
}
