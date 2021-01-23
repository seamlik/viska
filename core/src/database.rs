//! Database models and operations.

diesel_migrations::embed_migrations!();

pub(crate) mod chatroom;
pub(crate) mod message;
mod object;
pub(crate) mod peer;
mod schema;
pub(crate) mod vcard;

use self::chatroom::ChatroomService;
use self::message::MessageService;
use self::peer::PeerService;
use crate::changelog::ChangelogMerger;
use crate::mock_profile::MockProfileService;
use crate::pki::CanonicalId;
use blake3::Hash;
use blake3::Hasher;
use chrono::prelude::*;
use diesel::prelude::*;
use serde_bytes::ByteBuf;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use vcard::VcardService;

/// THE hash function (BLAKE3) universally used in the project.
///
/// This is exported only because this algorithm isn't available in most languages or platform at moment.
#[riko::fun]
pub fn hash(src: &ByteBuf) -> ByteBuf {
    let raw_hash: [u8; 32] = blake3::hash(src).into();
    ByteBuf::from(raw_hash)
}

/// Serializes a timestamp to a floating point number.
///
/// By using a floating point number as the universal timestamp format, we can have arbitrary
/// precision on the time value.
pub(crate) fn float_from_time(src: DateTime<Utc>) -> f64 {
    src.timestamp() as f64 + src.timestamp_subsec_nanos() as f64 / 1_000_000_000.0
}

/// Converts a [Hash] to bytes.
pub(crate) fn bytes_from_hash(src: Hash) -> Vec<u8> {
    let raw_hash: [u8; 32] = src.into();
    raw_hash.to_vec()
}

/// Database config.
#[derive(Default)]
pub struct Config {
    pub storage: Storage,
}

/// Where to store the database.
pub enum Storage {
    InMemory,
    OnDisk(PathBuf),
}

impl Default for Storage {
    fn default() -> Self {
        Self::InMemory
    }
}

pub(crate) struct Database {
    pub connection: Mutex<SqliteConnection>,
}

impl Database {
    pub fn create(config: Config) -> Result<Self, DatabaseInitializationError> {
        let database_url = match config.storage {
            Storage::InMemory => ":memory:".into(),
            Storage::OnDisk(path) => path.display().to_string(),
        };
        let connection = SqliteConnection::establish(&database_url)?;

        log::info!("Beginning database migration");
        embedded_migrations::run(&connection)?;

        Ok(Self {
            connection: connection.into(),
        })
    }
}

/// Error when failed to initialize the database.
#[derive(Error, Debug)]
#[error("Failed to initialize the database")]
pub enum DatabaseInitializationError {
    DatabaseConnection(#[from] diesel::ConnectionError),
    DatabaseMigration(#[from] diesel_migrations::RunMigrationsError),
}

/// Creates a profile with a newly generated account.
///
/// A profile is a directory of files that contains all data regarding an account on the device.
/// Such a directory is supposed to be placed at the user config directory, usually named with the
/// account ID.
///
/// # Filesystem Structure of an Account Profile
///
/// - User config directory (e.g. on Linux it's `~/.config/Viska`)
///   - `account`
///     - `0C88CF8B12C190651C4B98885D035D43F1E87C20ADC80B5ED439FF9C76FF2BE3` (Account ID)
///       - `certificate.der`
///       - `key.der`
///       - `database`
///         - `main.db`
///         - Maybe some auxilliary files generated by SQLite
///
/// # Parameters
///
/// * `base_data_dir`: The location of the `account` directory described above.
///
/// # Returns
///
/// The uppercase HEX of the generated account ID.
///
/// # TODO
///
/// * Async
#[riko::fun]
pub fn create_standard_profile(base_data_dir: PathBuf) -> Result<String, CreateProfileError> {
    let bundle = crate::pki::new_certificate();
    let account_id = bundle
        .certificate
        .canonical_id()
        .to_hex()
        .to_ascii_uppercase();

    let mut destination = base_data_dir;
    destination.push(&account_id);
    std::fs::create_dir_all(&destination)?;

    destination.push("certificate.der");
    std::fs::write(&destination, &bundle.certificate)?;

    destination.pop();
    destination.push("key.der");
    std::fs::write(&destination, &bundle.key)?;

    destination.pop();
    destination.push("database");
    std::fs::create_dir_all(&destination)?;
    destination.push("main.db");

    let database_config = Config {
        storage: Storage::OnDisk(destination),
    };
    Database::create(database_config)?;

    Ok(account_id)
}

/// Creates a mock profile.
///
/// The profile contains a freshly-generated account with a lot of random database content.
#[riko::fun]
pub fn create_mock_profile(base_data_dir: PathBuf) -> Result<String, CreateProfileError> {
    let account_id_text = create_standard_profile(base_data_dir.clone())?;
    let mut destination = base_data_dir;
    destination.push(&account_id_text);
    destination.push("certificate.der");
    let certificate = std::fs::read(&destination)?;
    let account_id = certificate.canonical_id();

    destination.pop();
    destination.push("database");
    destination.push("main.db");
    let database_config = Config {
        storage: Storage::OnDisk(destination),
    };
    let database = Database::create(database_config)?;
    let event_sink = crate::util::dummy_mpmc_sender();
    let chatroom_service = Arc::new(ChatroomService {
        event_sink: event_sink.clone(),
    });
    let changelog_merger = ChangelogMerger {
        peer_service: PeerService {
            event_sink: event_sink.clone(),
            verifier: None,
        }
        .into(),
        chatroom_service: chatroom_service.clone(),
        message_service: MessageService {
            chatroom_service,
            event_sink: event_sink.clone(),
        }
        .into(),
    }
    .into();
    let mock_profile_service = MockProfileService {
        account_id,
        database: database.into(),
        vcard_service: VcardService { event_sink }.into(),
        changelog_merger,
    };
    mock_profile_service.populate_mock_data()?;
    Ok(account_id_text)
}

/// Error when failed to create a profile.
#[derive(Error, Debug)]
#[error("Failed to create a profile")]
pub enum CreateProfileError {
    DatabaseQuery(#[from] diesel::result::Error),
    FileSystem(#[from] std::io::Error),
    InitializeDatabase(#[from] DatabaseInitializationError),
}

impl CanonicalId for crate::changelog::Blob {
    fn canonical_id(&self) -> Hash {
        let mut hasher = Hasher::default();
        hasher.update(format!("Viska blob {}", self.mime).as_bytes());
        hasher.update(&self.content.len().to_be_bytes());
        hasher.update(&self.content);
        hasher.finalize()
    }
}

pub(crate) enum Event {
    Chatroom { chatroom_id: Vec<u8> },
    Message { chatroom_id: Vec<u8> },
    Roster,
    Vcard { account_id: Vec<u8> },
}
