//! Utilities.

use rand::prelude::*;
use tokio::sync::broadcast::Sender;

/// Generates a random port within the private range untouched by IANA.
pub fn random_port() -> u16 {
    thread_rng().gen_range(49152..u16::MAX)
}

pub(crate) fn dummy_mpmc_sender<T>() -> Sender<T> {
    let (sender, _) = tokio::sync::broadcast::channel(1);
    sender
}
