//! Database models and operations.

diesel_migrations::embed_migrations!();

pub(crate) mod chatroom;
pub(crate) mod message;
mod object;
pub(crate) mod peer;
mod schema;
pub(crate) mod vcard;

use self::peer::PeerService;
use crate::changelog::ChangelogMerger;
use crate::mock_profile::MockProfileService;
use crate::pki::CanonicalId;
use async_std::path::PathBuf;
use blake3::Hash;
use blake3::Hasher;
use chrono::prelude::*;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use std::sync::Mutex;
use thiserror::Error;

/// THE hash function (BLAKE3) universally used in the project.
///
/// This is exported only because this algorithm isn't available in most languages or platform at moment.
#[riko::fun]
pub fn hash(src: &ByteBuf) -> ByteBuf {
    let raw_hash: [u8; 32] = blake3::hash(src).into();
    ByteBuf::from(raw_hash)
}

/// Serializes a timestamp to a floating-point number.
///
/// By using a floating-point number as the universal timestamp format, we can have arbitrary
/// precision on the time value.
pub(crate) fn float_from_time(src: DateTime<Utc>) -> f64 {
    src.timestamp() as f64 + src.timestamp_subsec_nanos() as f64 / 1_000_000_000.0
}

/// Converts a [Hash] to bytes.
pub(crate) fn bytes_from_hash(src: Hash) -> Vec<u8> {
    let raw_hash: [u8; 32] = src.into();
    raw_hash.to_vec()
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
    pub fn create(storage: &Storage) -> Result<Self, DatabaseInitializationError> {
        let database_url = match storage {
            Storage::InMemory => ":memory:".into(),
            Storage::OnDisk(path) => path.display().to_string(),
        };
        log::info!("Opening database URL {}", &database_url);
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

/// Configurations regarding account profiles.
///
/// An account profile is a directory of files that contains all data regarding an account on the device.
///
/// # Filesystem Structure of an Account Profile
///
/// - `dir_data` (e.g. on Linux it's `~/.config/Viska`)
///   - `account`
///     - `0C88CF8B12C190651C4B98885D035D43F1E87C20ADC80B5ED439FF9C76FF2BE3` (Account ID)
///       - `certificate.der`
///       - `key.der`
///       - `database`
///         - `main.db`
///         - Maybe some auxiliary files generated by SQLite
#[derive(Deserialize, Serialize)]
pub struct ProfileConfig {
    pub dir_data: std::path::PathBuf,
}

impl ProfileConfig {
    pub async fn path_database(&self, account_id: &[u8]) -> std::io::Result<PathBuf> {
        let mut destination = async_std::fs::canonicalize(&self.dir_data).await?;
        destination.push("account");
        destination.push(hex::encode_upper(account_id));
        destination.push("database");
        destination.push("main.db");
        Ok(destination)
    }

    pub async fn path_certificate(&self, account_id: &[u8]) -> std::io::Result<PathBuf> {
        let mut destination = async_std::fs::canonicalize(&self.dir_data).await?;
        destination.push("account");
        destination.push(hex::encode_upper(account_id));
        destination.push("certificate.der");
        Ok(destination)
    }

    pub async fn path_key(&self, account_id: &[u8]) -> std::io::Result<PathBuf> {
        let mut destination = async_std::fs::canonicalize(&self.dir_data).await?;
        destination.push("account");
        destination.push(hex::encode_upper(account_id));
        destination.push("key.der");
        Ok(destination)
    }
}

/// Creates a profile with a newly generated account.
///
/// # Parameters
///
/// * `dir_data`: See [ProfileConfig::dir_data]
///
/// # Returns
///
/// The generated account ID.
#[riko::fun]
pub async fn create_standard_profile(
    dir_data: std::path::PathBuf,
) -> Result<ByteBuf, CreateProfileError> {
    let bundle = crate::pki::new_certificate();
    let account_id = bundle.certificate.canonical_id();
    let profile_config = ProfileConfig { dir_data };
    let path_certificate = profile_config
        .path_certificate(account_id.as_bytes())
        .await?;

    let path_account = path_certificate.parent().unwrap();
    log::debug!("Creating account directory {}", path_account.display());
    async_std::fs::create_dir_all(path_account).await?;
    async_std::fs::write(&path_certificate, &bundle.certificate).await?;
    async_std::fs::write(
        &profile_config.path_key(account_id.as_bytes()).await?,
        &bundle.key,
    )
    .await?;

    async_std::fs::create_dir_all(
        profile_config
            .path_database(account_id.as_bytes())
            .await?
            .parent()
            .unwrap(),
    )
    .await?;
    Database::create(&Storage::OnDisk(
        profile_config.path_database(account_id.as_bytes()).await?,
    ))?;

    Ok(ByteBuf::from(account_id.as_bytes().to_vec()))
}

/// Creates a mock profile.
///
/// The profile contains a freshly-generated account with a lot of random database content.
///
/// # Returns
///
/// The generated account ID.
#[riko::fun]
pub async fn create_mock_profile(
    dir_data: std::path::PathBuf,
) -> Result<ByteBuf, CreateProfileError> {
    let profile_config = ProfileConfig {
        dir_data: dir_data.clone(),
    };
    let account_id = create_standard_profile(dir_data).await?;

    let database = Database::create(&Storage::OnDisk(
        profile_config.path_database(&account_id).await?,
    ))?;
    let changelog_merger = ChangelogMerger {
        peer_service: PeerService { verifier: None }.into(),
    }
    .into();
    let mock_profile_service = MockProfileService {
        account_id: account_id.clone().into_vec(),
        database: database.into(),
        changelog_merger,
    };
    mock_profile_service.populate_mock_data()?;
    Ok(account_id)
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
