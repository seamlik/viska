//! Data models used in the database.
//!
//! Data is mostly (de)serialized using [CBOR](https://cbor.io). Thier key in the database is not
//! included with itself.

use crate::pki::CertificateId;
use blake2::Blake2b;
use blake2::Digest;
use chrono::offset::Utc;
use chrono::DateTime;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::ExactSizeIterator;
use std::str::FromStr;

pub const DEFAULT_MIME: &Mime = &mime::TEXT_PLAIN_UTF_8;

/// UUID version 4.
pub type MessageId = [u8; 16];

/// Blake2b-512
pub type ChatroomId = Vec<u8>;

#[derive(Deserialize, Serialize)]
pub struct MessageHead {
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub sender: Address,

    #[serde(with = "serde_with::rust::display_fromstr")]
    pub mime: Mime,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct Chatroom {
    /// Self account is excluded.
    pub members: HashSet<CertificateId>,
}

/// Generates a Chatroom ID from its member IDs.
///
/// A chatroom's ID only depends on its members and nothing else, such that messages sent to the
/// same set of accounts are always stored in the same chatroom. The ID generation is reproducible
/// and not affected by the order of the members.
pub fn chatroom_id_from_members<'a>(
    members: impl ExactSizeIterator<Item = &'a CertificateId>,
) -> ChatroomId {
    let mut members_sorted: Vec<&'a CertificateId> = members.collect();
    members_sorted.sort();
    members_sorted.dedup();

    let mut digest = Blake2b::default();
    for it in members_sorted {
        digest.input(&it);
    }

    digest.result().into_iter().collect()
}

#[derive(Deserialize, Serialize)]
pub struct Vcard {
    pub avatar: Vec<u8>,
    pub description: String,
    pub devices: HashMap<CertificateId, DeviceInfo>,
    pub name: String,
    pub time_updated: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct DeviceInfo {
    pub name: String,
}

/// X.509 certificate encoded in ASN.1 DER.
pub type Certificate = Vec<u8>;

/// RFC 5958 PKCS #8 encoded in ASN.1 DER.
pub type CryptoKey = Vec<u8>;

/// Combination of an account ID and a device ID.
///
/// It is used to identify an entity a client can interact with. For example, specifying the
/// destination of a message.
///
/// Components are separated by a `/`. For example: `1A2B/3D4C`.
pub struct Address {
    pub account: Vec<u8>,
    pub device: Vec<u8>,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let convert = |data: &[u8]| data_encoding::HEXUPPER.encode(data);
        write!(f, "{}/{}", convert(&self.account), convert(&self.device))
    }
}

impl FromStr for Address {
    type Err = AddressFromStrError;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = src.split('/').collect();
        if parts.len() != 2 {
            Result::Err(AddressFromStrError("Does not contain exctly 2 components."))
        } else {
            let encoding = &data_encoding::HEXUPPER_PERMISSIVE;
            let account = encoding.decode(parts.get(0).unwrap().as_ref());
            let device = encoding.decode(parts.get(1).unwrap().as_ref());
            if account.is_err() {
                Result::Err(AddressFromStrError("Invalid account."))
            } else if device.is_err() {
                Result::Err(AddressFromStrError("Invalid device."))
            } else {
                Result::Ok(Address {
                    account: account.unwrap(),
                    device: device.unwrap(),
                })
            }
        }
    }
}

pub struct AddressFromStrError(&'static str);

impl Display for AddressFromStrError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Failed to parse address: {}", self.0)
    }
}
