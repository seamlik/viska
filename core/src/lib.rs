pub mod android;
pub mod database;
pub mod mock_profiles;
pub mod pki;

mod jni;
mod utils;

use crate::database::IoError;
use crate::database::RawOperations;
use crate::database::Vcard;
use crate::pki::Certificate;
use crate::pki::CertificateId;
use crate::utils::ResultOption;
use futures::Stream;
use riko_runtime::HeapObject;
use sled::Db;
use std::path::Path;
use std::path::PathBuf;

/// The protagonist.
///
/// # Asynchronous getters
///
/// This struct includes a lot of asynchronous property getter methods (e.g. `vcard`). These let one
/// subscribe to the changes to that property. Upon subscription, the current value is immediately
/// returned. So in order to get the current value, one may take the first element of the `Stream`
/// and blockingly wait for it.
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
        let database = Db::start_default(&database_path)?;

        Ok(Client {
            database,
            profile_path,
        })
    }

    /// Gets the path to the profile loaded.
    pub fn profile_path(&self) -> &Path {
        &self.profile_path
    }

    // Subscribes to the `Vcard` of the current account.
    pub fn vcard(
        &self,
        account_id: Option<&CertificateId>,
    ) -> impl Stream<Item = Result<Option<Vcard>, IoError>> {
        if let Some(id) = account_id {
            futures::stream::once(futures::future::ready(self.database.vcard(id)))
        } else {
            let vcard = match self.account_id() {
                Err(e) => Err(e.into()),
                Ok(None) => Ok(None),
                Ok(Some(id)) => self.database.vcard(&id),
            };
            futures::stream::once(futures::future::ready(vcard)) // TODO: Subscription
        }
    }

    // Gets the ID of the current account.
    pub fn account_id(&self) -> Result<Option<Vec<u8>>, sled::Error> {
        self.database
            .account_certificate()
            .map_deep(|cert| cert.id())
    }
}

/* <TODO: derive> */
impl HeapObject for Client {
    fn into_handle_jni(self, _: &::jni::JNIEnv) -> riko_runtime::Handle {
        let mut heap_guard = HEAP.write().expect("Failed to write-lock the heap!");
        let mut rng = rand::thread_rng();
        let handle = loop {
            let candidate = ::rand::Rng::gen::<riko_runtime::Handle>(&mut rng);
            if !heap_guard.contains_key(&candidate) {
                break candidate;
            }
        };
        heap_guard.insert(handle, ::std::sync::Arc::new(::std::sync::Mutex::new(self)));
        handle
    }
}

::lazy_static::lazy_static! {
    pub(crate) static ref HEAP: ::riko_runtime::heap::Heap<Client> = Default::default();
}
/* <TODO: derive/> */
