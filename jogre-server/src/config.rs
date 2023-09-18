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
    /// Capabilities of the server as advertised to the client, and enforced
    /// at the server.
    #[serde(default)]
    pub core_capabilities: CoreCapabilities,
    /// Base URL of the server
    pub base_url: url::Url,
}

#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CoreCapabilities {
    /// The maximum file size, in octets, that the server will accept
    /// for a single file upload (for any purpose).  Suggested minimum:
    /// 50,000,000.
    #[serde(default = "CoreCapabilities::default_max_size_upload")]
    pub max_size_upload: u64,
    /// The maximum number of concurrent requests the server will
    /// accept to the upload endpoint.  Suggested minimum: 4.
    #[serde(default = "CoreCapabilities::default_max_concurrent_upload")]
    pub max_concurrent_upload: u64,
    /// The maximum size, in octets, that the server will accept for a
    /// single request to the API endpoint.  Suggested minimum:
    /// 10,000,000.
    #[serde(default = "CoreCapabilities::default_max_size_request")]
    pub max_size_request: u64,
    /// The maximum number of concurrent requests the server will
    /// accept to the API endpoint.  Suggested minimum: 4.
    #[serde(default = "CoreCapabilities::default_max_concurrent_requests")]
    pub max_concurrent_requests: u64,
    /// The maximum number of method calls the server will accept in a
    /// single request to the API endpoint.  Suggested minimum: 16.
    #[serde(default = "CoreCapabilities::default_max_calls_in_request")]
    pub max_calls_in_request: u64,
    /// The maximum number of objects that the client may request in a
    /// single /get type method call.  Suggested minimum: 500.
    #[serde(default = "CoreCapabilities::default_max_objects_in_get")]
    pub max_objects_in_get: u64,
    /// The maximum number of objects the client may send to create,
    /// update, or destroy in a single /set type method call.  This is
    /// the combined total, e.g., if the maximum is 10, you could not
    /// create 7 objects and destroy 6, as this would be 13 actions,
    /// which exceeds the limit.  Suggested minimum: 500.
    #[serde(default = "CoreCapabilities::default_max_objects_in_set")]
    pub max_objects_in_set: u64,
}

impl Default for CoreCapabilities {
    fn default() -> Self {
        Self {
            max_size_upload: Self::default_max_size_upload(),
            max_concurrent_upload: Self::default_max_concurrent_upload(),
            max_size_request: Self::default_max_size_request(),
            max_concurrent_requests: Self::default_max_concurrent_requests(),
            max_calls_in_request: Self::default_max_calls_in_request(),
            max_objects_in_get: Self::default_max_objects_in_get(),
            max_objects_in_set: Self::default_max_objects_in_set(),
        }
    }
}

impl CoreCapabilities {
    const fn default_max_size_upload() -> u64 {
        50_000_000
    }

    const fn default_max_concurrent_upload() -> u64 {
        4
    }

    const fn default_max_size_request() -> u64 {
        10_000_000
    }

    const fn default_max_concurrent_requests() -> u64 {
        4
    }

    const fn default_max_calls_in_request() -> u64 {
        16
    }

    const fn default_max_objects_in_get() -> u64 {
        500
    }

    const fn default_max_objects_in_set() -> u64 {
        500
    }
}
