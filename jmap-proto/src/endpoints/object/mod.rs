use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub mod changes;
pub mod copy;
pub mod get;
pub mod query;
pub mod query_changes;
pub mod set;

/// A (preferably short) string representing the state on the server
/// for *all* the data of this type in the account (not just the
/// objects returned in this call).  If the data changes, this string
/// MUST change.  If the Foo data is unchanged, servers SHOULD return
/// the same state string on subsequent requests for this data type.
/// When a client receives a response with a different state string to
/// a previous call, it MUST either throw away all currently cached
/// objects for the type or call "Foo/changes" to get the exact
/// changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectState<'a>(#[serde(borrow)] Cow<'a, str>);
