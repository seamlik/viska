//! Utilities.

use tokio::sync::broadcast::Sender;

pub(crate) fn dummy_mpmc_sender<T>() -> Sender<T> {
    let (sender, _) = tokio::sync::broadcast::channel(1);
    sender
}
