use super::schema::object as Schema;
use crate::changelog::Blob;
use diesel::prelude::*;
use uuid::Uuid;

pub struct ObjectService;

impl ObjectService {
    pub fn save(connection: &'_ SqliteConnection, payload: Blob) -> QueryResult<Uuid> {
        let id = Uuid::new_v4();
        diesel::replace_into(Schema::table)
            .values((
                Schema::object_id.eq(id.as_bytes().as_ref()),
                Schema::content.eq(payload.content),
                Schema::mime.eq(payload.mime),
            ))
            .execute(connection)?;
        Ok(id)
    }
}
