//! Modifying the state of Foo objects on the server is done via the
//! "Foo/set" method.  This encompasses creating, updating, and
//! destroying Foo records.  This allows the server to sort out ordering
//! and dependencies that may exist if doing multiple operations at once
//! (for example, to ensure there is always a minimum number of a certain
//! record type).

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, BorrowCow};

use crate::{common::Id, endpoints::object::ObjectState};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetParams<'a, T> {
    /// The id of the account to use.
    account_id: Id<'a>,
    /// This is a state string as returned by the "Foo/get" method
    /// (representing the state of all objects of this type in the
    /// account). If supplied, the string must match the current state;
    /// otherwise, the method will be aborted and a "stateMismatch" error
    /// returned. If null, any changes will be applied to the current
    /// state.
    #[serde(borrow)]
    if_in_state: Option<ObjectState<'a>>,
    /// A map of a *creation id* (a temporary id set by the client) to Foo
    /// objects, or null if no objects are to be created.
    ///
    /// The Foo object type definition may define default values for
    /// properties.  Any such property may be omitted by the client.
    ///
    /// The client MUST omit any properties that may only be set by the
    /// server (for example, the "id" property on most object types).
    #[serde(default)]
    create: HashMap<Id<'a>, T>,
    /// A map of an id to a Patch object to apply to the current Foo
    /// object with that id, or null if no objects are to be updated.
    #[serde(default)]
    update: HashMap<Id<'a>, PatchObject<'a>>,
    /// A list of ids for Foo objects to permanently delete, or null if no
    /// objects are to be destroyed.
    #[serde(default)]
    destroy: Vec<Id<'a>>,
}

/// A *PatchObject* is of type "String[*]" and represents an unordered
/// set of patches.  The keys are a path in JSON Pointer format
/// [RFC6901], with an implicit leading "/" (i.e., prefix each key
/// with "/" before applying the JSON Pointer evaluation algorithm).
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatchObject<'a>(#[serde_as(as = "HashMap<BorrowCow, _>")] HashMap<Cow<'a, str>, Value>);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetResult<'a, T> {
    /// The id of the account used for the call.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// The state string that would have been returned by "Foo/get" before
    /// making the requested changes, or null if the server doesn't know
    /// what the previous state string was.
    #[serde(borrow)]
    old_state: Option<ObjectState<'a>>,
    /// The state string that will now be returned by "Foo/get".
    #[serde(borrow)]
    new_state: ObjectState<'a>,
    /// A map of the creation id to an object containing any properties of
    /// the created Foo object that were not sent by the client.  This
    /// includes all server-set properties (such as the "id" in most
    /// object types) and any properties that were omitted by the client
    /// and thus set to a default by the server.
    ///
    /// This argument is null if no Foo objects were successfully created.
    #[serde(default, borrow)]
    created: HashMap<Id<'a>, T>,
    /// The keys in this map are the ids of all Foos that were
    /// successfully updated.
    ///
    /// The value for each id is a Foo object containing any property that
    /// changed in a way *not* explicitly requested by the PatchObject
    /// sent to the server, or null if none.  This lets the client know of
    /// any changes to server-set or computed properties.
    ///
    /// This argument is null if no Foo objects were successfully updated.
    #[serde(default, borrow)]
    updated: HashMap<Id<'a>, Option<T>>,
    /// A list of Foo ids for records that were successfully destroyed, or
    /// null if none.
    #[serde(default, borrow)]
    destroyed: Vec<Id<'a>>,
    /// A map of the creation id to a SetError object for each record that
    /// failed to be created, or null if all successful.
    #[serde(default, borrow)]
    not_created: HashMap<Id<'a>, SetError<'a>>,
    /// A map of the Foo id to a SetError object for each record that
    /// failed to be updated, or null if all successful.
    #[serde(default, borrow)]
    not_updated: HashMap<Id<'a>, SetError<'a>>,
    /// A map of the Foo id to a SetError object for each record that
    /// failed to be destroyed, or null if all successful.
    #[serde(default, borrow)]
    not_destroyed: HashMap<Id<'a>, SetError<'a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetError<'a> {
    /// The type of error.
    #[serde(rename = "type")]
    type_: SetErrorKind,
    /// A description of the error to help with debugging that includes an
    /// explanation of what the problem was.  This is a non-localised
    /// string and is not intended to be shown directly to end users.
    #[serde(borrow)]
    description: Option<Cow<'a, str>>,
    /// The SetError object SHOULD also have a property called "properties" of
    /// type "String[]" that lists *all* the properties that were invalid. For
    /// type of `invalidProperties`.
    #[serde(borrow)]
    properties: Vec<Cow<'a, str>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SetErrorKind {
    /// (create; update; destroy).  The create/update/destroy would violate
    /// an ACL or other permissions policy.
    Forbidden,
    /// (create; update).  The create would exceed a server-defined limit
    /// on the number or total size of objects of this type.
    OverQuota,
    /// (create; update).  The create/update would result in an object that
    /// exceeds a server-defined limit for the maximum size of a single object
    /// of this type.
    TooLarge,
    /// (create).  Too many objects of this type have been created recently,
    /// and a server-defined rate limit has been reached.  It may work if tried
    /// again later.
    RateLimit,
    /// (update; destroy).  The id given to update/destroy cannot be found.
    NotFound,
    /// (update).  The PatchObject given to update the record was not a valid
    /// patch (see the patch description).
    InvalidPatch,
    /// (update).  The client requested that an object be both updated and
    /// destroyed in the same /set request, and the server has decided to
    /// therefore ignore the update.
    WillDestroy,
    /// (create; update).  The record given is invalid in some way.  For
    /// example:
    ///
    /// - It contains properties that are invalid according to the type specification of this
    ///   record type.
    /// - It contains a property that may only be set by the server (e.g., "id") and is different
    ///   to the current value.  Note, to allow clients to pass whole objects back, it is not an
    ///   error to include a server-set property in an update as long as the value is identical to
    ///   the current value on the server.
    /// - There is a reference to another record (foreign key), and the given id does not
    ///   correspond to a valid record.
    InvalidProperties,
    /// (create; destroy).  This is a singleton type, so you cannot create
    /// another one or destroy the existing one.
    Singleton,
}
