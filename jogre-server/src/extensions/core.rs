use std::collections::BTreeSet;

use jmap_proto::endpoints::session::CoreCapability;
use uuid::Uuid;

use crate::{
    config::CoreCapabilities,
    extensions::{JmapExtension, JmapSessionCapabilityExtension},
};

#[derive(Clone)]
pub struct Core {
    pub(crate) core_capabilities: CoreCapabilities,
}

impl JmapExtension for Core {
    const EXTENSION: &'static str = "urn:ietf:params:jmap:core";
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
