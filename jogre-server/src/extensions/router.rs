use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{value::RawValue, Value};

use crate::extensions::{JmapEndpoint, JmapExtension, ResolvedArguments};

pub struct ExtensionRouter<Ext: JmapExtension> {
    routes: HashMap<&'static str, Box<dyn ErasedJmapEndpoint<Ext> + Send + Sync>>,
}

impl<Ext: JmapExtension> ExtensionRouter<Ext> {
    pub fn register<E: JmapEndpoint<Ext> + Send + Sync + 'static>(mut self, endpoint: E) -> Self {
        self.routes.insert(E::ENDPOINT, Box::new(endpoint));
        self
    }

    pub fn handle(
        &self,
        extension: &Ext,
        method: &str,
        params: ResolvedArguments<'_>,
    ) -> Option<HashMap<String, Value>> {
        Some(self.routes.get(method)?.handle(extension, params))
    }
}

impl<Ext: JmapExtension> Default for ExtensionRouter<Ext> {
    fn default() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
}

trait ErasedJmapEndpoint<Ext> {
    fn handle(&self, endpoint: &Ext, params: ResolvedArguments<'_>) -> HashMap<String, Value>;
}

impl<Ext: JmapExtension, E: JmapEndpoint<Ext>> ErasedJmapEndpoint<Ext> for E {
    fn handle(&self, endpoint: &Ext, params: ResolvedArguments<'_>) -> HashMap<String, Value> {
        let res = <Self as JmapEndpoint<Ext>>::handle(
            self,
            endpoint,
            Deserialize::deserialize(params).unwrap(),
        );

        serde_json::from_value(serde_json::to_value(res).unwrap()).unwrap()
    }
}
