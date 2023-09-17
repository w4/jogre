use serde::Deserialize;

use crate::store::StoreConfig;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// A private key used for encrypting data at rest, building CSRF tokens,
    /// etc after being fed through Argon2 for key derivation. This key should
    /// be at least 32 bytes long.
    pub private_key: String,
    /// Storage configuration, supported databases are currently `rocksdb`.
    ///
    /// ```toml
    /// [store]
    /// type = "rocksdb"
    /// path = "db"
    /// ```
    pub store: StoreConfig,
}
