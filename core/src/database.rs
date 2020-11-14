//! Database models and operations.

tonic::include_proto!("viska.database");

pub(crate) mod chatroom;
pub(crate) mod message;
pub(crate) mod peer;
pub(crate) mod vcard;

use blake3::Hash;
use chrono::prelude::*;
use rusqlite::Connection;
use serde_bytes::ByteBuf;
use std::path::PathBuf;
use std::sync::Mutex;

/// THE hash function (BLAKE3) universally used in the project.
///
/// This is exported only because this algorithm isn't available in most languages or platform at moment.
#[riko::fun]
pub fn hash(src: &ByteBuf) -> ByteBuf {
    let raw_hash: [u8; 32] = blake3::hash(src).into();
    ByteBuf::from(raw_hash)
}

/// Serializes a timestamp to a floating point number.
///
/// By using a floating point number as the universal timestamp format, we can have arbitrary
/// precision on the time value.
pub(crate) fn float_from_time(src: DateTime<Utc>) -> f64 {
    src.timestamp() as f64 + src.timestamp_subsec_nanos() as f64 / 1_000_000_000.0
}

/// Converts a [Hash] to bytes.
pub(crate) fn bytes_from_hash(src: Hash) -> Vec<u8> {
    let raw_hash: [u8; 32] = src.into();
    raw_hash.to_vec()
}

#[derive(Default)]
pub struct Config {
    pub storage: Storage,
}

pub enum Storage {
    InMemory,
    OnDisk(PathBuf),
}

impl Default for Storage {
    fn default() -> Self {
        Self::InMemory
    }
}

pub(crate) struct Database {
    pub connection: Mutex<Connection>,
}

impl Database {
    pub fn create(config: Config) -> rusqlite::Result<Self> {
        let mut connection = match config.storage {
            Storage::InMemory => Connection::open_in_memory()?,
            Storage::OnDisk(path) => Connection::open(path)?,
        };

        connection
            .transaction()?
            .execute_batch(include_str!("database/migration/genesis.sql"))?;

        Ok(Self {
            connection: connection.into(),
        })
    }
}

fn unwrap_optional_row<T>(result: rusqlite::Result<T>) -> rusqlite::Result<Option<T>> {
    if let Err(rusqlite::Error::QueryReturnedNoRows) = result {
        Ok(None)
    } else {
        result.map(|inner| inner.into())
    }
}
