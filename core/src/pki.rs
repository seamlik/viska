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

pub struct CertificateBundle {
    pub certificate: Vec<u8>,
    pub keypair: Vec<u8>,
}

/// Generates a certificate for an account.
pub fn new_certificate() -> CertificateBundle {
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "Viska Account");

    let mut params = CertificateParams::default();
    params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
    params.distinguished_name = dn;

    let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");
    let keypair = cert.get_key_pair().serialize_der();
    CertificateBundle {
        certificate: cert
            .serialize_der()
            .expect("Failed to serialize certificate into DER"),
        keypair,
    }
}

/// X.509 certificate with extra features.
pub trait Certificate {
    /// Calculates its ID.
    ///
    /// # See
    ///
    /// * [CertificateId]
    fn id(&self) -> Hash;
}

impl Certificate for Vec<u8> {
    fn id(&self) -> Hash {
        blake3::hash(self)
    }
}

impl Certificate for [u8] {
    fn id(&self) -> Hash {
        blake3::hash(self)
    }
}

/// BLAKE3 digest of the entire certificate encoded in ASN.1 DER.
pub type CertificateId = [u8; 32];
