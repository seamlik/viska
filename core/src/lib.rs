#[path = "../../target/riko/jni/bridge.rs"]
mod bridge;

pub mod android;
pub mod database;
mod jni;
pub mod mock_profiles;
pub mod pki;
pub(crate) mod util;

use crate::database::DisplayableId;
use crate::database::IoError;
use crate::database::Profile;
use crate::database::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use crate::util::ResultOption;
use futures::prelude::*;
use riko::Heaped;
use sled::Db;
use std::path::Path;

/// The protagonist.
///
/// # Asynchronous Getters
///
/// Many of the property getter methods (e.g. [account_vcard()](Client::account_vcard)) return a
/// [Stream]. These let one subscribe to
/// the changes to that property. The current value of the property is immediately returned as the
/// first value.
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
    pub fn account_vcard(&self) -> Box<dyn Stream<Item = Result<Vcard, IoError>>> {
        let current = futures::future::ready(self.database.profile.vcard()).into_stream();
        let futures = self.database.profile.watch_vcard();
        Box::new(current.chain(futures))
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
