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
//! * `issuer`: `CN = "Viska Account"` (informative)
//! * `serialNumber`: [RFC 4122](https://datatracker.ietf.org/doc/rfc4122) UUID v4.
//! * `signature` & `signatureAlgorithm`: SHA-256 + RSA.
//! * `subjectPublicKeyInfo`: RSA 4096-bit
//! * `subject`: `CN = "Viska Device"` (informative)
//! * `validity`: Accounts may never expire, devices should expire in 30 days by default.
//!
//! Additionally, the ID of a certificate is the SHA-256 hash value of the entire certificate encoded in ANS.1 DER
//! format. It must be displayed in [Multihash](https://multiformats.io/multihash) format.
//!
//! All decisions on crytographic algorithms in this section are only mandatory during certificate creation. A client
//! should be able perform verifications based on the built-in information. If a legacy client does not support some of
//! the algorithms, it must notify the user and urge for an immediate update on software.

use crate::Address;
use failure::Error;
use openssl::pkey::HasPrivate;
use openssl::pkey::PKey;
use openssl::pkey::PKeyRef;
use openssl::x509::X509;

pub fn new_certificate_account<T: HasPrivate>() -> Result<(X509, PKey<T>), Error> {
    panic!();
}

pub fn new_certificate_device<T: HasPrivate>(
    account_cert: &X509,
    account_key: &PKeyRef<T>,
) -> Result<(X509, PKey<T>), Error> {
    panic!()
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
    fn id(&self) -> Result<Vec<u8>, Error>;
    fn certificate_type(&self) -> CertificateType;
    fn verify_signer(&self, certificate: &X509) -> bool;
}

impl Certificate for X509 {
    fn certificate_type(&self) -> CertificateType {
        panic!()
    }
    fn verify_signer(&self, issuer: &X509) -> bool {
        panic!()
    }
    fn id(&self) -> Result<Vec<u8>, Error> {
        multihash::encode(multihash::Hash::SHA2256, &self.to_der()?).map_err(Into::<Error>::into)
    }
}

/// Indicates whether a certificate is for an account or a device.
///
/// An account certificate must be self-signed, while a device certificate is signed by an account certificate.
pub enum CertificateType {
    /// Account certificate.
    ///
    /// This type of certificates must be self-signed.
    Account,

    /// Account certificate.
    ///
    /// This type of certificates must be signed by an account certificate.
    Device,

    /// Unable to determine the nature of a certificate.
    ///
    /// This variant is reserved for future changes to how to define the type of a certificate.
    Unknown,
}
