#[macro_use]
extern crate log;

use std::fmt::Display;
use std::fmt::Formatter;

pub mod android;
pub mod models;
pub mod pki;

/// Combination of an account ID and a device ID.
///
/// It is used to identify an entity a client can interact with. For example, specifying the destination of a message.
pub struct Address {
    pub account: Vec<u8>,
    pub device: Vec<u8>,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let convert = |data: &[u8]| data_encoding::HEXUPPER.encode(data);
        write!(f, "{}/{}", convert(&self.account), convert(&self.device))
    }
}
