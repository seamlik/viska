//! Database operations and models.
//!
//! A key-value store is used as the database backend. Data is serialized in [CBOR](https://cbor.io)
//! format and stored as the "value" in the database entries.

use crate::pki::CertificateId;
use crate::utils::Result;
use blake2::Blake2b;
use blake2::Digest;
use chrono::offset::Utc;
use chrono::DateTime;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use sled::IVec;
use sled::Tree;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::ExactSizeIterator;
use std::ops::Deref;
use std::str::FromStr;

pub const DEFAULT_MIME: &Mime = &mime::TEXT_PLAIN_UTF_8;

/// UUID version 4.
pub type MessageId = [u8; 16];

/// Blake2b-512
pub type ChatroomId = [u8];

/// X.509 certificate encoded in ASN.1 DER.
pub type Certificate = [u8];

/// RFC 5958 PKCS #8 encoded in ASN.1 DER.
pub type CryptoKey = [u8];

const TABLE_CHATROOMS: &str = "chatrooms";
const TABLE_MESSAGE_BODIES: &str = "message-bodies";
const TABLE_MESSAGE_HEADS: &str = "message-heads";
const TABLE_PROFILE: &str = "profile";
const TABLE_VCARDS: &str = "vcards";

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
    /// Set of Certificate IDs.
    pub members: HashSet<Vec<u8>>,
}

impl Chatroom {
    pub fn id(&self) -> Vec<u8> {
        unimplemented!()
    }
}

/// Generates a Chatroom ID from its member IDs.
///
/// A chatroom's ID only depends on its members and nothing else, such that messages sent to the
/// same set of accounts are always stored in the same chatroom. The ID generation is reproducible
/// and not affected by the order of the members.
pub fn chatroom_id_from_members<'a>(
    members: impl ExactSizeIterator<Item = &'a Vec<u8>>,
) -> Vec<u8> {
    let mut members_sorted: Vec<&Vec<u8>> = members.collect();
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
    pub devices: HashMap<Vec<u8>, DeviceInfo>,
    pub name: String,
    pub time_updated: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct DeviceInfo {
    pub name: String,
}

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
    fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = src.split('/').collect();
        if parts.len() != 2 {
            std::result::Result::Err(AddressFromStrError("Does not contain exctly 2 components."))
        } else {
            let encoding = &data_encoding::HEXUPPER_PERMISSIVE;
            let account = encoding.decode(parts.get(0).unwrap().as_ref());
            let device = encoding.decode(parts.get(1).unwrap().as_ref());
            if account.is_err() {
                std::result::Result::Err(AddressFromStrError("Invalid account."))
            } else if device.is_err() {
                std::result::Result::Err(AddressFromStrError("Invalid device."))
            } else {
                std::result::Result::Ok(Address {
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

/// Makes a key with a table.
fn tabled_key(table: &str, key: &str) -> String {
    format!("{}/{}", table, key)
}

/// Low-level operations for accessing a profile stored in a database.
pub trait RawProfile {
    fn account_certificate(&self) -> Result<Option<Vec<u8>>>;
    fn account_key(&self) -> Result<Option<Vec<u8>>>;
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<()>;
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<()>;
    fn blacklist(&self) -> Result<HashSet<Vec<u8>>>;
    fn device_certificate(&self) -> Result<Option<Vec<u8>>>;
    fn device_key(&self) -> Result<Option<Vec<u8>>>;
    fn set_account_certificate(&self, cert: &Certificate) -> Result<()>;
    fn set_account_key(&self, key: &CryptoKey) -> Result<()>;
    fn set_blacklist(&self, blacklist: &HashSet<Vec<u8>>) -> Result<()>;
    fn set_device_certificate(&self, cert: &Certificate) -> Result<()>;
    fn set_device_key(&self, key: &CryptoKey) -> Result<()>;
    fn set_whitelist(&self, blacklist: &HashSet<Vec<u8>>) -> Result<()>;
    fn whitelist(&self) -> Result<HashSet<Vec<u8>>>;
}

impl RawProfile for Tree {
    fn set_account_certificate(&self, cert: &Certificate) -> Result<()> {
        self.set(
            tabled_key(TABLE_PROFILE, "account-certificate"),
            cert.deref(),
        )?;
        Ok(())
    }
    fn set_account_key(&self, key: &CryptoKey) -> Result<()> {
        self.set(tabled_key(TABLE_PROFILE, "account-key"), key)?;
        Ok(())
    }
    fn set_device_certificate(&self, cert: &Certificate) -> Result<()> {
        self.set(
            tabled_key(TABLE_PROFILE, "device-certificate"),
            cert.deref(),
        )?;
        Ok(())
    }
    fn set_device_key(&self, key: &CryptoKey) -> Result<()> {
        self.set(tabled_key(TABLE_PROFILE, "device-key"), key)?;
        Ok(())
    }
    fn account_certificate(&self) -> Result<Option<Vec<u8>>> {
        self.get(tabled_key(TABLE_PROFILE, "account-certificate"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn account_key(&self) -> Result<Option<Vec<u8>>> {
        self.get(tabled_key(TABLE_PROFILE, "account-key"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_certificate(&self) -> Result<Option<Vec<u8>>> {
        self.get(tabled_key(TABLE_PROFILE, "device-certificate"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_key(&self) -> Result<Option<Vec<u8>>> {
        self.get(tabled_key(TABLE_PROFILE, "device-key"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn blacklist(&self) -> Result<HashSet<Vec<u8>>> {
        let raw = self.get(tabled_key(TABLE_PROFILE, "blacklist"))?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_blacklist(&self, blacklist: &HashSet<Vec<u8>>) -> Result<()> {
        let cbor = serde_cbor::to_vec(blacklist).unwrap();
        self.set(tabled_key(TABLE_PROFILE, "blacklist"), cbor)?;
        Ok(())
    }
    fn whitelist(&self) -> Result<HashSet<Vec<u8>>> {
        let raw = self.get(tabled_key(TABLE_PROFILE, "whitelist"))?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_whitelist(&self, whitelist: &HashSet<Vec<u8>>) -> Result<()> {
        let cbor = serde_cbor::to_vec(whitelist).unwrap();
        self.set(tabled_key(TABLE_PROFILE, "whitelist"), cbor)?;
        Ok(())
    }
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<()> {
        self.set(
            tabled_key(TABLE_VCARDS, &crate::utils::display_id(id)),
            serde_cbor::to_vec(vcard)?,
        )?;
        Ok(())
    }
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<()> {
        let chatroom_id =
            crate::utils::display_id(&chatroom_id_from_members(chatroom.members.iter()));
        self.set(
            tabled_key(TABLE_CHATROOMS, &chatroom_id),
            serde_cbor::to_vec(chatroom)?,
        )?;
        Ok(())
    }
}

trait IntoBytes {
    fn into(self) -> Option<Vec<u8>>;
}

impl IntoBytes for Option<IVec> {
    fn into(self) -> Option<Vec<u8>> {
        self.map(|raw| (*raw).into())
    }
}
