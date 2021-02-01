use super::object::ObjectService;
use super::peer::PeerService;
use super::schema::vcard as Schema;
use super::Event;
use crate::changelog::Vcard;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use diesel::prelude::*;
use std::convert::AsRef;

pub(crate) struct VcardService;

impl VcardService {
    pub fn save(
        connection: &'_ SqliteConnection,
        vcards: impl Iterator<Item = Vcard>,
    ) -> QueryResult<Vec<Event>> {
        let mut events = vec![];
        // TODO: Batch insert
        for vcard in vcards {
            let vcard_id = vcard.canonical_id();

            let photo_id: Option<Vec<u8>> = vcard
                .photo
                .as_ref()
                .map(|obj| ObjectService::save(connection, obj))
                .transpose()?
                .map(|id| id.as_bytes().as_ref().into());

            diesel::replace_into(Schema::table)
                .values((
                    Schema::columns::vcard_id.eq(vcard_id.as_bytes().as_ref()),
                    Schema::columns::account_id.eq(&vcard.account_id),
                    Schema::columns::name.eq(&vcard.name),
                    Schema::columns::photo.eq(photo_id),
                ))
                .execute(connection)?;

            // Publish events
            events.push(Event::Vcard {
                account_id: vcard.account_id.clone(),
            });
            if PeerService::is_in_roster(connection, &vcard.account_id)? {
                events.push(Event::Roster);
            }
        }
        Ok(events)
    }

    pub fn find_by_account_id(
        connection: &'_ SqliteConnection,
        account_id: &[u8],
    ) -> QueryResult<Option<crate::daemon::Vcard>> {
        let result = Schema::table
            .select((Schema::columns::account_id, Schema::columns::name))
            .filter(Schema::account_id.eq(account_id))
            .first::<(Vec<u8>, String)>(connection)
            .optional()?
            .map(|(account_id, name)| crate::daemon::Vcard { account_id, name });
        Ok(result)
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
