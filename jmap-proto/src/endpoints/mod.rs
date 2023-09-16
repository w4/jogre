pub mod blob;
pub mod core;
pub mod object;
pub mod session;

use std::{borrow::Cow, collections::HashMap, fmt::Formatter};

use serde::{
    de::{Error, MapAccess, SeqAccess},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;
use serde_with::serde_as;

use crate::{
    common::{Id, SessionState},
    util::strip_prefix_from_cow,
};

/// To allow clients to make more efficient use of the network and avoid
/// round trips, an argument to one method can be taken from the result
/// of a previous method call in the same request.
///
/// To do this, the client prefixes the argument name with "#" (an
/// octothorpe).
const REFERENCE_OCTOTHORPE: &str = "#";

#[derive(Debug, Clone, Default)]
pub struct Arguments<'a>(HashMap<Cow<'a, str>, Argument<'a>>);

impl<'a> Serialize for Arguments<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_map(Some(self.0.len()))?;

        for (key, value) in &self.0 {
            match value {
                Argument::Reference(v) => {
                    ser.serialize_entry(&format!("{REFERENCE_OCTOTHORPE}{key}"), v)?
                }
                Argument::Absolute(v) => ser.serialize_entry(key, v)?,
            }
        }

        ser.end()
    }
}

impl<'de> Deserialize<'de> for Arguments<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        pub struct Visitor {}

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Arguments<'de>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a list of arguments as defined by RFC8620")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut arguments = Arguments::default();

                while let Some(key) = map.next_key::<Cow<'de, str>>()? {
                    if let Some(key) = strip_prefix_from_cow(key.clone(), REFERENCE_OCTOTHORPE) {
                        arguments
                            .0
                            .insert(key, Argument::Reference(map.next_value()?));
                    } else {
                        arguments
                            .0
                            .insert(key, Argument::Absolute(map.next_value()?));
                    }
                }

                Ok(arguments)
            }
        }

        deserializer.deserialize_seq(Visitor {})
    }
}

#[derive(Debug, Clone)]
pub enum Argument<'a> {
    Reference(ResultReference<'a>),
    Absolute(Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResultReference<'a> {
    /// The method call id (see Section 3.2) of a previous method call in
    /// the current request.
    #[serde(borrow)]
    result_of: Cow<'a, str>,
    /// The required name of a response to that method call.
    #[serde(borrow)]
    name: Cow<'a, str>,
    /// A pointer into the arguments of the response selected via the name
    /// and resultOf properties.  This is a JSON Pointer [RFC6901], except
    /// it also allows the use of "*" to map through an array.
    #[serde(borrow)]
    path: Cow<'a, str>,
}

/// Method calls and responses are represented by the *Invocation* data
/// type. This is a tuple, represented as a JSON array containing three
/// elements.
#[derive(Clone, Debug)]
pub struct Invocation<'a> {
    /// A "String" *name* of the method to call or of the response.
    name: Cow<'a, str>,
    /// A "String[*]" object containing named *arguments* for that method
    /// or response.
    arguments: Arguments<'a>,
    /// A "String" *method call id*: an arbitrary string from the client
    /// to be echoed back with the responses emitted by that method call
    /// (a method may return 1 or more responses, as it may make implicit
    /// calls to other methods; all responses initiated by this method
    /// call get the same method call id in the response).
    request_id: Cow<'a, str>,
}

impl<'a> Serialize for Invocation<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&self.name)?;
        ser.serialize_element(&self.arguments)?;
        ser.serialize_element(&self.request_id)?;
        ser.end()
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for Invocation<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        pub struct Visitor {}

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Invocation<'de>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "an Invocation data type containing 3 elements; name, arguments & id",
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let name = seq.next_element()?.ok_or(A::Error::missing_field("name"))?;
                let arguments = seq
                    .next_element()?
                    .ok_or(A::Error::missing_field("arguments"))?;
                let request_id = seq.next_element()?.ok_or(A::Error::missing_field("id"))?;

                if seq.next_element::<Value>()?.is_some() {
                    return Err(A::Error::invalid_length(4, &self));
                }

                Ok(Invocation {
                    name,
                    arguments,
                    request_id,
                })
            }
        }

        deserializer.deserialize_seq(Visitor {})
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde_as]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    /// The set of capabilities the client wishes to use.  The client MAY
    /// include capability identifiers even if the method calls it makes
    /// do not utilise those capabilities.  The server advertises the set
    /// of specifications it supports in the Session object (see
    /// Section 2), as keys on the "capabilities" property.
    #[serde_as(as = "Vec<BorrowedCow>")]
    using: Vec<Cow<'a, str>>,
    /// An array of method calls to process on the server.  The method
    /// calls MUST be processed sequentially, in order.
    #[serde(borrow)]
    method_calls: Vec<Invocation<'a>>,
    /// A map of a (client-specified) creation id to the id the server
    /// assigned when a record was successfully created.
    ///
    /// Records may have a property that contains the id of another record.  To
    /// allow more efficient network usage, you can set this property to
    /// reference a record created earlier in the same API request.  Since the
    /// real id is unknown when the request is created, the client can instead
    /// specify the creation id it assigned, prefixed with a "#" (see
    /// Section 5.3 for more details).
    #[serde(borrow)]
    created_ids: Option<HashMap<Id<'a>, Id<'a>>>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response<'a> {
    /// An array of responses, in the same format as the "methodCalls" on
    /// the Request object.  The output of the methods MUST be added to
    /// the "methodResponses" array in the same order that the methods are
    /// processed.
    #[serde(borrow)]
    method_responses: Invocation<'a>,
    /// A map of a (client-specified) creation id to the id the server
    /// assigned when a record was successfully created.  This MUST
    /// include all creation ids passed in the original createdIds
    /// parameter of the Request object, as well as any additional ones
    /// added for newly created records.
    #[serde(borrow)]
    created_ids: Option<HashMap<Id<'a>, Id<'a>>>,
    /// The current value of the "state" string on the Session object, as
    /// described in Section 2.  Clients may use this to detect if this
    /// object has changed and needs to be refetched.
    session_state: SessionState<'a>,
}
