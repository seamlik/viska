//! Mock profiles.
//!
//! A mock profile is filled with random yet sensible data. This is convenient for testing and
//! application prototyping.
//!
//! Feature-gated by `mock_profiles`

#![cfg(feature = "mock_profiles")]

use crate::database::Address;
use crate::database::Chatroom;
use crate::database::Device;
use crate::database::DisplayableId;
use crate::database::Message;
use crate::database::RawDatabase;
use crate::database::Vcard;
use crate::database::DEFAULT_MIME;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use chrono::DateTime;
use chrono::Utc;
use fake::dummy::Dummy;
use fake::faker::Faker;
use fake::faker::Internet;
use fake::faker::Lorem;
use fake::faker::Name;
use rand::seq::IteratorRandom;
use rand::Rng;
use sled::Db;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

/// Generates a mock profile.
///
/// Beware that this is a time-consuming operation.
pub fn new_mock_profile(dst: &Path) {
    let num_blacklist = 10;
    let num_devices = 5;
    let num_whitelist = 10;
    let num_chatrooms = 5;
    let num_messages_min = 20;
    let num_messages_max = 50;

    let mut db_path = PathBuf::from(dst);
    db_path.push("database");

    let database = Db::start_default(&db_path).unwrap();
    let mut rng = rand::thread_rng();

    log::info!("Issuing account certificate...");
    let (account_cert, account_key) = crate::pki::new_certificate_account().unwrap();
    database
        .set_account_certificate(&account_cert.to_der().unwrap())
        .unwrap();
    database
        .set_account_key(&account_key.private_key_to_der().unwrap())
        .unwrap();

    log::info!("Issuing device certificates...");
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

    log::info!("Generating blacklist...");
    write_vcard_list(&database, PeerList::Blacklist, num_blacklist);

    log::info!("Generating imaginary friends...");
    let whitelist = write_vcard_list(&database, PeerList::Whitelist, num_whitelist);

    log::info!("Arranging chatrooms...");
    let chatroom_candidates = whitelist.keys().map(Vec::as_slice).collect();
    for chatroom in random_chatroom(&chatroom_candidates, num_chatrooms) {
        database.add_chatroom(&chatroom).unwrap();

        log::info!(
            "Generating messages for chatroom {}...",
            chatroom.id().display(),
        );
        let members_map: HashMap<&CertificateId, &Vcard> = whitelist
            .iter()
            .filter(|&(id, _)| chatroom.members.contains(id))
            .map(|(id, vcard)| (id.as_slice(), vcard))
            .collect();
        for _ in 0..=rng.gen_range(num_messages_min, num_messages_max) {
            let (head, body) = random_message(&members_map);
            database
                .add_message(&Uuid::new_v4(), head, body, &chatroom.id())
                .unwrap();
        }
    }
}

enum PeerList {
    Blacklist,
    Whitelist,
}

/// Writes whitelist or blacklist and stores their `Vcard`s.
///
/// Returns a map of `CertificateId`s to `Vcard`s.
fn write_vcard_list(
    database: &impl RawDatabase,
    list_type: PeerList,
    num: u8,
) -> HashMap<Vec<u8>, Vcard> {
    let accounts = (0..num).map(|_| random_certificate_id()).collect();

    // Set whitelist or blacklist
    match list_type {
        PeerList::Blacklist => {
            database.set_blacklist(&accounts).unwrap();
        }
        PeerList::Whitelist => {
            database.set_whitelist(&accounts).unwrap();
        }
    }

    // Generating `Vcard`s and return the whole map
    accounts
        .into_iter()
        .map(|id| (id, random_vcard(Option::None)))
        .inspect(|(id, vcard)| database.add_vcard(&id, &vcard).unwrap())
        .collect()
}

fn random_certificate_id() -> Vec<u8> {
    crate::pki::new_certificate_account().unwrap().0.id()
}

fn random_vcard(ids: Option<HashSet<Vec<u8>>>) -> Vcard {
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
            (id, Device { name })
        })
        .collect();

    Vcard {
        devices,
        name: Faker::name(),
        time_updated: DateTime::<Utc>::dummy(),
    }
}

/// Generates `num` of random `Chatroom`s by choosing among `candidates`.
fn random_chatroom<'a>(candidates: &HashSet<&'a CertificateId>, num: usize) -> Vec<Chatroom> {
    let mut rng = rand::thread_rng();
    (0..num)
        .map(|_| {
            let members = candidates
                .iter()
                .filter(|_| rng.gen_bool(0.5))
                .map(|it| it.deref().into())
                .collect();
            Chatroom { members }
        })
        .collect()
}

fn random_message<'a>(participants: &HashMap<&'a CertificateId, &'a Vcard>) -> (Message, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let account: Vec<u8> = participants
        .keys()
        .choose(&mut rng)
        .expect("Empty `participants`!")
        .deref()
        .into();
    let device: Vec<u8> = participants
        .get(&account.as_slice())
        .unwrap()
        .devices
        .keys()
        .choose(&mut rng)
        .expect("Chosen account has no device!")
        .to_owned();

    let head = Message {
        mime: DEFAULT_MIME.clone(),
        sender: Address { account, device },
        time: DateTime::<Utc>::dummy(),
    };

    let body = match rng.gen_range(1, 6) {
        4 => crate::utils::join_strings(Faker::paragraphs(1).into_iter()),
        5 => crate::utils::join_strings(Faker::paragraphs(2).into_iter()),
        n => crate::utils::join_strings(Faker::sentences(n).into_iter()),
    };

    (head, body.into())
}
