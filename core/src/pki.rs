//! Public key infrastructure for managing certificates of accounts and devices.
//!
//! A user account is essentially an X.509 certificate combined with its private key. Such
//! certificate may issue multiple device certificates. These certificates are used in TLS sessions
//! between clients, thus providing end-to-end encryption and authentication.
//!
//! # Format
//!
//! The format of a certificate are defined by the X.509 version 3 standards with the following specializations:
//!
//! * `subject`: `CN` = `Viska Account` or `Viska Device`
//! * `validity`: Never expire.
//!
//! All decisions on crytographic algorithms in this section are only advisory during certificate creation. A client
//! should be able perform verification based on the built-in information. If a legacy client does not support some of
//! the algorithms, it must notify the user and urge for an immediate update on software.

use blake2::Blake2b;
use blake2::Digest;
use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
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

fn new_x509name_with_one_entry(key: Nid, value: &str) -> Result<X509Name, ErrorStack> {
    let mut builder = X509NameBuilder::new()?;
    builder.append_entry_by_nid(key, value)?;
    Ok(builder.build())
}

fn prepare_new_certificate() -> Result<(X509Builder, PKey<Private>), ErrorStack> {
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
pub fn new_certificate_account() -> Result<(X509, PKey<Private>), ErrorStack> {
    let (mut builder, key) = prepare_new_certificate()?;
    let subject = new_x509name_with_one_entry(Nid::COMMONNAME, "Viska Account")?;

    builder.set_issuer_name(&subject)?;
    builder.set_subject_name(&subject)?;

    builder.sign(&key, new_digest_for_certificate_signatures())?;
    Ok((builder.build(), key))
}

/// Issues a device certificate.
pub fn new_certificate_device<T: HasPrivate>(
    account_cert: &X509Ref,
    account_key: &PKeyRef<T>,
) -> Result<(X509, PKey<Private>), ErrorStack> {
    let (mut builder, key) = prepare_new_certificate()?;
    let subject = new_x509name_with_one_entry(Nid::COMMONNAME, "Viska Device")?;

    builder.set_issuer_name(&account_cert.subject_name())?;
    builder.set_subject_name(&subject)?;

    builder.sign(&account_key, new_digest_for_certificate_signatures())?;
    Ok((builder.build(), key))
}

/// X.509 certificate with extra features.
pub trait Certificate {
    /// Calculates the ID.
    fn id(&self) -> Vec<u8>;
}

impl Certificate for X509 {
    fn id(&self) -> Vec<u8> {
        Blake2b::digest(&self.to_der().unwrap())
            .into_iter()
            .collect()
    }
}

impl Certificate for crate::database::Certificate {
    fn id(&self) -> Vec<u8> {
        Blake2b::digest(&self).into_iter().collect()
    }
}

/// [BLAKE2b](https://blake2.net)-512 digest of the entire certificate encoded in ASN.1 DER.
pub type CertificateId = [u8];
