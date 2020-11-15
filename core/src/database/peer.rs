use rusqlite::Connection;

pub struct PeerService;

impl PeerService {
    pub fn save(
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
        Ok(())
    }
}
