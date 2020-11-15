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
use blake3::Hasher;
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
    pub key: ByteBuf,
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
        key: ByteBuf::from(keypair),
    }
}

/// Data structures that can produce a canonical ID.
///
/// This ID is used to uniquely identify important data structures in the project. It must be
/// reproducible and depends only on the necessary child data of such a data structure.
pub trait CanonicalId {
    /// Calculates its ID.
    fn canonical_id(&self) -> Hash;
}

/// Canonical ID of a X.509 certificate encoded in PKCS#12 ASN.1 DER.
impl CanonicalId for [u8] {
    fn canonical_id(&self) -> Hash {
        let mut hasher = Hasher::default();
        hasher.update(b"Viska application/pkcs12");
        hasher.update(&self.len().to_be_bytes());
        hasher.update(&self);
        hasher.finalize()
    }
}

impl CanonicalId for rustls::Certificate {
    fn canonical_id(&self) -> Hash {
        self.as_ref().canonical_id()
    }
}
