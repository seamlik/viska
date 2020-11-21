use crate::changelog::PeerRole;
use crate::daemon::Roster;
use crate::daemon::RosterItem;
use crate::event::Event;
use futures::channel::mpsc::UnboundedSender;
use rusqlite::Connection;

#[derive(Default)]
pub struct PeerService {
    pub event_sink: Option<UnboundedSender<Event>>,
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

        Ok(())
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
