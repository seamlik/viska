//! Database operations and models.
//!
//! A key-value store is used as the database backend. Data is serialized in [CBOR](https://cbor.io)
//! format and stored as the "value" in the database entries, while their "key" is described in
//! their summaries.

use crate::pki::CertificateId;
use crate::Result;
use blake2::Blake2b;
use blake2::Digest;
use chrono::offset::Utc;
use chrono::DateTime;
use derive_more::Display;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use sled::Db;
use sled::IVec;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::ExactSizeIterator;
use std::str::FromStr;
use uuid::Uuid;

pub const DEFAULT_MIME: &Mime = &mime::TEXT_PLAIN_UTF_8;
fn default_mime() -> Mime {
    DEFAULT_MIME.clone()
}
fn is_default_mime(value: &Mime) -> bool {
    value == DEFAULT_MIME
}

/// Blake2b-512
pub type ChatroomId = [u8];

/// X.509 certificate encoded in ASN.1 DER.
pub type Certificate = [u8];

/// RFC 5958 PKCS #8 encoded in ASN.1 DER.
pub type CryptoKey = [u8];

const TABLE_CHATROOMS: &str = "chatrooms";
const TABLE_BLOBS: &str = "blobs";
const TABLE_PROFILE: &str = "profile";
const TABLE_VCARDS: &str = "vcards";

fn table_messages(chatroom_id: &ChatroomId) -> Vec<u8> {
    format!("messages-{}", chatroom_id.display()).into()
}

/// Meta-info of a message.
///
/// Stored in table `messages-{chatroom ID}` with raw `MessageID` as key.
///
/// The body is stored in the "blobs" table. This is because a message body usually has a variable
/// size and poses unstable overhead of querying `Message`s.
#[derive(Deserialize, Serialize)]
pub struct Message {
    #[serde(default = "default_mime")]
    #[serde(skip_serializing_if = "is_default_mime")]
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub mime: Mime,

    #[serde(with = "serde_with::rust::display_fromstr")]
    pub sender: Address,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
}

/// Meta-info of a chatroom.
///
/// Stored in table `chatrooms` with raw `Uuid` as key.
#[derive(Deserialize, Serialize)]
pub struct Chatroom {
    /// Set of Certificate IDs.
    pub members: HashSet<Vec<u8>>,
}

impl Chatroom {
    pub fn id(&self) -> Vec<u8> {
        chatroom_id_from_members(self.members.iter())
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
    if members.len() == 0 {
       return Vec::default();
    }
    let mut members_sorted: Vec<&Vec<u8>> = members.collect();
    members_sorted.sort();
    members_sorted.dedup();

    let mut digest = Blake2b::default();
    for it in members_sorted {
        digest.input(&it);
    }

    digest.result().into_iter().collect()
}

/// Public information of an account.
///
/// Stored in table `vcards` with raw account ID as key.
///
/// Since an avatar is usually binary data in variable sizes, it is stored in the "blobs" table with
/// key `avatar-{account ID}`.
#[derive(Deserialize, Serialize)]
pub struct Vcard {
    pub description: String,
    /// Devices registered with this account.
    ///
    /// Key represents the device ID.
    pub devices: HashMap<Vec<u8>, Device>,
    pub name: String,
    pub time_updated: DateTime<Utc>,
}

/// Meta-info of a device registered to an account.
///
/// This is a sub-level structure and is stored within a `Vcard`.
#[derive(Deserialize, Serialize)]
pub struct Device {
    pub name: String,
}

/// Combination of an account ID and a device ID.
///
/// It is used to identify an entity a client can interact with. For example, specifying the
/// destination of a message.
///
/// Components are separated by a `/`. For example: `1A2B/3D4C`.
#[derive(Display)]
#[display(fmt = "{}/{}", "account.display()", "device.display()")]
pub struct Address {
    pub account: Vec<u8>,
    pub device: Vec<u8>,
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

/// If failed to parse an `Address` from a string.
#[derive(Display)]
#[display(fmt = "Failed to parse address: {}", _0)]
pub struct AddressFromStrError(&'static str);

/// Low-level operations for accessing a profile stored in a database.
pub(crate) trait RawDatabase {
    fn account_certificate(&self) -> Result<Option<Vec<u8>>>;
    fn account_key(&self) -> Result<Option<Vec<u8>>>;
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<()>;
    fn add_message(
        &self,
        id: &Uuid,
        head: Message,
        body: Vec<u8>,
        chatroom_id: &ChatroomId,
    ) -> Result<()>;
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

impl RawDatabase for Db {
    fn set_account_certificate(&self, cert: &Certificate) -> Result<()> {
        self.open_tree(TABLE_PROFILE)?
            .set("account-certificate", cert)?;
        Ok(())
    }
    fn set_account_key(&self, key: &CryptoKey) -> Result<()> {
        self.open_tree(TABLE_PROFILE)?.set("account-key", key)?;
        Ok(())
    }
    fn set_device_certificate(&self, cert: &Certificate) -> Result<()> {
        self.open_tree(TABLE_PROFILE)?
            .set("device-certificate", cert)?;
        Ok(())
    }
    fn set_device_key(&self, key: &CryptoKey) -> Result<()> {
        self.open_tree(TABLE_PROFILE)?.set("device-key", key)?;
        Ok(())
    }
    fn account_certificate(&self) -> Result<Option<Vec<u8>>> {
        self.open_tree(TABLE_PROFILE)?
            .get("account-certificate")
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn account_key(&self) -> Result<Option<Vec<u8>>> {
        self.open_tree(TABLE_PROFILE)?
            .get("account-key")
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_certificate(&self) -> Result<Option<Vec<u8>>> {
        self.open_tree(TABLE_PROFILE)?
            .get("device-certificate")
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_key(&self) -> Result<Option<Vec<u8>>> {
        self.open_tree(TABLE_PROFILE)?
            .get("device-key")
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn blacklist(&self) -> Result<HashSet<Vec<u8>>> {
        let raw = self.open_tree(TABLE_PROFILE)?.get("blacklist")?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_blacklist(&self, blacklist: &HashSet<Vec<u8>>) -> Result<()> {
        let cbor = serde_cbor::to_vec(blacklist).unwrap();
        self.open_tree(TABLE_PROFILE)?.set("blacklist", cbor)?;
        Ok(())
    }
    fn whitelist(&self) -> Result<HashSet<Vec<u8>>> {
        let raw = self.open_tree(TABLE_PROFILE)?.get("whitelist")?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_whitelist(&self, whitelist: &HashSet<Vec<u8>>) -> Result<()> {
        let cbor = serde_cbor::to_vec(whitelist).unwrap();
        self.open_tree(TABLE_PROFILE)?.set("whitelist", cbor)?;
        Ok(())
    }
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<()> {
        self.open_tree(TABLE_VCARDS)?
            .set(id, serde_cbor::to_vec(vcard)?)?;
        Ok(())
    }
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<()> {
        self.open_tree(TABLE_CHATROOMS)?
            .set(chatroom.id(), serde_cbor::to_vec(chatroom)?)?;
        Ok(())
    }
    fn add_message(
        &self,
        id: &Uuid,
        head: Message,
        body: Vec<u8>,
        chatroom_id: &ChatroomId,
    ) -> Result<()> {
        if chatroom_id.is_empty() {
            log::warn!("Message is being sent to an empty chatroom, ignoring.");
            return Ok(())
        }

        let message_key: IVec = id.as_bytes().into();
        self.open_tree(table_messages(chatroom_id))?
            .set(message_key, serde_cbor::to_vec(&head)?)?;

        let blob_key = format!("message-{}", id.to_hyphenated_ref());
        self.open_tree(TABLE_BLOBS)?.set(blob_key, body)?;

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

/// The unified way of displaying an ID byte string, which is uppercase Hex.
pub(crate) trait DisplayableId {
    fn display(&self) -> String;
}

impl DisplayableId for [u8] {
    fn display(&self) -> String {
        data_encoding::HEXUPPER_PERMISSIVE.encode(&self)
    }
}
