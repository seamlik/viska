//! Mock profiles.
//!
//! A mock profile is filled with random yet sensible data. This is convenient for testing and
//! application prototyping.

#![cfg(feature = "mock_profiles")]

use crate::database::RawProfile;
use crate::models::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use fake::faker::Chrono;
use fake::faker::Faker;
use fake::faker::Lorem;
use fake::faker::Name;
use sled::Db;
use sled::Tree;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

/// Generates a mock profile.
///
/// Beware that this is a time-consuming operation.
pub fn new_mock_profile(dst: &Path) {
    let num_blacklist = 10;
    let num_devices = 5;
    let num_whitelist = 10;

    let database = Db::start_default(dst).unwrap();

    info!("Issuing account certificate...");
    let (account_cert, account_key) = crate::pki::new_certificate_account().unwrap();
    database
        .set_account_certificate(&account_cert.to_der().unwrap())
        .unwrap();
    database
        .set_account_key(&account_key.private_key_to_der().unwrap())
        .unwrap();
    database
        .add_vcard(&account_cert.id(), &random_vcard())
        .unwrap();

    info!("Issuing device certificates...");
    for _ in 0..num_devices {
        let (device_cert, device_key) =
            crate::pki::new_certificate_device(&account_cert, &account_key).unwrap();
        database
            .set_device_certificate(&device_cert.to_der().unwrap())
            .unwrap();
        database
            .set_device_key(&device_key.private_key_to_der().unwrap())
            .unwrap();
    }

    info!("Generating blacklist...");
    write_vcard_list(&database, PeerList::Blacklist, num_blacklist);

    info!("Generating imaginary friends...");
    write_vcard_list(&database, PeerList::Whitelist, num_whitelist);
}

enum PeerList {
    Blacklist,
    Whitelist,
}

fn write_vcard_list(database: &Tree, list_type: PeerList, num: u8) {
    let list: HashSet<CertificateId> = (0..num).map(|_| random_certificate_id()).collect();

    // List
    match list_type {
        PeerList::Blacklist => {
            database.set_blacklist(&list).unwrap();
        }
        PeerList::Whitelist => {
            database.set_whitelist(&list).unwrap();
        }
    }

    // Vcard
    for id in list {
        database.add_vcard(&id, &random_vcard()).unwrap();
    }
}

fn random_certificate_id() -> CertificateId {
    crate::pki::new_certificate_account().unwrap().0.id()
}

fn random_vcard() -> Vcard {
    Vcard {
        avatar: Vec::new(),
        description: crate::utils::join_strings(Faker::sentences(2).into_iter()),
        devices: HashMap::new(),
        name: Faker::name(),
        time_updated: Faker::datetime(None).parse().unwrap(),
    }
}
