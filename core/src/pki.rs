//! Public key infrastructure for managing certificates of accounts and devices.
//!
//! A user account is essentially an X.509 certificate combined with its pricate key.  Such certificate may issue
//! multiple device certificates in order to identify devices this user has logged in. Therefor, this module is one of
//! the most critical part of the project.
//!
//! Certificates are used in various places, which include but not limited to:
//!
//! * TLS sessions between devices.
//! * Peer identity during peer discovery and user interaction.
//!
//! # Format
//!
//! The format of a certificate are defined by the X.509 version 3 standards with the following specializations:
//!
//! * `serialNumber`: [RFC 4122](https://datatracker.ietf.org/doc/rfc4122) UUID v4.
//! * `signatureAlgorithm`: SHA-256 + RSA.
//! * `subjectPublicKeyInfo`: RSA 4096-bit
//! * `subject`: `CN` = `Viska Account` or `Viska Device`
//! * `validity`: Never expire.
//!
//! All decisions on crytographic algorithms in this section are only advisory during certificate creation. A client
//! should be able perform verification based on the built-in information. If a legacy client does not support some of
//! the algorithms, it must notify the user and urge for an immediate update on software.
//!
//! # Errors
//!
//! Since this module uses `openssl` crate to perform most cryptographic operations, an `openssl::error::ErrorStack`
//! will most likely be the immediate cause when an `Error` is returned.

use crate::Address;
use failure::Error;
use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::HasPrivate;
use openssl::pkey::PKey;
use openssl::pkey::PKeyRef;
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use openssl::x509::X509Builder;
use openssl::x509::X509Name;
use openssl::x509::X509NameBuilder;
use openssl::x509::X509Ref;
use openssl::x509::X509;
use uuid::Uuid;

const LENGTH_KEY: u32 = 4096;

/// Specifies how long before a certificate expires.
///
/// Current version of `libopenssl` crate can't construct an arbitrary `Asn1Time` from a string, so
/// a thousand years should do for now.
const DAYS_VALIDITY: u32 = 300_000;

/// Version 3 of X.509 specification, zero-indexed.
const VERSION_X509: i32 = 2;

fn new_digest_for_certificate_signatures() -> MessageDigest {
    MessageDigest::sha256()
}

fn new_x509name_with_one_entry(key: &str, value: &str) -> Result<X509Name, Error> {
    let mut builder = X509NameBuilder::new()?;
    builder.append_entry_by_text(key, value)?;
    Ok(builder.build())
}

fn prepare_new_certificate() -> Result<(X509Builder, PKey<Private>), Error> {
    let mut builder = X509Builder::new()?;

    let key = PKey::from_rsa(Rsa::generate(LENGTH_KEY)?)?;
    let not_after = Asn1Time::days_from_now(DAYS_VALIDITY)?;
    let not_before = Asn1Time::days_from_now(0)?;
    let serial = BigNum::from_slice(Uuid::new_v4().as_bytes())?.to_asn1_integer()?;

    builder.set_not_after(&not_after)?;
    builder.set_not_before(&not_before)?;
    builder.set_pubkey(&key)?;
    builder.set_serial_number(&serial)?;
    builder.set_version(VERSION_X509)?;

    Ok((builder, key))
}

/// Generates a certificate for an account.
pub fn new_certificate_account() -> Result<(X509, PKey<Private>), Error> {
    let (mut builder, key) = prepare_new_certificate()?;
    let subject = new_x509name_with_one_entry("CN", "Viska Account")?;

    builder.set_issuer_name(&subject)?;
    builder.set_subject_name(&subject)?;

    builder.sign(&key, new_digest_for_certificate_signatures())?;
    Ok((builder.build(), key))
}

/// Issues a device certificate.
pub fn new_certificate_device<T: HasPrivate>(
    account_cert: &X509Ref,
    account_key: &PKeyRef<T>,
) -> Result<(X509, PKey<Private>), Error> {
    let (mut builder, key) = prepare_new_certificate()?;
    let subject = new_x509name_with_one_entry("CN", "Viska Device")?;

    builder.set_issuer_name(&account_cert.subject_name())?;
    builder.set_subject_name(&subject)?;

    builder.sign(&account_key, new_digest_for_certificate_signatures())?;
    Ok((builder.build(), key))
}

pub fn verify_certificate_chain(
    addr: &Address,
    cert_account: &X509,
    cert_device: &X509,
) -> Result<bool, Error> {
    panic!()
}

/// X.509 certificate with extra features.
pub trait Certificate {
    /// Calculates the ID.
    ///
    /// This is the [Multihash](https://multiformats.io/multihash) value of the SHA-256 hash of the entire certificate.
    /// The certificate is encoded in ASN.1 DER format when being hashed.
    ///
    /// # Errors
    ///
    /// * `multihash::Error`
    /// * `openssl::error::ErrorStack`
    fn id(&self) -> Result<Vec<u8>, Error>;
    fn kind(&self) -> CertificateKind;
    fn verify_signer(&self, certificate: &X509) -> bool;
}

impl Certificate for X509 {
    fn kind(&self) -> CertificateKind {
        panic!()
    }
    fn verify_signer(&self, issuer: &X509) -> bool {
        panic!()
    }
    fn id(&self) -> Result<Vec<u8>, Error> {
        multihash::encode(multihash::Hash::SHA2256, &self.to_der()?).map_err(Into::into)
    }
}

/// Indicates whether a certificate is for an account or a device.
///
/// An account certificate must be self-signed, while a device certificate is signed by an account certificate.
pub enum CertificateKind {
    /// Account certificate.
    ///
    /// This kind of certificates must be self-signed.
    Account,

    /// Account certificate.
    ///
    /// This kind of certificates must be signed by an account certificate.
    Device,

    /// Unable to determine the nature of a certificate.
    ///
    /// This variant is reserved for future changes to how to define the type of a certificate.
    Unknown,
}
