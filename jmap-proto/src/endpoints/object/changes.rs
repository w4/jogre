//! When the state of the set of Foo records in an account changes on the
//! server (whether due to creation, updates, or deletion), the "state"
//! property of the "Foo/get" response will change.  The "Foo/changes"
//! method allows a client to efficiently update the state of its Foo
//! cache to match the new state on the server.

use serde::{Deserialize, Serialize};

use crate::{
    common::{Id, UnsignedInt},
    endpoints::object::ObjectState,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangesParams<'a> {
    /// The id of the account to use.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// The current state of the client.  This is the string that was
    /// returned as the "state" argument in the "Foo/get" response.  The
    /// server will return the changes that have occurred since this
    /// state.
    since_state: ObjectState<'a>,
    /// The maximum number of ids to return in the response.  The server
    /// MAY choose to return fewer than this value but MUST NOT return
    /// more. If not given by the client, the server may choose how many
    /// to return.
    max_changes: Option<UnsignedInt>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangesResponse<'a> {
    /// The id of the account used for the call.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// This is the "sinceState" argument echoed back; it's the state from
    /// which the server is returning changes.
    old_state: ObjectState<'a>,
    /// This is the state the client will be in after applying the set of
    /// changes to the old state.
    new_state: ObjectState<'a>,
    /// If true, the client may call "Foo/changes" again with the
    /// "newState" returned to get further updates.  If false, "newState"
    /// is the current server state.
    has_more_changes: bool,
    /// An array of ids for records that have been created since the old
    /// state.
    created: Vec<Id<'a>>,
    /// An array of ids for records that have been updated since the old
    /// state.
    updated: Vec<Id<'a>>,
    /// An array of ids for records that have been destroyed since the old
    /// state.
    destroyed: Vec<Id<'a>>,
}
