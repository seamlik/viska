use std::fmt::Display;
use std::fmt::Formatter;

pub mod pki;

/// Combination of an account ID and a device ID.
///
/// This struct is used to identify an entity a client can interact with. For example, somewhere a real-time message can
/// be sent to.
///
/// An empty string represents an absent component. However, an `Address` with the `device` part but not the `account`
/// part is likely invalid in most cases.
pub struct Address {
    pub account: String,
    pub device: String,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.account, self.device)
    }
}
