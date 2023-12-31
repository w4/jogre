use jmap_proto::{
    common::Id,
    extensions::sharing::{
        Principal, PrincipalsAccountCapabilities, PrincipalsOwnerAccountCapabilities,
        PrincipalsSessionCapabilities, ShareNotification,
    },
};
use uuid::Uuid;

use crate::extensions::{
    router::ExtensionRouter, Get, JmapAccountCapabilityExtension, JmapDataExtension, JmapExtension,
    JmapSessionCapabilityExtension,
};

/// Represents support for the `Principal` and `ShareNotification` data types and associated API
/// methods.
pub struct Principals {}

impl JmapExtension for Principals {
    const EXTENSION: &'static str = "urn:ietf:params:jmap:principals";

    fn router(&self) -> ExtensionRouter<Self> {
        ExtensionRouter::default()
            .register(Get::<Principal<'static>>::default())
            .register(Get::<ShareNotification<'static>>::default())
    }
}

impl JmapSessionCapabilityExtension for Principals {
    type Metadata = PrincipalsSessionCapabilities;

    fn build(&self, _user: Uuid) -> Self::Metadata {
        PrincipalsSessionCapabilities {}
    }
}

impl JmapAccountCapabilityExtension for Principals {
    type Metadata = PrincipalsAccountCapabilities<'static>;

    fn build(&self, _user: Uuid, _account: Uuid) -> Self::Metadata {
        PrincipalsAccountCapabilities {
            current_user_principal_id: None,
        }
    }
}

impl JmapDataExtension<Principal<'static>> for Principals {
    const ENDPOINT: &'static str = "Principal";
}

impl JmapDataExtension<ShareNotification<'static>> for Principals {
    const ENDPOINT: &'static str = "ShareNotification";
}

/// This URI is solely used as a key in an account’s accountCapabilities property;
/// it does not appear in the JMAP Session capabilities. Support is implied by the
/// `urn:ietf:params:jmap:principals` session capability.
pub struct PrincipalsOwner {}

impl JmapExtension for PrincipalsOwner {
    const EXTENSION: &'static str = "urn:ietf:params:jmap:principals:owner";
}

impl JmapAccountCapabilityExtension for PrincipalsOwner {
    type Metadata = PrincipalsOwnerAccountCapabilities<'static>;

    fn build(&self, _user: Uuid, _account: Uuid) -> Self::Metadata {
        PrincipalsOwnerAccountCapabilities {
            account_id_for_principal: Id("test".into()),
            principal_id: Id("test".into()),
        }
    }
}
