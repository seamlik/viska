use rusqlite::Transaction;

pub struct PeerService;

impl PeerService {
    pub fn save<'t>(
        transaction: &'t Transaction,
        payload: crate::changelog::Peer,
    ) -> rusqlite::Result<()> {
        let sql = r#"
            REPLACE INTO peer (
                account_id,
                name,
                role
            ) VALUES (?);
        "#;
        let params = rusqlite::params![payload.account_id, payload.name, payload.role,];
        transaction.execute(sql, params)?;
        Ok(())
    }
}
