use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use jmap_proto::{extensions::sharing as proto_sharing, Value};
use router::ExtensionRouter;
use serde::{
    de::{value::CowStrDeserializer, DeserializeSeed, MapAccess, Visitor},
    forward_to_deserialize_any, Deserialize, Deserializer, Serialize,
};
use serde_json::value::RawValue;
use uuid::Uuid;

pub mod contacts;
pub mod core;
pub mod router;
pub mod sharing;

/// Defines a base extension to the JMAP specification.
pub trait JmapExtension: Sized {
    /// A URI that describes this extension (eg. `urn:ietf:params:jmap:contacts`).
    const EXTENSION: &'static str;

    fn router(&self) -> ExtensionRouter<Self> {
        ExtensionRouter::default()
    }
}

/// Defines an extension that can handle reads/writes.
pub trait JmapDataExtension<D>: JmapExtension {
    /// Endpoint from which this data type is exposed from (ie. `ContactBook`).
    const ENDPOINT: &'static str;
}

pub struct Get<D> {
    _phantom: PhantomData<fn(D)>,
}

impl<D> Default for Get<D> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<D, Ext: JmapDataExtension<D>> JmapEndpoint<Ext> for Get<D> {
    type Parameters<'de> = ();
    type Response<'s> = ();
    const ENDPOINT: &'static str = "";

    fn handle<'de>(&self, extension: &Ext, params: Self::Parameters<'de>) -> Self::Response<'de> {
        todo!()
    }
}

pub trait JmapEndpoint<E: JmapExtension> {
    type Parameters<'de>: Deserialize<'de>;
    type Response<'s>: Serialize + 's;

    const ENDPOINT: &'static str;

    fn handle<'de>(&self, extension: &E, params: Self::Parameters<'de>) -> Self::Response<'de>;
}

/// Defines an extension which should be exposed via session capabilities.
pub trait JmapSessionCapabilityExtension: JmapExtension {
    /// The metadata returned by this endpoint from the session endpoint.
    type Metadata: Serialize;

    fn build(&self, user: Uuid) -> Self::Metadata;
}

/// Defines an extension which should be exposed via account capabilities.
pub trait JmapAccountCapabilityExtension: JmapExtension {
    /// The metadata returned by this endpoint within account capabilities
    /// from the session endpoint.
    type Metadata: Serialize;

    fn build(&self, user: Uuid, account: Uuid) -> Self::Metadata;
}

pub struct ExtensionRouterRegistry {
    pub core: ExtensionRouter<core::Core>,
}

impl ExtensionRouterRegistry {
    pub fn handle(
        &self,
        uri: &str,
        registry: &ExtensionRegistry,
        params: ResolvedArguments<'_>,
    ) -> Option<HashMap<String, Value>> {
        let Some((namespace, uri)) = uri.split_once('/') else {
            return None;
        };

        match namespace {
            "Core" => self.core.handle(&registry.core, uri, params),
            _ => None,
        }
    }
}

/// Registry containing all extensions that can be handled by Jogre.
pub struct ExtensionRegistry {
    pub core: core::Core,
    pub contacts: contacts::Contacts,
    pub sharing_principals: sharing::Principals,
    pub sharing_principals_owner: sharing::PrincipalsOwner,
}

impl ExtensionRegistry {
    /// Builds the session capability payload from the .well-known/jmap endpoint
    pub fn build_session_capabilities(&self, user: Uuid) -> HashMap<Cow<'static, str>, Value> {
        let mut out = HashMap::new();
        out.insert(
            Cow::Borrowed(core::Core::EXTENSION),
            serde_json::to_value(JmapSessionCapabilityExtension::build(&self.core, user)).unwrap(),
        );
        out.insert(
            Cow::Borrowed(sharing::Principals::EXTENSION),
            serde_json::to_value(JmapSessionCapabilityExtension::build(
                &self.sharing_principals,
                user,
            ))
            .unwrap(),
        );
        out
    }

    pub fn build_router_registry(&self) -> ExtensionRouterRegistry {
        ExtensionRouterRegistry {
            core: self.core.router(),
        }
    }
}

/// A list of key => value pairs representing the built parameters for the
/// incoming request with all references to other requests resolved.
pub struct ResolvedArguments<'a>(pub HashMap<Cow<'a, str>, Cow<'a, Value>>);

impl<'de> Deserializer<'de> for ResolvedArguments<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(ResolvedArgumentsVisitor {
            iter: self.0.into_iter(),
            value: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ResolvedArgumentsVisitor<'de> {
    iter: <HashMap<Cow<'de, str>, Cow<'de, Value>> as IntoIterator>::IntoIter,
    value: Option<Cow<'de, Value>>,
}

impl<'de> MapAccess<'de> for ResolvedArgumentsVisitor<'de> {
    type Error = serde_json::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };

        self.value = Some(value);

        seed.deserialize(CowStrDeserializer::new(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or(serde::de::Error::custom("value is missing"))?;

        match value {
            Cow::Owned(v) => seed.deserialize(v),
            Cow::Borrowed(v) => seed.deserialize(v),
        }
    }
}
