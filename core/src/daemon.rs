tonic::include_proto!("viska.daemon");

use crate::database::chatroom::ChatroomService;
use crate::database::message::MessageService;
use crate::database::peer::PeerService;
use crate::database::vcard::VcardService;
use crate::database::Database;
use crate::database::Event as DatabaseEvent;
use crate::util::TaskSink;
use crate::EXECUTOR;
use async_trait::async_trait;
use diesel::prelude::*;
use futures_channel::mpsc::UnboundedReceiver as MpscReceiver;
use futures_util::FutureExt;
use node_server::NodeServer;
use std::any::Any;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tonic::transport::Server;
use tonic::Code;
use tonic::Response;
use tonic::Status;

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
    event_sink_database: BroadcastSender<Arc<DatabaseEvent>>,
    event_sink_daemon: BroadcastSender<Arc<Event>>,
    database: Arc<Database>,
    task_sink: TaskSink,
}

impl StandardNode {
    /// Constructor.
    ///
    /// Returns a [Future] to drive the gRPC service and a token for shutting down
    /// the service manually. Drop the token to shut it down.
    pub fn new(
        node_grpc_port: u16,
        event_sink_database: BroadcastSender<Arc<DatabaseEvent>>,
        event_sink_daemon: BroadcastSender<Arc<Event>>,
        database: Arc<Database>,
    ) -> (impl Future<Output = ()>, impl Any + Send + 'static) {
        // Handlers
        let (task_sink, dynamic_task) = TaskSink::new();

        let instance = Self {
            event_sink_database,
            event_sink_daemon,
            database,
            task_sink,
        };

        // Shutdown token
        let (sender, receiver) = tokio::sync::oneshot::channel::<()>();
        let shutdown_token = receiver
            .map(drop)
            .inspect(move |_| log::info!("Shutting down gRPC daemon at port {}", node_grpc_port));

        // gRPC
        // TODO: TLS
        log::info!("gRPC daemon serving at port {}", node_grpc_port);
        let grpc_task = Server::builder()
            .add_service(NodeServer::new(instance))
            .serve_with_shutdown(
                format!("[::1]:{}", node_grpc_port).parse().unwrap(),
                shutdown_token,
            )
            .map(|o| o.expect("Failed to spawn gRPC server"));

        let task = async move {
            let handle = EXECUTOR.spawn(grpc_task);
            dynamic_task.await;
            handle.await.unwrap();
        };

        (task, sender)
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
    ) -> Response<MpscReceiver<Result<T, Status>>>
    where
        F: Fn(&DatabaseEvent) -> bool + Send + 'static,
        T: Send + 'static,
        Q: Fn(&'_ SqliteConnection) -> QueryResult<T> + Send + Sync + 'static,
    {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let mut event_stream = self.event_sink_database.subscribe();
        let database = self.database.clone();
        let task = async move {
            if sender
                .unbounded_send(Self::run_query(&database, &query))
                .is_err()
            {
                return;
            }

            loop {
                match event_stream.recv().await {
                    Ok(event)
                        if event_filter(&event)
                            && sender
                                .unbounded_send(Self::run_query(&database, &query))
                                .is_err() =>
                    {
                        return
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    _ => return,
                }
            }
        };
        self.task_sink.submit(task);
        tonic::Response::new(receiver)
    }
}

#[async_trait]
impl node_server::Node for StandardNode {
    type WatchEventsStream = MpscReceiver<Result<Event, Status>>;

    async fn watch_events(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<Self::WatchEventsStream>, Status> {
        let (sender, receiver) = futures_channel::mpsc::unbounded();
        let mut subscription = self.event_sink_daemon.subscribe();
        let task = async move {
            loop {
                match subscription.recv().await {
                    Ok(event) if sender.unbounded_send(Ok(event.as_ref().clone())).is_err() => {
                        return
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    _ => return,
                }
            }
        };
        self.task_sink.submit(task);
        Ok(Response::new(receiver))
    }

    type WatchVcardStream = MpscReceiver<Result<Vcard, Status>>;

    async fn watch_vcard(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchVcardStream>, Status> {
        let requested_account_id = request.into_inner();
        let requested_account_id_for_filter = requested_account_id.clone();
        let result = self.run_subscription(
            move |event| {
                matches!(event, DatabaseEvent::Vcard { account_id } if account_id == &requested_account_id_for_filter)

            },
            move |connection| {
                VcardService::find_by_account_id(connection, &requested_account_id)
                    .map(Option::unwrap_or_default)
            },
        );
        Ok(result)
    }

    type WatchChatroomMessagesStream = MpscReceiver<Result<ChatroomMessagesSubscription, Status>>;

    async fn watch_chatroom_messages(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomMessagesStream>, Status> {
        let requested_chatroom_id = request.into_inner();
        let requested_chatroom_id_for_filter = requested_chatroom_id.clone();
        let result = self.run_subscription(
            move |event| {
                matches!(event, DatabaseEvent::Message { chatroom_id } if chatroom_id == &requested_chatroom_id_for_filter)
            },
            move |connection| MessageService::find_by_chatroom(connection, &requested_chatroom_id),
        );
        Ok(result)
    }

    type WatchChatroomStream = MpscReceiver<Result<Chatroom, Status>>;

    async fn watch_chatroom(
        &self,
        request: tonic::Request<Vec<u8>>,
    ) -> Result<tonic::Response<Self::WatchChatroomStream>, Status> {
        let requested_chatroom_id = request.into_inner();
        let requested_chatroom_id_for_filter = requested_chatroom_id.clone();
        let result = self.run_subscription(
            move |event| {
                matches!(event, DatabaseEvent::Chatroom { chatroom_id } if chatroom_id == &requested_chatroom_id_for_filter)
            },
            move |connection| {
                ChatroomService::find_by_id(connection, &requested_chatroom_id)
                    .map(Option::unwrap_or_default)
            },
        );
        Ok(result)
    }

    type WatchChatroomsStream = MpscReceiver<Result<ChatroomsSubscription, Status>>;

    async fn watch_chatrooms(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchChatroomsStream>, Status> {
        let result = self.run_subscription(
            |event| matches!(event, DatabaseEvent::Chatroom { chatroom_id: _ }),
            move |connection| ChatroomService::find_all(connection),
        );
        Ok(result)
    }

    type WatchRosterStream = MpscReceiver<Result<Roster, Status>>;

    async fn watch_roster(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::WatchRosterStream>, Status> {
        let result = self.run_subscription(
            |event| matches!(event, DatabaseEvent::Roster),
            move |connection| PeerService::roster(connection),
        );
        Ok(result)
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

#[cfg(test)]
mod test {
    use super::event::Content;
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn watch_events() -> anyhow::Result<()> {
        let (node, _) = crate::util::start_dummy_node().await?;

        let mut client_1 = node.grpc_client().await?;
        let mut stream_1 = client_1.watch_events(()).await?.into_inner();

        let mut client_2 = node.grpc_client().await?;
        let mut stream_2 = client_2.watch_events(()).await?.into_inner();

        let event = Event {
            content: Content::Message(vec![]).into(),
        };
        node.event_sink_daemon.send(event.into())?;

        assert!(matches!(stream_1.next().await, Some(_)));
        assert!(matches!(stream_2.next().await, Some(_)));

        Ok(())
    }
}
