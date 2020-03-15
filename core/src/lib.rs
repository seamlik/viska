#[path = "../../target/riko/bridge.rs"]
mod bridge;

pub mod android;
pub mod pki;
pub(crate) mod util;

/// The protagonist.
///
/// # Asynchronous Getters
///
/// Many of the property getter methods (e.g. [account_vcard()](Client::account_vcard)) return a
/// [Stream]. These let one subscribe to
/// the changes to that property. The current value of the property is immediately returned as the
/// first value.
pub struct Client;

impl Client {
    /// Constructor.
    ///
    /// No need to explicitly start running the client. Once it is created, everything is functional
    /// until the whole object is dropped.
    pub fn create() -> Client {
        Self
    }
}
