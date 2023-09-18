mod rocksdb;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::async_trait;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user corresponds to an actual end user that can login to the service,
/// objects aren't directly stored under users though - users are granted
/// access to a set of accounts that objects are stored under.
///
/// Each user automatically has a "personal" account created for them.
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    password: String,
}

impl User {
    /// Builds a new `User` with the given username and password.
    pub fn new(username: String, password: &str) -> Self {
        let password = Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .unwrap()
            .to_string();

        Self {
            id: Uuid::new_v4(),
            username,
            password,
        }
    }

    /// Verifies if the given password is valid for the user.
    pub fn verify_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.password).unwrap();
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

#[async_trait]
pub trait UserProvider {
    type Error;

    async fn increment_seq_number_for_user(&self, user: Uuid) -> Result<(), Self::Error>;

    async fn fetch_seq_number_for_user(&self, user: Uuid) -> Result<u64, Self::Error>;

    async fn has_any_users(&self) -> Result<bool, Self::Error>;

    async fn create_user(&self, user: User) -> Result<(), Self::Error>;

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;
}

/// An entity which contains many objects, these can be shared among users.
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    /// ID of the account
    pub id: Uuid,
    /// A user-friendly name for the account.
    pub name: String,
    /// Whether or not the account is a user's primary account.
    pub is_personal: bool,
    /// Whether or not the entire account is read-only.
    pub is_read_only: bool,
}

impl Account {
    pub fn new(name: String, is_personal: bool, is_read_only: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            is_personal,
            is_read_only,
        }
    }
}

#[async_trait]
pub trait AccountProvider {
    type Error;

    /// Creates or updates an account in the data store.
    async fn create_account(&self, account: Account) -> Result<(), Self::Error>;

    /// Grants a user access to an account.
    async fn attach_account_to_user(
        &self,
        account: Uuid,
        user: Uuid,
        access: AccountAccessLevel,
    ) -> Result<(), Self::Error>;

    /// Fetches a list of accounts for the given user.
    async fn get_accounts_for_user(&self, user_id: Uuid) -> Result<Vec<Account>, Self::Error>;
}

#[repr(u8)]
pub enum AccountAccessLevel {
    Owner,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StoreConfig {
    #[serde(rename = "rocksdb")]
    RocksDb(rocksdb::Config),
}

pub enum Store {
    RocksDb(rocksdb::RocksDb),
}

impl Store {
    pub fn from_config(config: StoreConfig) -> Self {
        match config {
            StoreConfig::RocksDb(config) => Self::RocksDb(rocksdb::RocksDb::new(config)),
        }
    }
}

#[async_trait]
impl AccountProvider for Store {
    type Error = rocksdb::Error;

    async fn create_account(&self, account: Account) -> Result<(), Self::Error> {
        match self {
            Store::RocksDb(db) => db.create_account(account).await,
        }
    }

    async fn attach_account_to_user(
        &self,
        account: Uuid,
        user: Uuid,
        access: AccountAccessLevel,
    ) -> Result<(), Self::Error> {
        match self {
            Store::RocksDb(db) => db.attach_account_to_user(account, user, access).await,
        }
    }

    async fn get_accounts_for_user(&self, user_id: Uuid) -> Result<Vec<Account>, Self::Error> {
        match self {
            Store::RocksDb(db) => db.get_accounts_for_user(user_id).await,
        }
    }
}

#[async_trait]
impl UserProvider for Store {
    type Error = rocksdb::Error;

    async fn increment_seq_number_for_user(&self, user: Uuid) -> Result<(), Self::Error> {
        match self {
            Store::RocksDb(db) => db.increment_seq_number_for_user(user).await,
        }
    }

    async fn fetch_seq_number_for_user(&self, user: Uuid) -> Result<u64, Self::Error> {
        match self {
            Store::RocksDb(db) => db.fetch_seq_number_for_user(user).await,
        }
    }

    /// Checks if any users have been registered to decide whether a root
    /// account should be created at boot.
    async fn has_any_users(&self) -> Result<bool, Self::Error> {
        match self {
            Store::RocksDb(db) => db.has_any_users().await,
        }
    }

    /// Creates or updates a user in the store.
    async fn create_user(&self, user: User) -> Result<(), Self::Error> {
        match self {
            Store::RocksDb(db) => db.create_user(user).await,
        }
    }

    /// Fetches a user by their username.
    async fn get_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
        match self {
            Store::RocksDb(db) => db.get_by_username(username).await,
        }
    }
}
