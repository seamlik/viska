//! Mock profiles.
//!
//! A mock profile is filled with random yet sensible data. This is convenient for testing and
//! application prototyping.
//!
//! Feature-gated by `mock-profiles`

#![cfg(feature = "mock-profiles")]

use crate::database::RawProfile;
use crate::models::Chatroom;
use crate::models::DeviceInfo;
use crate::models::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use fake::faker::Chrono;
use fake::faker::Faker;
use fake::faker::Internet;
use fake::faker::Lorem;
use fake::faker::Name;
use rand::Rng;
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
    let num_chatrooms = 5;

    let database = Db::start_default(dst).unwrap();

    info!("Issuing account certificate...");
    let (account_cert, account_key) = crate::pki::new_certificate_account().unwrap();
    database
        .set_account_certificate(&account_cert.to_der().unwrap())
        .unwrap();
    database
        .set_account_key(&account_key.private_key_to_der().unwrap())
        .unwrap();

    info!("Issuing device certificates...");
    let mut device_ids = HashSet::default();
    for _ in 0..num_devices {
        let (device_cert, device_key) =
            crate::pki::new_certificate_device(&account_cert, &account_key).unwrap();
        device_ids.insert(device_cert.id());
        database
            .set_device_certificate(&device_cert.to_der().unwrap())
            .unwrap();
        database
            .set_device_key(&device_key.private_key_to_der().unwrap())
            .unwrap();
    }
    database
        .add_vcard(&account_cert.id(), &random_vcard(Some(device_ids)))
        .unwrap();

    info!("Generating blacklist...");
    write_vcard_list(&database, PeerList::Blacklist, num_blacklist);

    info!("Generating imaginary friends...");
    let whitelist_ids = write_vcard_list(&database, PeerList::Whitelist, num_whitelist);

    info!("Arranging chatrooms...");
    for chatroom in random_chatroom(whitelist_ids.into_iter(), num_chatrooms) {
        database.add_chatroom(&chatroom).unwrap();
    }
}

enum PeerList {
    Blacklist,
    Whitelist,
}

fn write_vcard_list(database: &Tree, list_type: PeerList, num: u8) -> HashSet<CertificateId> {
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
    for id in &list {
        database
            .add_vcard(&id, &random_vcard(Option::None))
            .unwrap();
    }

    list
}

fn random_certificate_id() -> CertificateId {
    crate::pki::new_certificate_account().unwrap().0.id()
}

fn random_vcard(ids: Option<HashSet<CertificateId>>) -> Vcard {
    let num_devices_default = 2;
    let ids_nonnull = match ids {
        None => (0..num_devices_default)
            .map(|_| random_certificate_id())
            .collect(),
        Some(it) => it,
    };
    let devices = ids_nonnull
        .into_iter()
        .map(|id| {
            let name = Faker::user_agent().to_owned();
            (id, DeviceInfo { name: name })
        })
        .collect();

    Vcard {
        avatar: Vec::new(),
        description: crate::utils::join_strings(Faker::sentences(2).into_iter()),
        devices: devices,
        name: Faker::name(),
        time_updated: Faker::datetime(None).parse().unwrap(),
    }
}

fn random_chatroom(
    candidates: impl ExactSizeIterator<Item = CertificateId>,
    num: usize,
) -> Vec<Chatroom> {
    let mut candidates_sorted: Vec<CertificateId> = candidates.collect();
    candidates_sorted.sort();
    candidates_sorted.dedup();

    let mut rng = rand::thread_rng();

    (0..num)
        .map(|_| {
            let members: HashSet<CertificateId> = candidates_sorted
                .iter()
                .filter(|_| rng.gen_bool(0.5))
                .map(|id| id.clone())
                .collect();
            Chatroom { members: members }
        })
        .collect()
}
