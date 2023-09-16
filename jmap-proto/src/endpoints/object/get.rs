//! Objects of type Foo are fetched via a call to "Foo/get".

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BorrowCow};

use crate::{common::Id, endpoints::object::ObjectState};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetParams<'a> {
    /// The id of the account to use.
    account_id: Id<'a>,
    /// The ids of the Foo objects to return.  If null, then *all* records
    /// of the data type are returned, if this is supported for that data
    /// type and the number of records does not exceed the
    /// "maxObjectsInGet" limit.
    ids: Option<Vec<Id<'a>>>,
    /// If supplied, only the properties listed in the array are returned
    /// for each Foo object.  If null, all properties of the object are
    /// returned.  The id property of the object is *always* returned,
    /// even if not explicitly requested.  If an invalid property is
    /// requested, the call MUST be rejected with an "invalidArguments"
    /// error.
    #[serde_as(as = "Option<Vec<BorrowCow>>")]
    properties: Option<Vec<Cow<'a, str>>>,
}

// TODO: requestTooLarge error variant
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetResponse<'a, T> {
    /// The id of the account used for the call.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// A (preferably short) string representing the state on the server
    /// for *all* the data of this type in the account (not just the
    /// objects returned in this call).  If the data changes, this string
    /// MUST change.  If the Foo data is unchanged, servers SHOULD return
    /// the same state string on subsequent requests for this data type.
    /// When a client receives a response with a different state string to
    /// a previous call, it MUST either throw away all currently cached
    /// objects for the type or call "Foo/changes" to get the exact
    /// changes.
    state: ObjectState<'a>,
    /// An array of the Foo objects requested.  This is the *empty array*
    /// if no objects were found or if the "ids" argument passed in was
    /// also an empty array.  The results MAY be in a different order to
    /// the "ids" in the request arguments.  If an identical id is
    /// included more than once in the request, the server MUST only
    /// include it once in either the "list" or the "notFound" argument of
    /// the response.
    list: Vec<T>,
    /// This array contains the ids passed to the method for records that
    /// do not exist.  The array is empty if all requested ids were found
    /// or if the "ids" argument passed in was either null or an empty
    /// array.
    id: Vec<Id<'a>>,
}
