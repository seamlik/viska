use blake3::Hasher;
use hex::FromHexError;
use serde_bytes::ByteBuf;
use std::collections::BTreeSet;

/// THE hash function (BLAKE3) universally used in the project.
///
/// This is exported only because this algorithm isn't available in most languages or platform at moment.
#[riko::fun]
pub fn hash(src: &ByteBuf) -> ByteBuf {
    let raw_hash: [u8; 32] = blake3::hash(src).into();
    ByteBuf::from(raw_hash)
}

/// Encodes a binary ID (e.g. account ID) in the unified representation.
#[riko::fun]
pub fn display_id(id: &ByteBuf) -> String {
    hex::encode_upper(id)
}

/// Decodes a binary ID (e.g. account ID) from the unified representation.
#[riko::fun]
pub fn parse_id(id: &String) -> Result<ByteBuf, FromHexError> {
    hex::decode(id).map(ByteBuf::from)
}

/// Calculates the ID of a chatroom based on the account ID of its members.
#[riko::fun]
pub fn chatroom_id(members: BTreeSet<ByteBuf>) -> ByteBuf {
    let mut hasher = Hasher::default();
    for id in members {
        hasher.update(id.as_slice());
    }
    let raw_hash: [u8; 32] = hasher.finalize().into();
    ByteBuf::from(raw_hash)
}
