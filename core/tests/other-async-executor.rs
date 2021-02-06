use async_executor::Executor;
use futures_executor::ThreadPool;
use futures_util::task::SpawnExt;

#[test]
fn run_on_futures_executor() {
    let executor = ThreadPool::new().unwrap();
    let task = async {
        let (node, task) = viska::util::start_dummy_node().await.unwrap();
        let handle = executor.spawn_with_handle(task).unwrap();
        drop(node);
        handle.await;
    };
    futures_executor::block_on(task)
}

#[test]
fn run_on_async_executor() {
    let executor = Executor::default();
    let task = async {
        let (node, task) = viska::util::start_dummy_node().await.unwrap();
        let handle = executor.spawn(task);
        drop(node);
        handle.await;
    };
    futures_executor::block_on(task)
}
