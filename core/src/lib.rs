#[path = "../../target/riko/jni/bridge.rs"]
mod bridge;

pub mod android;
pub mod database;
mod jni;
pub mod mock_profiles;
pub mod pki;
mod utils;

use crate::database::DisplayableId;
use crate::database::IoError;
use crate::database::Profile;
use crate::database::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use crate::utils::ResultOption;
use riko::Heaped;
use sled::Db;
use std::path::Path;

/// The protagonist.
///
/// # Asynchronous getters
///
/// A lot of property getter methods (e.g. [account_vcard()](Client::account_vcard)) return an
/// [Iterator]. These let one subscribe to
/// the changes to that property. The current value of the property is immediately returned as the
/// first value.
///
/// [Iterator]s are used instead of `Streams` from the `futures` crate because Sled does not support
/// `Streams` natively.
#[derive(Heaped)]
pub struct Client {
    database: Database,
}

impl Client {
    /// Constructor.
    ///
    /// No need to explicitly start running the client. Once it is created, everything is functional
    /// until the whole object is dropped.
    pub fn create(database: Database) -> Result<Client, sled::Error> {
        Ok(Client { database })
    }

    fn create_ffi(profile: &String, cache: &String) -> Result<Self, sled::Error> {
        let database = Database::open(Path::new(profile), Path::new(cache))?;
        Self::create(database)
    }

    /// Subscribes to the [Vcard] of the current account.
    pub fn account_vcard(
        &self,
    ) -> Result<impl Iterator<Item = Result<Option<Vcard>, IoError>>, IoError> {
        let current = std::iter::once(self.database.profile.vcard());
        let futures = self.database.profile.watch_vcard()?;
        Ok(current.chain(futures))
    }

    // Gets the ID of the current account.
    pub fn account_id_display(&self) -> Result<Option<String>, sled::Error> {
        self.database
            .profile
            .certificate()
            .map_deep(|cert| cert.id().as_bytes().display())
    }
}

pub struct Database {
    profile: Db,
    cache: Db,
}

impl Database {
    pub fn open(profile: &Path, cache: &Path) -> sled::Result<Self> {
        Ok(Self {
            profile: sled::open(profile)?,
            cache: sled::open(cache)?,
        })
    }

    pub fn initialize(&self) {
        let bundle = crate::pki::new_certificate();
        let id: CertificateId = bundle.certificate.id().into();
        log::info!("Created account ID: {}", id.display());
        self.profile.set_certificate(&bundle.certificate).unwrap();
        self.profile.set_key(&bundle.keypair).unwrap();
    }
}
