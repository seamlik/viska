use crate::changelog::ChangelogMerger;
use crate::changelog::ChangelogPayload;
use crate::changelog::Message;
use crate::changelog::Peer;
use crate::changelog::PeerRole;
use crate::changelog::Vcard;
use crate::database::vcard::VcardService;
use crate::database::Database;
use chrono::prelude::*;
use chrono::Duration;
use chrono::LocalResult;
use diesel::prelude::*;
use fake::faker::lorem::en::Paragraphs;
use fake::faker::lorem::en::Sentences;
use fake::faker::name::en::Name;
use fake::Fake;
use itertools::Itertools;
use rand::prelude::*;
use std::sync::Arc;

pub(crate) struct MockProfileService {
    pub database: Arc<Database>,
    pub account_id: Vec<u8>,
    pub changelog_merger: Arc<ChangelogMerger>,
}

impl MockProfileService {
    #[cfg(not(debug_assertions))]
    pub fn populate_mock_data(&self) -> QueryResult<()> {
        unimplemented!("Only in debug mode")
    }

    #[cfg(debug_assertions)]
    pub fn populate_mock_data(&self) -> QueryResult<()> {
        let (vcards, changelog) = crate::mock_profile::populate_data(&self.account_id);
        log::info!(
            "Generated {} entries of vCard and {} entries of changelog",
            vcards.len(),
            changelog.len()
        );
        let connection = self.database.connection.lock().unwrap();
        connection.transaction::<_, diesel::result::Error, _>(|| {
            log::info!("Committing the mock Vcards as a transaction");
            VcardService::save(&connection, vcards.into_iter())?;

            log::info!("Merging changelog generated from `mock_profile`");
            self.changelog_merger
                .commit(&connection, changelog.into_iter())?;

            Ok(())
        })
    }
}

pub fn populate_data(account_id: &[u8]) -> (Vec<Vcard>, Vec<ChangelogPayload>) {
    let num_friends = 16;
    let num_messages = 128;

    let mut changelog = Vec::<ChangelogPayload>::default();

    log::info!("Generating Vcard");
    let vcards: Vec<Vcard> = (0..num_friends).map(|_| random_vcard()).collect();

    log::info!("Generating friends");
    changelog.extend(
        vcards
            .iter()
            .map(self::friend_from_vcard)
            .map(|peer| ChangelogPayload {
                content: Some(crate::changelog::changelog_payload::Content::AddPeer(peer)),
            }),
    );

    log::info!("Generating Messages");
    changelog.extend(
        (0..num_messages)
            .map(|_| random_messages(account_id, &vcards))
            .into_iter()
            .map(|message| ChangelogPayload {
                content: Some(crate::changelog::changelog_payload::Content::AddMessage(
                    message,
                )),
            }),
    );

    (vcards, changelog)
}

fn random_messages(account_id: &[u8], friends: &[Vcard]) -> Message {
    let mut rng = thread_rng();

    let content = match rng.gen_range(1..6) {
        4 => Paragraphs(1..2).fake::<Vec<String>>().into_iter().join(" "),
        5 => Paragraphs(2..3).fake::<Vec<String>>().into_iter().join(" "),
        n => Sentences(1..(n + 1))
            .fake::<Vec<String>>()
            .into_iter()
            .join(" "),
    };

    let mut chatroom_members: Vec<&[u8]> = friends
        .iter()
        .map(|vcard| vcard.account_id.as_slice())
        .collect();
    chatroom_members.push(account_id);

    let num_recipients = rng.gen_range(2..5);
    let recipients: Vec<Vec<u8>> = chatroom_members
        .choose_multiple(&mut rng, num_recipients)
        .map(|o| o.to_vec())
        .collect();

    Message {
        time: random_time(),
        sender: chatroom_members
            .choose(&mut rng)
            .unwrap()
            .to_owned()
            .to_owned(),
        recipients,
        content,
        attachment: None,
    }
}

fn friend_from_vcard(vcard: &Vcard) -> Peer {
    let mut peer = Peer {
        account_id: vcard.account_id.clone(),
        ..Default::default()
    };
    peer.set_role(PeerRole::Friend);
    peer
}

fn random_vcard() -> Vcard {
    Vcard {
        account_id: random_account_id(),
        name: Name().fake(),
        photo: None,
    }
}

fn random_account_id() -> Vec<u8> {
    (0..32_u8).map(|_| thread_rng().gen()).collect()
}

fn random_time() -> f64 {
    let mut rng = rand::thread_rng();
    let offset = Duration::days(365 * 100).num_seconds();
    loop {
        let time = rng.gen_range(-offset..offset);
        let result = Utc.timestamp_opt(time, 0);
        if let LocalResult::Single(datetime) = result {
            break crate::database::float_from_time(datetime);
        }
    }
}
