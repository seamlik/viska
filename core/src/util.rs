//! Utilities.

use crate::database::ProfileConfig;
use crate::Node;
use crate::EXECUTOR;
use futures_channel::mpsc::UnboundedSender;
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use futures_util::StreamExt;
use rand::prelude::*;
use std::future::Future;

/// Configures to start a [Node] that does nothing.
pub async fn start_dummy_node() -> anyhow::Result<(Node, impl Future<Output = ()>)> {
    // TODO: In-memory database
    let tmp_dir = tempfile::tempdir()?;
    let account_id = crate::database::create_standard_profile(tmp_dir.path().to_path_buf()).await?;
    let profile_config = ProfileConfig {
        dir_data: tmp_dir.path().to_path_buf(),
    };
    let node_grpc_port = random_port();

    let (node, task) = Node::new(&account_id, &profile_config, node_grpc_port).await?;
    let handle = EXECUTOR.spawn(task);
    let task = async move { handle.await.unwrap() };
    Ok((node, task))
}

/// Generates a random port within the private range untouched by IANA.
fn random_port() -> u16 {
    thread_rng().gen_range(49152..u16::MAX)
}

#[derive(Clone)]
pub(crate) struct TaskSink {
    sink: UnboundedSender<BoxFuture<'static, ()>>,
}

impl TaskSink {
    pub fn new() -> (Self, impl Future<Output = ()>) {
        let (sink, receiver) = futures_channel::mpsc::unbounded();
        let task = receiver.for_each_concurrent(None, |o| o);
        let instance = Self { sink };
        (instance, task)
    }

    pub fn submit<F>(&self, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let _ = self.sink.unbounded_send(task.boxed());
    }
}
