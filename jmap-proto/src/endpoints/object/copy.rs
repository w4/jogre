//! The only way to move Foo records *between* two different accounts is
//! to copy them using the "Foo/copy" method; once the copy has
//! succeeded, delete the original.  The "onSuccessDestroyOriginal"
//! argument allows you to try to do this in one method call; however,
//! note that the two different actions are not atomic, so it is possible
//! for the copy to succeed but the original not to be destroyed for some
//! reason.
//!
//! The copy is conceptually in three phases:
//!
//! 1. Reading the current values from the "from" account.
//! 2. Writing the new copies to the other account.
//! 3. Destroying the originals in the "from" account, if requested.
//!
//! Data may change in between phases due to concurrent requests.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    common::Id,
    endpoints::object::{set::SetError, ObjectState},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CopyParams<'a, T> {
    /// The id of the account to copy records from.
    #[serde(borrow)]
    from_account_id: Id<'a>,
    /// This is a state string as returned by the "Foo/get" method.  If
    /// supplied, the string must match the current state of the account
    /// referenced by the fromAccountId when reading the data to be
    /// copied; otherwise, the method will be aborted and a
    /// "stateMismatch" error returned.  If null, the data will be read
    /// from the current state.
    if_from_in_state: Option<ObjectState<'a>>,
    /// The id of the account to copy records to.  This MUST be different
    /// to the "fromAccountId".
    account_id: Id<'a>,
    /// This is a state string as returned by the "Foo/get" method.  If
    /// supplied, the string must match the current state of the account
    /// referenced by the accountId; otherwise, the method will be aborted
    /// and a "stateMismatch" error returned.  If null, any changes will
    /// be applied to the current state.
    if_in_state: Option<ObjectState<'a>>,
    /// A map of the *creation id* to a Foo object.  The Foo object MUST
    /// contain an "id" property, which is the id (in the fromAccount) of
    /// the record to be copied.  When creating the copy, any other
    /// properties included are used instead of the current value for that
    /// property on the original.
    create: HashMap<Id<'a>, T>,
    /// If true, an attempt will be made to destroy the original records
    /// that were successfully copied: after emitting the "Foo/copy"
    /// response, but before processing the next method, the server MUST
    /// make a single call to "Foo/set" to destroy the original of each
    /// successfully copied record; the output of this is added to the
    /// responses as normal, to be returned to the client.
    #[serde(default)]
    on_success_destroy_original: bool,
    /// This argument is passed on as the "ifInState" argument to the
    /// implicit "Foo/set" call, if made at the end of this request to
    /// destroy the originals that were successfully copied.
    destroy_from_if_in_state: Option<ObjectState<'a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CopyResponse<'a, T> {
    /// The id of the account records were copied from.
    #[serde(borrow)]
    from_account_id: Id<'a>,
    /// The id of the account records were copied to.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// The state string that would have been returned by "Foo/get" on the
    /// account records that were copied to before making the requested
    /// changes, or null if the server doesn't know what the previous
    /// state string was.
    #[serde(borrow)]
    old_state: Option<ObjectState<'a>>,
    /// The state string that will now be returned by "Foo/get" on the
    /// account records were copied to.
    #[serde(borrow)]
    new_state: ObjectState<'a>,
    /// A map of the creation id to an object containing any properties of
    /// the copied Foo object that are set by the server (such as the "id"
    /// in most object types; note, the id is likely to be different to
    /// the id of the object in the account it was copied from).
    ///
    /// This argument is null if no Foo objects were successfully copied.
    #[serde(default, borrow)]
    created: HashMap<Id<'a>, T>,
    /// A map of the creation id to a SetError object for each record that
    /// failed to be copied, or null if none.
    #[serde(default, borrow)]
    not_created: HashMap<Id<'a>, SetError<'a>>,
}
