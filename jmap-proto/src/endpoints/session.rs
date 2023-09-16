use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BorrowCow};

use crate::common::{Id, SessionState, UnsignedInt};

/// Implementors must take care to avoid inappropriate caching of the
/// Session object at the HTTP layer.  Since the client should only
/// refetch when it detects there is a change (via the sessionState
/// property of an API response), it is RECOMMENDED to disable HTTP
/// caching altogether, for example, by setting "Cache-Control: no-cache,
/// no-store, must-revalidate" on the response.
///
/// Exposed from https://${hostname}[:${port}]/.well-known/jmap
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Session<'a> {
    /// An object specifying the capabilities of this server.  Each key is
    /// a URI for a capability supported by the server.  The value for
    /// each of these keys is an object with further information about the
    /// server's capabilities in relation to that capability.
    #[serde(borrow)]
    capabilities: ServerCapabilities<'a>,
    /// A map of an account id to an Account object for each account (see
    /// Section 1.6.2) the user has access to.
    #[serde(borrow)]
    accounts: HashMap<Id<'a>, Account<'a>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerCapabilities<'a> {
    /// The capabilities object MUST include a property called
    /// "urn:ietf:params:jmap:core".
    #[serde(rename = "urn:ietf:params:jmap:core", borrow)]
    core: CoreCapability<'a>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoreCapability<'a> {
    /// The maximum file size, in octets, that the server will accept
    /// for a single file upload (for any purpose).  Suggested minimum:
    /// 50,000,000.
    max_size_upload: UnsignedInt,
    /// The maximum number of concurrent requests the server will
    /// accept to the upload endpoint.  Suggested minimum: 4.
    max_concurrent_upload: UnsignedInt,
    /// The maximum size, in octets, that the server will accept for a
    /// single request to the API endpoint.  Suggested minimum:
    /// 10,000,000.
    max_size_request: UnsignedInt,
    /// The maximum number of concurrent requests the server will
    /// accept to the API endpoint.  Suggested minimum: 4.
    max_concurrent_requests: UnsignedInt,
    /// The maximum number of method calls the server will accept in a
    /// single request to the API endpoint.  Suggested minimum: 16.
    max_calls_in_request: UnsignedInt,
    /// The maximum number of objects that the client may request in a
    /// single /get type method call.  Suggested minimum: 500.
    max_objects_in_get: UnsignedInt,
    /// The maximum number of objects the client may send to create,
    /// update, or destroy in a single /set type method call.  This is
    /// the combined total, e.g., if the maximum is 10, you could not
    /// create 7 objects and destroy 6, as this would be 13 actions,
    /// which exceeds the limit.  Suggested minimum: 500.
    max_objects_in_set: UnsignedInt,
    /// A list of identifiers for algorithms registered in the
    /// collation registry, as defined in [RFC4790], that the server
    /// supports for sorting when querying records.
    #[serde_as(as = "BTreeSet<BorrowCow>")]
    collation_algorithms: BTreeSet<Cow<'a, str>>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Account<'a> {
    /// A user-friendly string to show when presenting content from
    /// this account, e.g., the email address representing the owner of
    /// the account.
    #[serde(borrow)]
    name: Cow<'a, str>,
    /// This is true if the account belongs to the authenticated user
    /// rather than a group account or a personal account of another
    /// user that has been shared with them.
    is_personal: bool,
    /// This is true if the entire account is read-only.
    is_read_only: bool,
    /// The set of capability URIs for the methods supported in this
    /// account.  Each key is a URI for a capability that has methods
    /// you can use with this account.  The value for each of these
    /// keys is an object with further information about the account's
    /// permissions and restrictions with respect to this capability,
    /// as defined in the capability's specification.
    account_capabilities: AccountCapabilities,
    /// A map of capability URIs (as found in accountCapabilities) to the
    /// account id that is considered to be the user's main or default
    /// account for data pertaining to that capability.  If no account
    /// being returned belongs to the user, or in any other way there is
    /// no appropriate way to determine a default account, there MAY be no
    /// entry for a particular URI, even though that capability is
    /// supported by the server (and in the capabilities object).
    /// "urn:ietf:params:jmap:core" SHOULD NOT be present.
    #[serde_as(as = "HashMap<BorrowCow, _>")]
    primary_accounts: HashMap<Cow<'a, str>, Id<'a>>,
    /// The username associated with the given credentials, or the empty
    /// string if none.
    #[serde(borrow)]
    username: Cow<'a, str>,
    /// The URL to use for JMAP API requests.
    #[serde(borrow)]
    api_url: Cow<'a, str>,
    /// The URL endpoint to use when downloading files, in URI Template
    /// (level 1) format [RFC6570].  The URL MUST contain variables called
    /// "accountId", "blobId", "type", and "name".  The use of these
    /// variables is described in Section 6.2.  Due to potential encoding
    /// issues with slashes in content types, it is RECOMMENDED to put the
    /// "type" variable in the query section of the URL.
    #[serde(borrow)]
    download_url: Cow<'a, str>,
    /// The URL endpoint to use when uploading files, in URI Template
    /// (level 1) format [RFC6570].  The URL MUST contain a variable
    /// called "accountId".  The use of this variable is described in
    /// Section 6.1.
    #[serde(borrow)]
    upload_url: Cow<'a, str>,
    /// The URL to connect to for push events, as described in
    /// Section 7.3, in URI Template (level 1) format [RFC6570].  The URL
    /// MUST contain variables called "types", "closeafter", and "ping".
    /// The use of these variables is described in Section 7.3.
    #[serde(borrow)]
    event_source_url: Cow<'a, str>,
    /// A (preferably short) string representing the state of this object
    /// on the server.  If the value of any other property on the Session
    /// object changes, this string will change.  The current value is
    /// also returned on the API Response object (see Section 3.4),
    /// allowing clients to quickly determine if the session information
    /// has changed (e.g., an account has been added or removed), so they
    /// need to refetch the object.
    #[serde(borrow)]
    state: SessionState<'a>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountCapabilities {}
