use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    common::{Id, UtcDate},
    endpoints::session::Account,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalsSessionCapabilities {}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalsAccountCapabilities<'a> {
    /// The id of the principal in this account that corresponds to the user
    /// fetching this object, if any.
    #[serde(borrow)]
    pub current_user_principal_id: Option<Id<'a>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalsOwnerAccountCapabilities<'a> {
    /// The id of an account with the `urn:ietf:params:jmap:principals`
    /// capability that contains the corresponding Principal object.
    #[serde(borrow)]
    pub account_id_for_principal: Id<'a>,
    /// The id of the Principal that owns this account.
    #[serde(borrow)]
    pub principal_id: Id<'a>,
}

/// A Principal represents an individual, group, location (e.g. a room),
/// resource (e.g. a projector) or other entity in a collaborative environment.
/// Sharing in JMAP is generally configured by assigning rights to certain data
/// within an account to other principals, for example a user may assign
/// permission to read their calendar to a principal representing another user,
/// or their team.
///
/// In a shared environment such as a workplace, a user may have access to a
/// large number of principals.
///
/// In most systems the user will have access to a single Account containing
/// Principal objects, but they may have access to multiple if, for example,
/// aggregating data from different places.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Principal<'a> {
    /// The id of the principal.
    #[serde(borrow)]
    pub id: Id<'a>,
    pub type_: PrincipalType,
    /// The name of the principal, e.g. “Jane Doe”, or “Room 4B”.
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    /// A longer description of the principal, for example details about the facilities of a
    /// resource, or null if no description available.
    #[serde(borrow)]
    pub description: Option<Cow<'a, str>>,
    /// An email address for the principal, or null if no email is available.
    #[serde(borrow)]
    pub email: Option<Cow<'a, str>>,
    /// The time zone for this principal, if known. If not null, the value MUST
    /// be a time zone id from the IANA Time Zone Database TZDB.
    #[serde(borrow)]
    pub time_zone: Option<Cow<'a, str>>,
    /// A map of JMAP capability URIs to domain specific information about the principal in
    /// relation to that capability, as defined in the document that registered the capability.
    #[serde(borrow)]
    pub capabilities: HashMap<Cow<'a, str>, Value>,
    /// A map of account id to Account object for each JMAP Account containing data for this
    /// principal that the user has access to, or null if none.
    #[serde(borrow)]
    pub accounts: Option<HashMap<Id<'a>, Account<'a>>>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PrincipalType {
    /// This represents a single person.
    Individual,
    /// This represents a group of people.
    Group,
    /// This represents some resource, e.g. a projector.
    Resource,
    /// This represents a location.
    Location,
    /// This represents some other undefined principal.
    Other,
}

/// The ShareNotification data type records when the user’s permissions to access a shared object
/// changes. ShareNotification are only created by the server; users cannot create them explicitly.
/// Notifications are stored in the same Account as the Principals.
///
/// Clients SHOULD present the list of notifications to the user and allow them to dismiss them. To
/// dismiss a notification you use a standard “/set” call to destroy it.
///
/// The server SHOULD create a ShareNotification whenever the user’s permissions change on an
/// object. It SHOULD NOT create a notification for permission changes to a group principal, even if
/// the user is in the group.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShareNotification<'a> {
    /// The id of the ShareNotification.
    pub id: Cow<'a, str>,
    /// The time this notification was created.
    pub created: UtcDate,
    /// Who made the change.
    pub changed_by: Person<'a>,
    /// The name of the data type for the object whose permissions have changed, e.g. “Calendar” or
    /// “Mailbox”.
    pub object_id: Cow<'a, str>,
    /// The id of the account where this object exists.
    pub object_account_id: Cow<'a, str>,
    /// The name of the object at the time the notification was made.
    pub name: Cow<'a, str>,
    /// The “myRights” property of the object for the user before the change.
    pub old_rights: Cow<'a, str>,
    /// The “myRights” property of the object for the user after the change.
    pub new_rights: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person<'a> {
    /// The name of the person who made the change.
    pub name: Cow<'a, str>,
    /// The email of the person who made the change, or null if no email is available.
    pub email: Option<Cow<'a, str>>,
    /// The id of the Principal corresponding to the person who made the change, or null if no
    /// associated principal.
    pub principal: Option<Cow<'a, str>>,
}
