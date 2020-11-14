use super::Vcard;
use blake3::Hash;
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
            let vcard_id = vcard.vcard_id();
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

impl super::Vcard {
    pub fn vcard_id(&self) -> Hash {
        todo!()
    }
}
