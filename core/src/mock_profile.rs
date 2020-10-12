use crate::changelog::ChangelogPayload;
use crate::changelog::Message;
use crate::changelog::Peer;
use crate::changelog::PeerRole;
use crate::database::TransactionPayload;
use crate::database::Vcard;
use chrono::prelude::*;
use chrono::Duration;
use chrono::LocalResult;
use fake::faker::lorem::en::Paragraphs;
use fake::faker::lorem::en::Sentences;
use fake::faker::name::en::Name;
use fake::Fake;
use itertools::Itertools;
use rand::prelude::*;

#[cfg(not(debug_assertions))]
pub fn populate_data(account_id: &Vec<u8>) -> (Vec<TransactionPayload>, Vec<ChangelogPayload>) {
    Default::default()
}

#[cfg(debug_assertions)]
pub fn populate_data(account_id: &Vec<u8>) -> (Vec<TransactionPayload>, Vec<ChangelogPayload>) {
    let num_friends = 16;
    let num_messages = 128;

    let mut changelog = Vec::<ChangelogPayload>::default();

    let vcards: Vec<Vcard> = (0..num_friends).map(|_| random_vcard()).collect();

    // Populate Friends
    changelog.extend(
        vcards
            .iter()
            .map(self::friend_from_vcard)
            .map(|peer| ChangelogPayload {
                content: Some(crate::changelog::changelog_payload::Content::AddPeer(peer)),
            }),
    );

    // Populate Messages
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

    (
        vcards
            .into_iter()
            .map(|vcard| TransactionPayload {
                content: Some(crate::database::transaction_payload::Content::AddVcard(
                    vcard,
                )),
            })
            .collect(),
        changelog,
    )
}

fn random_messages(account_id: &Vec<u8>, friends: &[Vcard]) -> Message {
    let mut rng = thread_rng();

    let content = match rng.gen_range(1, 6) {
        4 => Paragraphs(1..2).fake::<Vec<String>>().into_iter().join(" "),
        5 => Paragraphs(2..3).fake::<Vec<String>>().into_iter().join(" "),
        n => Sentences(1..(n + 1))
            .fake::<Vec<String>>()
            .into_iter()
            .join(" "),
    };

    let mut chatroom_members: Vec<&Vec<u8>> =
        friends.iter().map(|vcard| &vcard.account_id).collect();
    chatroom_members.push(account_id);

    let num_recipients = rng.gen_range(2, 5);
    let recipients: Vec<_> = chatroom_members
        .choose_multiple(&mut rng, num_recipients)
        .cloned()
        .cloned()
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
        time_updated: random_time(),
    }
}

fn random_account_id() -> Vec<u8> {
    (0..32_u8).map(|_| thread_rng().gen()).collect()
}

fn random_time() -> f64 {
    let mut rng = rand::thread_rng();
    let offset = Duration::days(365 * 100).num_seconds();
    loop {
        let time = rng.gen_range(-offset, offset);
        let result = Utc.timestamp_opt(time, 0);
        if let LocalResult::Single(datetime) = result {
            break crate::database::float_from_time(datetime);
        }
    }
}
