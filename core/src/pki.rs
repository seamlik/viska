//! Public key infrastructure for managing certificates of accounts and devices.
//!
//! A user account is essentially an X.509 certificate combined with its private key. These
//! certificates are used in TLS sessions between clients, thus providing end-to-end encryption and
//! authentication.
//!
//! # Format
//!
//! The format of a certificate are defined by the X.509 version 3 standards with the following specializations:
//!
//! * `subject`: `CN` = `Viska Account`
//! * `validity`: Never expire.
//!
//! All decisions on crytographic algorithms in this section are only advisory during certificate creation. A client
//! should be able perform verification based on the built-in information. If a legacy client does not support some of
//! the algorithms, it must notify the user and urge for an immediate update on software.

use blake3::Hash;
use rcgen::CertificateParams;
use rcgen::DistinguishedName;
use rcgen::DnType;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;

/// Bundle generated when creating a certificate.
#[derive(Deserialize, Serialize)]
pub struct CertificateBundle {
    /// X.509 certificate encoded in DER.
    pub certificate: ByteBuf,

    // Private and (optionally) public key in PKCS#8 encoded in DER.
    pub keypair: ByteBuf,
}

/// Generates a certificate for an account.
#[riko::fun]
pub fn new_certificate() -> crate::pki::CertificateBundle {
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "Viska Account");

    let mut params = CertificateParams::default();
    params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
    params.distinguished_name = dn;

    let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");
    let cert_bytes = cert
        .serialize_der()
        .expect("Failed to serialize certificate into DER");
    let keypair = cert.get_key_pair().serialize_der();
    CertificateBundle {
        certificate: ByteBuf::from(cert_bytes),
        keypair: ByteBuf::from(keypair),
    }
}

/// X.509 certificate with extra features.
pub trait Certificate {
    /// Calculates its ID.
    fn id(&self) -> CertificateId;
}

impl Certificate for [u8] {
    fn id(&self) -> CertificateId {
        blake3::hash(self)
    }
}

impl Certificate for rustls::Certificate {
    fn id(&self) -> CertificateId {
        blake3::hash(self.as_ref())
    }
}

/// BLAKE3 digest of the entire certificate encoded in ASN.1 DER.
pub type CertificateId = Hash;

/// THE hash function (BLAKE3) universally used in the project.
///
/// This is exported only because this algorithm isn't available in most languages or platform at moment.
#[riko::fun]
pub fn hash(src: &ByteBuf) -> ByteBuf {
    let raw_hash: [u8; 32] = blake3::hash(src).into();
    ByteBuf::from(raw_hash)
}
