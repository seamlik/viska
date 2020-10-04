use crate::transaction::Message;
use blake3::Hasher;
use prost::DecodeError;
use prost::Message as _;
use prost_types::Timestamp;
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

/// Calculates the ID of a Message.
#[riko::fun]
pub fn message_id(message_protobuf: ByteBuf) -> Result<ByteBuf, DecodeError> {
    let mut hasher = Hasher::default();
    let message = Message::decode(message_protobuf.as_slice())?;

    hasher.update(&message.sender);
    for account in message.recipients.iter() {
        hasher.update(&account);
    }
    if let Some(time) = message.time {
        hasher.update(&encode_protobuf_time(&time));
    }
    hasher.update(message.content.as_bytes());
    if let Some(attachment) = message.attachment {
        hasher.update(attachment.mime.as_bytes());
        hasher.update(&attachment.content);
    }

    let raw_hash: [u8; 32] = hasher.finalize().into();
    Ok(ByteBuf::from(raw_hash))
}

fn encode_protobuf_time(value: &Timestamp) -> [u8; 8] {
    let float = value.seconds as f64 + (value.nanos as f64) / 1_000_000_000.0;
    float.to_be_bytes()
}
