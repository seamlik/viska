//! Database operations and models.
//!
//! A key-value store is used as the database backend. Data is serialized in [CBOR](https://cbor.io)
//! format and stored as the "value" in the database entries, while their "key" is described in
//! their summaries.

use crate::pki::CertificateId;
use crate::utils::ResultOption;
use blake3::Hash;
use blake3::Hasher;
use chrono::offset::Utc;
use chrono::DateTime;
use derive_more::Display;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use sled::Db;
use sled::IVec;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::error::Error;
use std::result::Result;
use uuid::Uuid;

pub const DEFAULT_MIME: &Mime = &mime::TEXT_PLAIN_UTF_8;
fn default_mime() -> Mime {
    DEFAULT_MIME.clone()
}
fn is_default_mime(value: &Mime) -> bool {
    value == DEFAULT_MIME
}

/// BLAKE3
pub type ChatroomId = [u8; 32];

/// X.509 certificate encoded in ASN.1 DER.
pub type Certificate = [u8];

/// RFC 5958 PKCS #8 encoded in ASN.1 DER.
pub type CryptoKey = [u8];

/* Tables for profile */
const TABLE_CHATROOMS: &str = "chatrooms";
const TABLE_MESSAGE_BODY: &str = "message-body";
const TABLE_MESSAGE_HEAD: &str = "message-head";
const TABLE_PROFILE: &str = "profile";

/* Tables for cache */
const TABLE_VCARD: &str = "vcard";

/// Meta-info of a message.
///
/// Stored in table `messages-{chatroom ID}` with raw message ID (a UUID v4) as key.
///
/// The body is stored in the "blobs" table. This is because a message body usually has a variable
/// size and poses unstable overhead when querying [MessageHead]s.
#[derive(Deserialize, Serialize)]
pub struct MessageHead {
    #[serde(default = "default_mime")]
    #[serde(skip_serializing_if = "is_default_mime")]
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub mime: Mime,

    /// Set of certificate IDs.
    pub recipients: BTreeSet<CertificateId>,

    pub sender: CertificateId,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
}

/// Meta-info of a chatroom.
///
/// Stored in table `chatrooms` with raw [ChatroomId] as key.
#[derive(Deserialize, Serialize)]
pub struct Chatroom {
    /// Set of Certificate IDs.
    pub members: BTreeSet<CertificateId>,
}

impl Chatroom {
    /// Calculates its ID which is the BLAKE3 hash of the certificate IDs of all of its members.
    /// A chatroom's ID only depends on its members and nothing else, such that messages sent to
    /// the same set of accounts are always stored in the same chatroom. The ID generation is
    /// reproducible and not affected by the order of the members.
    pub fn id(&self) -> Hash {
        let mut hasher = Hasher::default();
        for m in self.members.iter() {
            hasher.update(m);
        }
        hasher.finalize()
    }
}

/// Public information of an account.
///
/// Stored in table `vcards` with raw account ID as key.
#[derive(Deserialize, Serialize)]
pub struct Vcard {
    pub name: String,
    pub time_updated: DateTime<Utc>,
}

/// Low-level operations for accessing a profile stored in a database.
pub(crate) trait Profile {
    fn certificate(&self) -> Result<Option<Vec<u8>>, sled::Error>;
    fn key(&self) -> Result<Option<Vec<u8>>, sled::Error>;
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<(), IoError>;
    fn add_message(&self, id: &Uuid, head: MessageHead, body: Vec<u8>) -> Result<(), IoError>;
    fn blacklist(&self) -> Result<HashSet<Vec<u8>>, IoError>;
    fn set_certificate(&self, cert: &Certificate) -> Result<(), sled::Error>;
    fn set_key(&self, key: &CryptoKey) -> Result<(), sled::Error>;
    fn set_blacklist(&self, blacklist: &HashSet<CertificateId>) -> Result<(), IoError>;
    fn set_whitelist(&self, blacklist: &HashSet<CertificateId>) -> Result<(), IoError>;
    fn vcard(&self) -> Result<Option<Vcard>, IoError>;
    fn set_vcard(&self, vcard: &Vcard) -> Result<(), IoError>;
    fn watch_vcard(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Vcard>, IoError>> + Send>, IoError>;
    fn whitelist(&self) -> Result<HashSet<Vec<u8>>, IoError>;
}

impl Profile for Db {
    fn set_certificate(&self, cert: &Certificate) -> Result<(), sled::Error> {
        self.open_tree(TABLE_PROFILE)?.insert("certificate", cert)?;
        Ok(())
    }
    fn set_key(&self, key: &CryptoKey) -> Result<(), sled::Error> {
        self.open_tree(TABLE_PROFILE)?.insert("key", key)?;
        Ok(())
    }
    fn certificate(&self) -> Result<Option<Vec<u8>>, sled::Error> {
        self.open_tree(TABLE_PROFILE)?
            .get("certificate")
            .map(IntoBytes::into)
    }
    fn key(&self) -> Result<Option<Vec<u8>>, sled::Error> {
        self.open_tree(TABLE_PROFILE)?
            .get("key")
            .map(IntoBytes::into)
    }
    fn blacklist(&self) -> Result<HashSet<Vec<u8>>, IoError> {
        let raw = self.open_tree(TABLE_PROFILE)?.get("blacklist")?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_blacklist(&self, blacklist: &HashSet<CertificateId>) -> Result<(), IoError> {
        let cbor = serde_cbor::to_vec(blacklist)?;
        self.open_tree(TABLE_PROFILE)?.insert("blacklist", cbor)?;
        Ok(())
    }
    fn whitelist(&self) -> Result<HashSet<Vec<u8>>, IoError> {
        let raw = self.open_tree(TABLE_PROFILE)?.get("whitelist")?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_whitelist(&self, whitelist: &HashSet<CertificateId>) -> Result<(), IoError> {
        let cbor = serde_cbor::to_vec(whitelist)?;
        self.open_tree(TABLE_PROFILE)?.insert("whitelist", cbor)?;
        Ok(())
    }
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<(), IoError> {
        self.open_tree(TABLE_CHATROOMS)?
            .insert(chatroom.id().as_bytes(), serde_cbor::to_vec(chatroom)?)?;
        Ok(())
    }
    fn add_message(&self, id: &Uuid, head: MessageHead, body: Vec<u8>) -> Result<(), IoError> {
        let message_key: IVec = id.as_bytes().into();
        self.open_tree(TABLE_MESSAGE_HEAD)?
            .insert(message_key, serde_cbor::to_vec(&head)?)?;
        self.open_tree(TABLE_MESSAGE_BODY)?
            .insert(id.as_bytes(), body)?;

        Ok(())
    }
    fn vcard(&self) -> Result<Option<Vcard>, IoError> {
        self.open_tree(TABLE_PROFILE)?
            .get("vcard")
            .map_deep(|raw| serde_cbor::from_slice(raw.as_ref()).unwrap())
            .map_err(Into::into)
    }
    fn watch_vcard(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Vcard>, IoError>> + Send>, IoError> {
        let result =
            self.open_tree(TABLE_PROFILE)?
                .watch_prefix("vcard")
                .map(|event| match event {
                    sled::Event::Insert(_, raw) => serde_cbor::from_slice(&raw).map_err(Into::into),
                    sled::Event::Remove(_) => Ok(None),
                });
        Ok(Box::new(result))
    }
    fn set_vcard(&self, vcard: &Vcard) -> Result<(), IoError> {
        self.open_tree(TABLE_PROFILE)?
            .insert("vcard", serde_cbor::to_vec(&vcard)?)?;
        Ok(())
    }
}

pub(crate) trait Cache {
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<(), IoError>;
    fn vcard(&self, id: &CertificateId) -> Result<Option<Vcard>, IoError>;
    fn watch_vcard(
        &self,
        id: &CertificateId,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Vcard>, IoError>> + Send>, IoError>;
}

impl Cache for Db {
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<(), IoError> {
        self.open_tree(TABLE_VCARD)?
            .insert(id, serde_cbor::to_vec(vcard)?)?;
        Ok(())
    }
    fn vcard(&self, id: &CertificateId) -> Result<Option<Vcard>, IoError> {
        self.open_tree(TABLE_VCARD)?
            .get(id)
            .map_deep(|raw| serde_cbor::from_slice(raw.as_ref()).unwrap())
            .map_err(Into::into)
    }
    fn watch_vcard(
        &self,
        id: &CertificateId,
    ) -> Result<Box<dyn Iterator<Item = Result<Option<Vcard>, IoError>> + Send>, IoError> {
        let result = self
            .open_tree(TABLE_VCARD)?
            .watch_prefix(id)
            .map(|event| match event {
                sled::Event::Insert(_, raw) => serde_cbor::from_slice(&raw).map_err(Into::into),
                sled::Event::Remove(_) => Ok(None),
            });
        Ok(Box::new(result))
    }
}

/// When fail to perform a database access operation.
#[derive(Display, Debug)]
#[display(fmt = "Failed to perform a database access operation!")]
pub enum IoError {
    Database(sled::Error),
    Serde(serde_cbor::error::Error),
}

impl Error for IoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IoError::Database(err) => Some(err),
            IoError::Serde(err) => Some(err),
        }
    }
}

impl From<sled::Error> for IoError {
    fn from(err: sled::Error) -> IoError {
        IoError::Database(err)
    }
}

impl From<serde_cbor::error::Error> for IoError {
    fn from(err: serde_cbor::error::Error) -> IoError {
        IoError::Serde(err)
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
