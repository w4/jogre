use std::path::PathBuf;

use axum::async_trait;
use rocksdb::{IteratorMode, MergeOperands, Options, DB};
use serde::Deserialize;
use uuid::Uuid;

use crate::store::{Account, AccountAccessLevel, AccountProvider, User, UserProvider};

#[derive(Debug)]
pub enum Error {}

const USER_BY_USERNAME_CF: &str = "users_by_username";
const USER_BY_UUID_CF: &str = "users_by_uuid";
const USER_SEQ_NUMBER: &str = "users_seq_number";

const ACCOUNTS_BY_UUID: &str = "accounts_by_uuid";
const ACCOUNTS_ACCESS_BY_USER: &str = "accounts_access_by_user";

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
        db_options.set_merge_operator_associative("test operator", rocksdb_merger);
        db_options.create_missing_column_families(true);

        let db = DB::open_cf_with_opts(
            &db_options,
            config.path,
            [
                (USER_BY_USERNAME_CF, db_options.clone()),
                (USER_BY_UUID_CF, db_options.clone()),
                (ACCOUNTS_BY_UUID, db_options.clone()),
                (ACCOUNTS_ACCESS_BY_USER, db_options.clone()),
                (USER_SEQ_NUMBER, db_options.clone()),
            ],
        )
        .unwrap();

        Self { db }
    }
}

fn rocksdb_merger(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut new_val = existing_val.map(|v| v.to_vec()).unwrap_or_default();

    for operand in operands {
        let (operation, operand) = MergeOperation::parse(operand);

        match operation {
            Some(MergeOperation::Increment) => {
                if new_val.is_empty() {
                    new_val.extend_from_slice(&0_u64.to_be_bytes());
                }

                let mut carry = true;

                for byte in new_val.iter_mut().rev() {
                    if carry {
                        *byte = byte.wrapping_add(1);
                        carry = *byte == 0;
                    } else {
                        break;
                    }
                }

                if carry {
                    new_val.fill(0);
                }
            }
            None => {
                panic!("unknown operand: {operand:?}");
            }
        }
    }

    Some(new_val)
}

enum MergeOperation {
    Increment,
}

impl MergeOperation {
    pub fn parse(v: &[u8]) -> (Option<MergeOperation>, &[u8]) {
        if v == b"INCR" {
            (Some(Self::Increment), &[])
        } else {
            (None, v)
        }
    }
}

#[async_trait]
impl AccountProvider for RocksDb {
    type Error = Error;

    async fn create_account(&self, account: Account) -> Result<(), Self::Error> {
        let bytes = bincode::serde::encode_to_vec(&account, BINCODE_CONFIG).unwrap();

        let by_uuid_handle = self.db.cf_handle(ACCOUNTS_BY_UUID).unwrap();
        self.db
            .put_cf(by_uuid_handle, account.id.as_bytes(), bytes)
            .unwrap();

        Ok(())
    }

    async fn attach_account_to_user(
        &self,
        account: Uuid,
        user: Uuid,
        access: AccountAccessLevel,
    ) -> Result<(), Self::Error> {
        {
            let access_handle = self.db.cf_handle(ACCOUNTS_ACCESS_BY_USER).unwrap();

            let mut compound_key = [0_u8; 32];
            compound_key[..16].copy_from_slice(user.as_bytes());
            compound_key[16..].copy_from_slice(account.as_bytes());

            self.db
                .put_cf(access_handle, compound_key, (access as u8).to_be_bytes())
                .unwrap();
        }

        self.increment_seq_number_for_user(user).await.unwrap();

        Ok(())
    }

    async fn get_accounts_for_user(&self, user_id: Uuid) -> Result<Vec<Account>, Self::Error> {
        let access_handle = self.db.cf_handle(ACCOUNTS_ACCESS_BY_USER).unwrap();
        let account_handle = self.db.cf_handle(ACCOUNTS_BY_UUID).unwrap();

        Ok(self
            .db
            .prefix_iterator_cf(access_handle, user_id.as_bytes())
            .map(|v| v.unwrap())
            .filter_map(|(key, _access_level)| {
                let Some(account) = key.strip_prefix(user_id.as_bytes()) else {
                    panic!("got invalid key from rocksdb");
                };

                let Some(account_bytes) = self.db.get_cf(account_handle, account).unwrap() else {
                    return None;
                };

                let (res, _): (Account, _) =
                    bincode::serde::decode_from_slice(&account_bytes, BINCODE_CONFIG).unwrap();

                Some(res)
            })
            .collect())
    }
}

#[async_trait]
impl UserProvider for RocksDb {
    type Error = Error;

    async fn increment_seq_number_for_user(&self, user: Uuid) -> Result<(), Self::Error> {
        let seq_handle = self.db.cf_handle(USER_SEQ_NUMBER).unwrap();
        self.db
            .merge_cf(seq_handle, user.as_bytes(), "INCR")
            .unwrap();
        Ok(())
    }

    async fn fetch_seq_number_for_user(&self, user: Uuid) -> Result<u64, Self::Error> {
        let seq_handle = self.db.cf_handle(USER_SEQ_NUMBER).unwrap();

        let Some(bytes) = self.db.get_pinned_cf(seq_handle, user.as_bytes()).unwrap() else {
            return Ok(0);
        };

        let mut val = [0_u8; std::mem::size_of::<u64>()];
        val.copy_from_slice(&bytes);

        Ok(u64::from_be_bytes(val))
    }

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
