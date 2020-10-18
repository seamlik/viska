tonic::include_proto!("viska.database");

pub(crate) mod chatroom;
pub(crate) mod message;
pub(crate) mod peer;

use blake3::Hash;
use chrono::prelude::*;
use serde_bytes::ByteBuf;

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
