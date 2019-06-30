//! Database operations.
//!
//! Only key-value store is supported.

use crate::models::Certificate;
use crate::models::Chatroom;
use crate::models::CryptoKey;
use crate::models::Vcard;
use crate::pki::CertificateId;
use crate::utils::Result;
use sled::IVec;
use sled::Tree;
use std::collections::HashSet;
use std::ops::Deref;

const TABLE_CHATROOMS: &str = "chatrooms";
const TABLE_MESSAGE_BODIES: &str = "message-bodies";
const TABLE_MESSAGE_HEADS: &str = "message-heads";
const TABLE_PROFILE: &str = "profile";
const TABLE_VCARDS: &str = "vcards";

/// Makes a key with a table.
fn tabled_key(table: &str, key: &str) -> String {
    format!("{}/{}", table, key)
}

/// Low-level operations for accessing a profile stored in a database.
pub trait RawProfile {
    fn account_certificate(&self) -> Result<Option<Certificate>>;
    fn account_key(&self) -> Result<Option<CryptoKey>>;
    fn add_chatroom(&self, chatroom: &Chatroom) -> Result<()>;
    fn add_vcard(&self, id: &CertificateId, vcard: &Vcard) -> Result<()>;
    fn blacklist(&self) -> Result<HashSet<CertificateId>>;
    fn device_certificate(&self) -> Result<Option<Certificate>>;
    fn device_key(&self) -> Result<Option<CryptoKey>>;
    fn set_account_certificate(&self, cert: &Certificate) -> Result<()>;
    fn set_account_key(&self, key: &[u8]) -> Result<()>;
    fn set_blacklist(&self, blacklist: &HashSet<CertificateId>) -> Result<()>;
    fn set_device_certificate(&self, cert: &Certificate) -> Result<()>;
    fn set_device_key(&self, key: &[u8]) -> Result<()>;
    fn set_whitelist(&self, blacklist: &HashSet<CertificateId>) -> Result<()>;
    fn whitelist(&self) -> Result<HashSet<CertificateId>>;
}

impl RawProfile for Tree {
    fn set_account_certificate(&self, cert: &Certificate) -> Result<()> {
        self.set(
            tabled_key(TABLE_PROFILE, "account-certificate"),
            cert.deref(),
        )?;
        Ok(())
    }
    fn set_account_key(&self, key: &[u8]) -> Result<()> {
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
    fn set_device_key(&self, key: &[u8]) -> Result<()> {
        self.set(tabled_key(TABLE_PROFILE, "device-key"), key)?;
        Ok(())
    }
    fn account_certificate(&self) -> Result<Option<Certificate>> {
        self.get(tabled_key(TABLE_PROFILE, "account-certificate"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn account_key(&self) -> Result<Option<CryptoKey>> {
        self.get(tabled_key(TABLE_PROFILE, "account-key"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_certificate(&self) -> Result<Option<Certificate>> {
        self.get(tabled_key(TABLE_PROFILE, "device-certificate"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn device_key(&self) -> Result<Option<CryptoKey>> {
        self.get(tabled_key(TABLE_PROFILE, "device-key"))
            .map(IntoBytes::into)
            .map_err(|e| e.into())
    }
    fn blacklist(&self) -> Result<HashSet<CertificateId>> {
        let raw = self.get(tabled_key(TABLE_PROFILE, "blacklist"))?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_blacklist(&self, blacklist: &HashSet<CertificateId>) -> Result<()> {
        let cbor = serde_cbor::to_vec(blacklist).unwrap();
        self.set(tabled_key(TABLE_PROFILE, "blacklist"), cbor)?;
        Ok(())
    }
    fn whitelist(&self) -> Result<HashSet<CertificateId>> {
        let raw = self.get(tabled_key(TABLE_PROFILE, "whitelist"))?;
        match raw {
            None => Ok(HashSet::default()),
            Some(ref raw) if raw.is_empty() => Ok(HashSet::default()),
            Some(raw) => serde_cbor::from_slice(&raw).map_err(|e| e.into()),
        }
    }
    fn set_whitelist(&self, whitelist: &HashSet<CertificateId>) -> Result<()> {
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
        let chatroom_id = crate::utils::display_id(&crate::models::chatroom_id_from_members(
            chatroom.members.iter(),
        ));
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
