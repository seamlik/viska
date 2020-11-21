use super::peer::PeerService;
use crate::changelog::Vcard;
use crate::event::Event;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use futures::channel::mpsc::UnboundedSender;
use rusqlite::Connection;
use std::convert::AsRef;

#[derive(Default)]
pub struct VcardService {
    pub event_sink: Option<UnboundedSender<Event>>,
}

impl VcardService {
    pub fn save(
        &self,
        connection: &'_ Connection,
        vcards: impl Iterator<Item = Vcard>,
    ) -> rusqlite::Result<()> {
        let sql = r#"
            REPLACE INTO vcard (
                vcard_id,
                account_id,
                name,
                photo,
                photo_mime
            ) VALUES (?1, ?2, ?3, ?4, ?5);
        "#;
        let mut stmt = connection.prepare_cached(sql)?;

        for vcard in vcards {
            let vcard_id = vcard.canonical_id();
            let (photo, photo_mime) = vcard
                .photo
                .map(|blob| (blob.content, blob.mime))
                .unwrap_or_default();
            stmt.execute(rusqlite::params![
                vcard_id.as_bytes().as_ref(),
                &vcard.account_id,
                &vcard.name,
                photo,
                photo_mime
            ])?;

            // Publish events
            if let Some(sink) = &self.event_sink {
                if let Err(_) = sink.unbounded_send(Event::Vcard {
                    account_id: vcard.account_id.clone(),
                }) {
                    continue;
                }
                if PeerService::is_in_roster(connection, &vcard.account_id)? {
                    let _ = sink.unbounded_send(Event::Roster);
                }
            }
        }
        Ok(())
    }

    pub fn find_by_account_id(
        connection: &'_ Connection,
        account_id: &[u8],
    ) -> rusqlite::Result<Option<crate::daemon::Vcard>> {
        let mut stmt = connection
            .prepare_cached("SELECT account_id, name FROM vcard WHERE account_id = ? LIMIT 1;")?;
        super::unwrap_optional_row(stmt.query_row(rusqlite::params![&account_id], |row| {
            Ok(crate::daemon::Vcard {
                account_id: row.get("account_id")?,
                name: row.get("name")?,
            })
        }))
    }
}

impl CanonicalId for crate::changelog::Vcard {
    fn canonical_id(&self) -> Hash {
        let mut hasher = Hasher::default();

        hasher.update(b"Viska vCard");

        hasher.update(&self.account_id.len().to_be_bytes());
        hasher.update(&self.account_id);

        hasher.update(&self.name.len().to_be_bytes());
        hasher.update(self.name.as_bytes());

        if let Some(photo) = &self.photo {
            hasher.update(&blake3::OUT_LEN.to_be_bytes());
            hasher.update(photo.canonical_id().as_bytes());
        }

        hasher.finalize()
    }
}
