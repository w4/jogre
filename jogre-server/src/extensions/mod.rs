use std::{borrow::Cow, collections::HashMap};

use jmap_proto::{extensions::sharing as proto_sharing, Value};
use serde::{
    de::{value::CowStrDeserializer, DeserializeSeed, MapAccess, Visitor},
    forward_to_deserialize_any, Deserialize, Deserializer, Serialize,
};
use uuid::Uuid;

pub mod contacts;
pub mod core;
pub mod sharing;

/// Defines a base extension to the JMAP specification.
pub trait JmapExtension {
    /// A URI that describes this extension (eg. `urn:ietf:params:jmap:contacts`).
    const EXTENSION: &'static str;
}

/// Defines an extension that can handle reads/writes.
pub trait JmapDataExtension<D>: JmapExtension {
    /// Endpoint from which this data type is exposed from (ie. `ContactBook`).
    const ENDPOINT: &'static str;
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
}

/// Defines all the data types that can be handled by our [`JmapDataExtension`]
/// extensions.
pub enum ConcreteData<'a> {
    AddressBook(contacts::AddressBook),
    Principal(proto_sharing::Principal<'a>),
    ShareNotification(proto_sharing::ShareNotification<'a>),
}

impl<'a> ConcreteData<'a> {
    /// Determines which extension should handle an incoming request by
    /// the defined endpoint, and deserializes the request into the
    /// relevant data type.
    pub fn parse(endpoint: &str, data: ResolvedArguments<'a>) -> Option<Self> {
        match endpoint {
            <contacts::Contacts as JmapDataExtension<contacts::AddressBook>>::ENDPOINT => {
                Some(Self::AddressBook(Deserialize::deserialize(data).unwrap()))
            }
            <sharing::Principals as JmapDataExtension<proto_sharing::Principal>>::ENDPOINT => {
                Some(Self::Principal(Deserialize::deserialize(data).unwrap()))
            },
            <sharing::Principals as JmapDataExtension<proto_sharing::ShareNotification>>::ENDPOINT => {
                Some(Self::ShareNotification(Deserialize::deserialize(data).unwrap()))
            },
            _ => None,
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
