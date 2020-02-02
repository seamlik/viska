pub mod android;
pub mod database;
pub mod mock_profiles;
pub mod pki;

mod jni;
mod utils;

use crate::database::DisplayableId;
use crate::database::IoError;
use crate::database::RawOperations;
use crate::database::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use crate::utils::ResultOption;
use riko_runtime::Heap;
use sled::Db;
use std::path::Path;
use std::path::PathBuf;

/// The protagonist.
///
/// # Asynchronous getters
///
/// A lot of property getter methods (e.g. [vcard()](Client::vcard)) return an [Iterator]. These let one subscribe to
/// the changes to that property. The current value of the property is immediately returned as the
/// first value.
///
/// [Iterator]s are used instead of `Streams` from the `futures` crate because Sled does not support
/// `Streams` natively.
pub struct Client {
    database: Db,
    profile_path: PathBuf,
}

impl Client {
    /// Constructor.
    ///
    /// No need to explicitly start running the client. Once it is created, everything is functional
    /// until the whole object is dropped.
    pub fn new(profile_path: PathBuf) -> Result<Client, sled::Error> {
        let mut database_path = profile_path.clone();
        database_path.push("database");
        let database = sled::open(&database_path)?;

        Ok(Client {
            database,
            profile_path,
        })
    }

    fn new_ffi(profile_path: &String) -> Result<Client, sled::Error> {
        Self::new(PathBuf::from(profile_path))
    }

    /// Gets the path to the profile loaded.
    pub fn profile_path(&self) -> &Path {
        &self.profile_path
    }

    /// Subscribes to the [Vcard] of the current account.
    ///
    /// # Panics
    ///
    /// If no `account_id` is provided and no account is configured in the database.
    pub fn vcard(
        &self,
        account_id: Option<&CertificateId>,
    ) -> Result<impl Iterator<Item = Result<Option<Vcard>, IoError>>, IoError> {
        let account_id_nonnull = match account_id {
            Some(id) => id.to_vec(),
            None => self
                .database
                .account_certificate()?
                .expect("No account found in the database.")
                .id(),
        };
        let current = std::iter::once(self.database.vcard(&account_id_nonnull));
        let futures = self.database.watch_vcard(&account_id_nonnull)?;
        Ok(current.chain(futures))
    }

    // Gets the ID of the current account.
    pub fn account_id_display(&self) -> Result<Option<String>, sled::Error> {
        self.database
            .account_certificate()
            .map_deep(|cert| cert.id().display())
    }
}

/* <TODO: derive> */
impl Heap for Client {
    fn into_handle(self) -> ::riko_runtime::returned::Returned<::riko_runtime::heap::Handle> {
        ::riko_runtime::heap::store(&__RIKO_POOL_Client, self).into()
    }
}

::lazy_static::lazy_static! {
    pub(crate) static ref __RIKO_POOL_Client: ::riko_runtime::heap::Pool<Client> = Default::default();
}
/* <TODO: derive/> */
