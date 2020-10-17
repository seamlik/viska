tonic::include_proto!("viska.changelog");

use crate::daemon::platform_client::PlatformClient;
use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use crate::database::TransactionPayload;
use blake3::Hash;
use blake3::Hasher;
use changelog_payload::Content;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Status;

type MessageId = Hash;

impl Message {
    /// Calculates the ID of a [Message].
    pub fn message_id(&self) -> MessageId {
        let mut hasher = Hasher::default();

        hasher.update(&self.sender);
        for account in self.recipients.iter() {
            hasher.update(&account);
        }

        hasher.update(self.time.to_be_bytes().as_ref());
        hasher.update(self.content.as_bytes());

        if let Some(attachment) = &self.attachment {
            hasher.update(attachment.mime.as_bytes());
            hasher.update(&attachment.content);
        }

        hasher.finalize()
    }

    pub fn chatroom_id(&self) -> ChatroomId {
        let recipients = self.recipients.iter().cloned();
        let members = recipients.chain(std::iter::once(self.sender.clone()));
        chatroom_id(members)
    }
}

pub type ChatroomId = Hash;

impl Chatroom {
    pub fn id(&self) -> ChatroomId {
        chatroom_id(self.members.iter().cloned())
    }
}

pub fn chatroom_id(members: impl Iterator<Item = Vec<u8>>) -> ChatroomId {
    let members_sorted: BTreeSet<Vec<u8>> = members.collect();
    let mut hasher = Hasher::default();
    for id in members_sorted {
        hasher.update(id.as_slice());
    }
    hasher.finalize()
}

pub struct ChangelogMerger {
    platform: Arc<Mutex<PlatformClient<Channel>>>,
    chatroom_service: Arc<ChatroomService>,
    peer_service: PeerService,
    message_service: MessageService,
}

impl ChangelogMerger {
    pub fn new(platform: Arc<Mutex<PlatformClient<Channel>>>) -> Self {
        let chatroom_service = Arc::new(ChatroomService {
            platform: platform.clone(),
        });
        Self {
            chatroom_service: chatroom_service.clone(),
            peer_service: PeerService {
                platform: platform.clone(),
            },
            message_service: MessageService {
                platform: platform.clone(),
                chatroom_service,
            },
            platform,
        }
    }

    pub async fn commit(
        &self,
        payloads: impl Iterator<Item = ChangelogPayload>,
    ) -> Result<Vec<TransactionPayload>, Status> {
        let mut transaction = Vec::<TransactionPayload>::new();
        for payload in payloads {
            match payload.content {
                Some(Content::AddChatroom(chatroom)) => {
                    transaction.extend(self.chatroom_service.update(chatroom).await?);
                }
                Some(Content::AddPeer(peer)) => {
                    transaction.extend(self.peer_service.update(peer).await?);
                }
                Some(Content::AddMessage(message)) => {
                    transaction.extend(self.message_service.update(message).await?);
                }
                None => return Err(Status::invalid_argument("Empty transaction payload")),
                _ => todo!(),
            }
        }
        Ok(transaction)
    }
}
