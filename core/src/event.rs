use futures::channel::mpsc::UnboundedSender;
use futures::prelude::*;
use std::collections::LinkedList;
use std::sync::Arc;
use std::sync::Mutex;

pub struct EventBus<T> {
    sink: UnboundedSender<T>,
    subscribers: Arc<Mutex<LinkedList<UnboundedSender<Arc<T>>>>>,
}

impl<T> EventBus<T> {
    pub fn sink(&self) -> impl Sink<T> {
        self.sink.clone()
    }

    pub fn subscribe(&self) -> impl Stream<Item = Arc<T>> {
        let (sender, receiver) = futures::channel::mpsc::unbounded::<Arc<T>>();

        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push_back(sender);
        receiver
    }

    /// Constructor.
    ///
    /// Returns an instance as well as a [Future]. Before the [Future] is started, events can be
    /// published but will not be dispatched to subscribers.
    pub fn new() -> (Self, impl Future<Output = ()>) {
        let (sender, mut receiver) = futures::channel::mpsc::unbounded::<T>();
        let subscribers: Arc<Mutex<LinkedList<UnboundedSender<Arc<T>>>>> = Default::default();

        let subscribers_cloned = subscribers.clone();
        let task = async move {
            while let Some(item) = receiver.next().await {
                let item_packed: Arc<T> = item.into();
                subscribers_cloned
                    .lock()
                    .unwrap()
                    .drain_filter(|subscriber| {
                        if let Ok(_) = subscriber.unbounded_send(item_packed.clone()) {
                            false
                        } else {
                            true
                        }
                    });
            }
        };
        let this = Self {
            sink: sender,
            subscribers,
        };
        (this, task)
    }
}
