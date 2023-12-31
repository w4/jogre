use std::collections::BTreeSet;

use jmap_proto::endpoints::session::CoreCapability;
use uuid::Uuid;

use crate::{
    config::CoreCapabilities,
    extensions::{
        router::ExtensionRouter, JmapEndpoint, JmapExtension, JmapSessionCapabilityExtension,
    },
};

#[derive(Clone)]
pub struct Core {
    pub(crate) core_capabilities: CoreCapabilities,
}

impl JmapExtension for Core {
    const EXTENSION: &'static str = "urn:ietf:params:jmap:core";

    fn router(&self) -> ExtensionRouter<Self> {
        ExtensionRouter::default().register(Echo)
    }
}

impl JmapSessionCapabilityExtension for Core {
    type Metadata = CoreCapability<'static>;

    fn build(&self, _user: Uuid) -> Self::Metadata {
        CoreCapability {
            max_size_upload: self.core_capabilities.max_size_upload.into(),
            max_concurrent_upload: self.core_capabilities.max_concurrent_upload.into(),
            max_size_request: self.core_capabilities.max_size_request.into(),
            max_concurrent_requests: self.core_capabilities.max_concurrent_requests.into(),
            max_calls_in_request: self.core_capabilities.max_calls_in_request.into(),
            max_objects_in_get: self.core_capabilities.max_objects_in_get.into(),
            max_objects_in_set: self.core_capabilities.max_objects_in_set.into(),
            collation_algorithms: BTreeSet::default(),
        }
    }
}

pub struct Echo;

impl JmapEndpoint<Core> for Echo {
    type Parameters<'de> = &'de serde_json::value::RawValue;
    type Response<'s> = &'s serde_json::value::RawValue;

    const ENDPOINT: &'static str = "echo";

    fn handle<'de>(&self, _extension: &Core, params: Self::Parameters<'de>) -> Self::Response<'de> {
        params
    }
}
