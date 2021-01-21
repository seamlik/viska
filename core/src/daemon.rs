tonic::include_proto!("viska.daemon");

use crate::database::peer::PeerService;
use crate::database::vcard::VcardService;
use crate::database::Chatroom;
use crate::database::Database;
use crate::event::Event;
use crate::event::EventBus;
use async_trait::async_trait;
use diesel::prelude::*;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::TrySendError;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::prelude::*;
use node_client::NodeClient;
use node_server::NodeServer;
use std::error::Error;
use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tonic::transport::NamedService;
use tonic::transport::Server;
use tonic::Code;
use tonic::Status;
use tower::Service;

trait GrpcService<S>
where
    S: Service<http::Request<Body>, Response = http::Response<BoxBody>>
        + NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn Error + Send + Sync>> + Send,
{
    fn spawn_server(service: S, port: u16) {
        // TODO: TLS
        log::info!("{} serving at port {}", std::any::type_name::<S>(), port);
        let task = async move {
            Server::builder()
                .add_service(service)
                .serve(format!("[::1]:{}", port).parse().unwrap())
                .await
                .expect("Failed to spawn gRPC server")
        };
        tokio::spawn(task);
    }
}

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

pub(crate) struct StandardNode {
    event_bus: Arc<EventBus<Event>>,
    database: Arc<Database>,
}

impl<T: node_server::Node> GrpcService<NodeServer<T>> for StandardNode {}

impl StandardNode {
    pub fn start(node_grpc_port: u16, event_bus: Arc<EventBus<Event>>, database: Arc<Database>) {
        let instance = Self {
            event_bus,
            database,
        };
        Self::spawn_server(NodeServer::new(instance), node_grpc_port);
    }

    fn run_query<Q, T>(
        database: Arc<Database>,
        sender: UnboundedSender<Result<T, Status>>,
        query: Q,
    ) -> Result<(), TrySendError<Result<T, Status>>>
    where
        Q: FnOnce(&'_ SqliteConnection) -> QueryResult<T>,
    {
        let connection = database.connection.lock().unwrap();
        let queried = query(&connection).map_err(IntoTonicStatus::into_tonic_status);
        drop(connection);
        sender.unbounded_send(queried)
    }

    fn run_subscription<F, T, Q>(
        &self,
        event_filter: F,
        query: Q,
    ) -> Result<tonic::Response<UnboundedReceiver<Result<T, Status>>>, Status>
    where
        F: FnOnce(&Event) -> bool + Send + Clone + 'static,
        T: Send + 'static,
        Q: FnOnce(&'_ SqliteConnection) -> QueryResult<T> + Send + Clone + 'static,
    {
        let (sender, receiver) = futures::channel::mpsc::unbounded::<Result<T, Status>>();
        let database = self.database.clone();
        let mut subscription = self.event_bus.subscribe();
        tokio::spawn(async move {
            if let Err(_) = Self::run_query(database.clone(), sender.clone(), query.clone()) {
                return;
            }

            while let Some(event) = subscription.next().await {
                let filter = event_filter.clone();
                if filter(&event) {
                    if let Err(_) = Self::run_query(database.clone(), sender.clone(), query.clone())
                    {
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
    type WatchVcardStream = UnboundedReceiver<Result<Vcard, Status>>;

    async fn watch_vcard(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchVcardStream>, Status> {
        let requested_account_id = request.into_inner();
        let requested_account_id_for_filter = requested_account_id.clone();
        self.run_subscription(
            |event| {
                let requested_account_id_for_filter = requested_account_id_for_filter;
                if let Event::Vcard { account_id } = event {
                    account_id == &requested_account_id_for_filter
                } else {
                    false
                }
            },
            move |connection| {
                let requested_account_id = requested_account_id;
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
        todo!()
    }

    type WatchChatroomStream = Receiver<Result<Chatroom, Status>>;

    async fn watch_chatroom(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomStream>, Status> {
        todo!()
    }

    type WatchChatroomsStream = Receiver<Result<ChatroomsSubscription, Status>>;

    async fn watch_chatrooms(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchChatroomsStream>, Status> {
        todo!()
    }

    type WatchRosterStream = UnboundedReceiver<Result<Roster, Status>>;

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
