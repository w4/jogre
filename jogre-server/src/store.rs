mod rocksdb;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::async_trait;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    pub username: String,
    password: String,
}

impl User {
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

    async fn has_any_users(&self) -> Result<bool, Self::Error>;

    async fn create_user(&self, user: User) -> Result<(), Self::Error>;

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;
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
impl UserProvider for Store {
    type Error = rocksdb::Error;

    async fn has_any_users(&self) -> Result<bool, Self::Error> {
        match self {
            Store::RocksDb(db) => db.has_any_users().await,
        }
    }

    async fn create_user(&self, user: User) -> Result<(), Self::Error> {
        match self {
            Store::RocksDb(db) => db.create_user(user).await,
        }
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
        match self {
            Store::RocksDb(db) => db.get_by_username(username).await,
        }
    }
}
