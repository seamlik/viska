use std::fmt::Display;
use std::fmt::Formatter;

pub mod pki;

/// Combination of an account ID and a device ID.
///
/// It is used to identify an entity a client can interact with. For example, specifying the destination of a message.
///
/// Fields are bytes returned from `pki::Certificate::id()` and are optional. However, an
/// `Address` with the `device` part but not the `account` part is likely invalid in most cases.
pub struct Address {
    pub account: Option<Vec<u8>>,
    pub device: Option<Vec<u8>>,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let convert = |x: &Option<Vec<u8>>| match x {
            Some(data) => multihash::to_hex(&data),
            None => "".to_string(),
        };
        write!(f, "{}/{}", convert(&self.account), convert(&self.device))
    }
}
