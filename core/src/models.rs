//! Data models.

use crate::pki::CertificateId;
use crate::Address;
use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashSet;

/// UUID version 4.
pub type MessageId = [u8; 16];

/// Blake2b-512
pub type ChatroomId = Vec<u8>;

pub struct Message {
    pub id: MessageId,
    pub body: Vec<u8>,
    pub sender: Address,
    pub time_sent: DateTime<Utc>,

    /// Defaults to `text/plain`.
    pub type_mime: Option<String>,
}

pub struct Chatroom {
    /// Self account is excluded.
    pub participants: HashSet<CertificateId>,
}

pub fn chatroom_id_from_participants(participants: &[CertificateId]) -> ChatroomId {
    unimplemented!()
}

pub struct Vcard {
    pub account: CertificateId,
    pub avatar: Vec<u8>,
    pub description: String,
    pub devices: HashSet<DeviceInfo>,
    pub name: String,
    pub time_updated: DateTime<Utc>,
}

pub struct DeviceInfo {
    pub name: String,
    pub id: CertificateId,
}

/// X.509 certificate encoded in ASN.1 DER.
pub type Certificate = Vec<u8>;

/// RFC 5958 PKCS #8 encoded in ASN.1 DER.
pub type CryptoKey = Vec<u8>;

pub struct ProfileInfo {
    pub account_certificate: Certificate,
    pub account_key: CryptoKey,
    pub blacklist: HashSet<CertificateId>,
    pub device_certificate: Certificate,
    pub device_key: CryptoKey,
}
