//! Mock profiles.
//!
//! A mock profile is filled with random yet sensible data. This is convenient for testing and
//! application prototyping.
//!
//! Feature-gated by `mock_profiles`

#![cfg(feature = "mock_profiles")]

use crate::database::Chatroom;
use crate::database::DisplayableId;
use crate::database::MessageHead;
use crate::database::RawOperations;
use crate::database::Vcard;
use crate::database::DEFAULT_MIME;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use chrono::offset::LocalResult;
use chrono::offset::TimeZone;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use fake::faker::lorem::en::Paragraphs;
use fake::faker::lorem::en::Sentences;
use fake::faker::name::en::Name;
use fake::Fake;
use itertools::Itertools;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

/// Generates a mock profile.
///
/// Beware that this is a time-consuming operation.
#[riko::fun]
pub fn new_mock_profile(dst: &String) {
    let num_blacklist = 10;
    let num_whitelist = 10;
    let num_chatrooms = 5;
    let num_messages_min = 20;
    let num_messages_max = 50;

    let mut db_path = PathBuf::from(dst);
    db_path.push("database");

    let database = sled::open(&db_path).unwrap();
    let mut rng = rand::thread_rng();

    log::info!("Generating account certificate...");
    let bundle = crate::pki::new_certificate();
    database.set_certificate(&bundle.certificate).unwrap();
    database.set_key(&bundle.keypair).unwrap();

    let id: CertificateId = bundle.certificate.id().into();
    log::info!("Created account ID: {}", id.display());

    database.add_vcard(&id, &random_vcard()).unwrap();

    log::info!("Generating blacklist...");
    write_vcard_list(&database, PeerList::Blacklist, num_blacklist);

    log::info!("Generating imaginary friends...");
    let whitelist = write_vcard_list(&database, PeerList::Whitelist, num_whitelist);

    log::info!("Arranging chatrooms...");
    let chatroom_candidates = whitelist.keys().collect();
    for chatroom in random_chatroom(&chatroom_candidates, num_chatrooms) {
        database.add_chatroom(&chatroom).unwrap();

        log::info!(
            "Generating messages for chatroom {}...",
            chatroom.id().as_bytes().display(),
        );
        let members_map: HashMap<&CertificateId, &Vcard> = whitelist
            .iter()
            .filter(|&(id, _)| chatroom.members.contains(id))
            .map(|(id, vcard)| (id, vcard))
            .collect();
        for _ in 0..=rng.gen_range(num_messages_min, num_messages_max) {
            let (head, body) = random_message(&members_map);
            database.add_message(&Uuid::new_v4(), head, body).unwrap();
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
    database: &impl RawOperations,
    list_type: PeerList,
    num: u8,
) -> HashMap<CertificateId, Vcard> {
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
        .map(|id| (id, random_vcard()))
        .inspect(|(id, vcard)| database.add_vcard(&id, &vcard).unwrap())
        .collect()
}

fn random_certificate_id() -> CertificateId {
    crate::pki::new_certificate().certificate.id().into()
}

fn random_vcard() -> Vcard {
    Vcard {
        name: Name().fake(),
        time_updated: random_datetime(),
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
                .map(|it| **it)
                .collect();
            Chatroom { members }
        })
        .collect()
}

fn random_message<'a>(
    participants: &HashMap<&'a CertificateId, &'a Vcard>,
) -> (MessageHead, Vec<u8>) {
    let mut rng = rand::thread_rng();

    let account: CertificateId = **participants
        .keys()
        .choose(&mut rng)
        .expect("Empty `participants`!");
    let recipients = participants
        .keys()
        .map(|it| **it)
        .collect::<BTreeSet<CertificateId>>();

    let head = MessageHead {
        mime: DEFAULT_MIME.clone(),
        recipients,
        sender: account,
        time: random_datetime(),
    };

    let body = match rng.gen_range(1, 6) {
        4 => Paragraphs(1..2).fake::<Vec<String>>().into_iter().join(" "),
        5 => Paragraphs(2..3).fake::<Vec<String>>().into_iter().join(" "),
        n => Sentences(1..(n + 1))
            .fake::<Vec<String>>()
            .into_iter()
            .join(" "),
    };

    (head, body.into())
}

fn random_datetime() -> DateTime<Utc> {
    let mut rng = rand::thread_rng();
    let offset = Duration::days(365 * 100).num_seconds();
    loop {
        let time = rng.gen_range(-offset, offset);
        let result = Utc.timestamp_opt(time, 0);
        if let LocalResult::Single(datetime) = result {
            break datetime;
        }
        log::warn!("Invalid time: {}", time)
    }
}
