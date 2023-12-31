use std::sync::Arc;

use crate::{
    config::{Config, CoreCapabilities},
    extensions,
    extensions::{
        sharing::{Principals, PrincipalsOwner},
        ExtensionRegistry, ExtensionRouterRegistry,
    },
    store::Store,
};

pub mod oauth2;

pub struct Context {
    pub oauth2: oauth2::OAuth2,
    pub store: Arc<Store>,
    pub base_url: url::Url,
    pub core_capabilities: CoreCapabilities,
    pub extension_registry: ExtensionRegistry,
    pub extension_router_registry: ExtensionRouterRegistry,
}

impl Context {
    pub fn new(config: Config) -> Self {
        let derived_keys = Arc::new(DerivedKeys::new(&config.private_key));
        let store = Arc::new(Store::from_config(config.store));

        let extension_registry = ExtensionRegistry {
            core: extensions::core::Core {
                core_capabilities: config.core_capabilities,
            },
            contacts: extensions::contacts::Contacts {},
            sharing_principals: Principals {},
            sharing_principals_owner: PrincipalsOwner {},
        };

        let extension_router_registry = extension_registry.build_router_registry();

        Self {
            oauth2: oauth2::OAuth2::new(store.clone(), derived_keys),
            store,
            base_url: config.base_url,
            core_capabilities: config.core_capabilities,
            extension_registry,
            extension_router_registry,
        }
    }
}

pub struct DerivedKeys {
    pub(crate) csrf_hmac_key: [u8; argon2::Params::DEFAULT_OUTPUT_LEN],
}

impl DerivedKeys {
    /// Salt used for deriving the CSRF HMAC key
    const CSRF: &'static [u8] = b"CSRFTOKEN";

    /// Instantiates a new [`DerivedKeys`], dropping the private key.
    fn new(private_key: &str) -> Self {
        let argon2 = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::DEFAULT,
        );

        Self {
            csrf_hmac_key: Self::derive_key(&argon2, private_key, Self::CSRF),
        }
    }

    fn derive_key(
        argon2: &argon2::Argon2,
        private_key: &str,
        salt: &[u8],
    ) -> [u8; argon2::Params::DEFAULT_OUTPUT_LEN] {
        let mut out = [0_u8; argon2::Params::DEFAULT_OUTPUT_LEN];
        argon2
            .hash_password_into(private_key.as_bytes(), salt, &mut out)
            .unwrap();

        out
    }
}
