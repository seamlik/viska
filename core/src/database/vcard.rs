use super::Vcard;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use rusqlite::Transaction;
use std::convert::AsRef;

pub struct VcardService;

impl VcardService {
    pub fn save(
        transaction: &Transaction,
        vcards: impl Iterator<Item = Vcard>,
    ) -> rusqlite::Result<()> {
        let sql = r#"
            REPLACE INTO vcard (
                vcard_id,
                account_id,
                name,
                photo,
                photo_mime
            ) VALUES (?);
        "#;
        for vcard in vcards {
            let vcard_id = vcard.canonical_id();
            let (photo, photo_mime) = vcard
                .photo
                .map(|blob| (blob.content, blob.mime))
                .unwrap_or_default();
            transaction.execute(
                sql,
                rusqlite::params![
                    vcard_id.as_bytes().as_ref(),
                    &vcard.account_id,
                    &vcard.name,
                    photo,
                    photo_mime
                ],
            )?;
        }
        Ok(())
    }
}

impl CanonicalId for super::Vcard {
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
