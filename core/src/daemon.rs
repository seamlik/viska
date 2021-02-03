tonic::include_proto!("viska.daemon");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use crate::database::vcard::VcardService;
use crate::database::Database;
use crate::database::Event;
use crate::EXECUTOR;
use async_channel::Receiver;
use async_trait::async_trait;
use diesel::prelude::*;
use futures_util::FutureExt;
use futures_util::StreamExt;
use node_client::NodeClient;
use node_server::NodeServer;
use std::any::Any;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tonic::transport::Channel;
use tonic::transport::Server;
use tonic::Code;
use tonic::Status;

#[async_trait]
pub(crate) trait GrpcClient: Sized {
    async fn create(port: u16) -> Result<Self, tonic::transport::Error>;
}

#[async_trait]
impl GrpcClient for NodeClient<Channel> {
    async fn create(port: u16) -> Result<Self, tonic::transport::Error> {
        Self::connect(format!("http://[::1]:{}", port)).await
    }
}

/// Nullable gRPC response.
///
/// If error status is [NotFound](Code::NotFound), the resposne can be seen as [None].
pub trait NullableResponse<T: prost::Message> {
    fn unwrap_response(self) -> Result<Option<T>, Status>;
}

impl<T: prost::Message> NullableResponse<T> for Result<tonic::Response<T>, Status> {
    fn unwrap_response(self) -> Result<Option<T>, Status> {
        match self {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) => {
                if let Code::NotFound = status.code() {
                    Ok(None)
                } else {
                    Err(status)
                }
            }
        }
    }
}

/// The standard implementation of a "Node" gRPC daemon.
pub(crate) struct StandardNode {
    event_stream: Receiver<Arc<Event>>,
    database: Arc<Database>,
}

impl StandardNode {
    /// Starts the gRPC daemon.
    ///
    /// Returns a [Future] to await the shutdown of the gRPC service and a token for shutting down
    /// the service manually. Drop the token to shut it down.
    pub fn start(
        node_grpc_port: u16,
        event_stream: Receiver<Arc<Event>>,
        database: Arc<Database>,
    ) -> (JoinHandle<()>, impl Any + Send + 'static) {
        let instance = Self {
            event_stream,
            database,
        };

        // Shutdown token
        let (sender, receiver) = async_channel::bounded::<()>(1);
        let shutdown_token = receiver
            .collect::<Vec<_>>()
            .map(drop)
            .inspect(move |_| log::info!("Shutting down gRPC daemon at port {}", node_grpc_port));

        // TODO: TLS
        log::info!("gRPC daemon serving at port {}", node_grpc_port);
        let task = Server::builder()
            .add_service(NodeServer::new(instance))
            .serve_with_shutdown(
                format!("[::1]:{}", node_grpc_port).parse().unwrap(),
                shutdown_token,
            )
            .map(|o| o.expect("Failed to spawn gRPC server"));
        (EXECUTOR.spawn(task), sender)
    }

    fn run_query<Q, T>(database: &Database, query: Q) -> Result<T, Status>
    where
        Q: FnOnce(&'_ SqliteConnection) -> QueryResult<T>,
    {
        let connection = database.connection.lock().unwrap();
        query(&connection).map_err(IntoTonicStatus::into_tonic_status)
    }

    fn run_subscription<F, T, Q>(
        &self,
        event_filter: F,
        query: Q,
    ) -> Result<tonic::Response<Receiver<Result<T, Status>>>, Status>
    where
        F: Fn(&Event) -> bool + Send + 'static,
        T: Send + 'static,
        Q: Fn(&'_ SqliteConnection) -> QueryResult<T> + Send + Sync + 'static,
    {
        let (sender, receiver) = async_channel::unbounded::<Result<T, Status>>();
        let mut event_stream = self.event_stream.clone();
        let database = self.database.clone();
        EXECUTOR.spawn(async move {
            if let Err(_) = sender.send(Self::run_query(&database, &query)).await {
                return;
            }

            while let Some(event) = event_stream.next().await {
                if event_filter(&event) {
                    if let Err(_) = sender.send(Self::run_query(&database, &query)).await {
                        return;
                    }
                }
            }
        });

        Ok(tonic::Response::new(receiver))
    }
}

#[async_trait]
impl node_server::Node for StandardNode {
    type WatchVcardStream = Receiver<Result<Vcard, Status>>;

    async fn watch_vcard(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchVcardStream>, Status> {
        let requested_account_id = request.into_inner();
        let requested_account_id_for_filter = requested_account_id.clone();
        self.run_subscription(
            move |event| {
                if let Event::Vcard { account_id } = event {
                    account_id == &requested_account_id_for_filter
                } else {
                    false
                }
            },
            move |connection| {
                VcardService::find_by_account_id(connection, &requested_account_id)
                    .map(Option::unwrap_or_default)
            },
        )
    }

    type WatchChatroomMessagesStream = Receiver<Result<ChatroomMessagesSubscription, Status>>;

    async fn watch_chatroom_messages(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomMessagesStream>, Status> {
        let requested_chatroom_id = request.into_inner();
        let requested_chatroom_id_for_filter = requested_chatroom_id.clone();
        self.run_subscription(
            move |event| {
                if let Event::Message { chatroom_id } = event {
                    chatroom_id == &requested_chatroom_id_for_filter
                } else {
                    false
                }
            },
            move |connection| MessageService::find_by_chatroom(connection, &requested_chatroom_id),
        )
    }

    type WatchChatroomStream = Receiver<Result<Chatroom, Status>>;

    async fn watch_chatroom(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomStream>, Status> {
        let requested_chatroom_id = request.into_inner();
        let requested_chatroom_id_for_filter = requested_chatroom_id.clone();
        self.run_subscription(
            move |event| {
                if let Event::Chatroom { chatroom_id } = event {
                    chatroom_id == &requested_chatroom_id_for_filter
                } else {
                    false
                }
            },
            move |connection| {
                ChatroomService::find_by_id(connection, &requested_chatroom_id)
                    .map(Option::unwrap_or_default)
            },
        )
    }

    type WatchChatroomsStream = Receiver<Result<ChatroomsSubscription, Status>>;

    async fn watch_chatrooms(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchChatroomsStream>, Status> {
        self.run_subscription(
            |event| {
                if let Event::Chatroom { chatroom_id: _ } = event {
                    true
                } else {
                    false
                }
            },
            move |connection| ChatroomService::find_all(connection),
        )
    }

    type WatchRosterStream = Receiver<Result<Roster, Status>>;

    async fn watch_roster(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchRosterStream>, Status> {
        self.run_subscription(
            |event| {
                if let Event::Roster = event {
                    true
                } else {
                    false
                }
            },
            move |connection| PeerService::roster(connection),
        )
    }
}

trait IntoTonicStatus {
    fn into_tonic_status(self) -> Status;
}

impl IntoTonicStatus for diesel::result::Error {
    fn into_tonic_status(self) -> Status {
        match self {
            diesel::result::Error::NotFound => Status::not_found(self.to_string()),
            _ => Status::internal(self.to_string()),
        }
    }
}
