use std::path::PathBuf;

use axum::async_trait;
use rocksdb::{IteratorMode, Options, DB};
use serde::Deserialize;

use crate::store::{User, UserProvider};

#[derive(Debug)]
pub enum Error {}

const USER_BY_USERNAME_CF: &str = "users_by_username";
const USER_BY_UUID_CF: &str = "users_by_uuid";

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    path: PathBuf,
}

// TODO: lots of blocking on async thread
pub struct RocksDb {
    db: DB,
}

impl RocksDb {
    pub fn new(config: Config) -> Self {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);
        db_options.create_missing_column_families(true);

        let db = DB::open_cf(
            &db_options,
            config.path,
            [USER_BY_USERNAME_CF, USER_BY_UUID_CF],
        )
        .unwrap();

        Self { db }
    }
}

#[async_trait]
impl UserProvider for RocksDb {
    type Error = Error;

    async fn has_any_users(&self) -> Result<bool, Self::Error> {
        let by_uuid_handle = self.db.cf_handle(USER_BY_UUID_CF).unwrap();
        Ok(self
            .db
            .full_iterator_cf(by_uuid_handle, IteratorMode::Start)
            .next()
            .is_some())
    }

    async fn create_user(&self, user: User) -> Result<(), Self::Error> {
        let bytes = bincode::serde::encode_to_vec(&user, BINCODE_CONFIG).unwrap();

        let by_uuid_handle = self.db.cf_handle(USER_BY_UUID_CF).unwrap();
        self.db
            .put_cf(by_uuid_handle, user.id.as_bytes(), bytes)
            .unwrap();

        let by_username_handle = self.db.cf_handle(USER_BY_USERNAME_CF).unwrap();
        self.db
            .put_cf(
                by_username_handle,
                user.username.as_bytes(),
                user.id.as_bytes(),
            )
            .unwrap();

        Ok(())
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let uuid = {
            let by_username_handle = self.db.cf_handle(USER_BY_USERNAME_CF).unwrap();
            self.db.get_pinned_cf(by_username_handle, username).unwrap()
        };

        let Some(uuid) = uuid else {
            return Ok(None);
        };

        let user_bytes = {
            let by_uuid_handle = self.db.cf_handle(USER_BY_UUID_CF).unwrap();
            self.db.get_pinned_cf(by_uuid_handle, &uuid).unwrap()
        };

        let Some(user_bytes) = user_bytes else {
            return Ok(None);
        };

        Ok(Some(
            bincode::serde::decode_from_slice(&user_bytes, BINCODE_CONFIG)
                .unwrap()
                .0,
        ))
    }
}
