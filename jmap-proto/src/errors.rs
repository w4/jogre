use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestError {
    #[serde(rename = "type")]
    type_: ProblemType,
    status: u16,
    detail: Cow<'static, str>,
    #[serde(flatten)]
    meta: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProblemType {
    /// The client included a capability in the "using" property of the
    /// request that the server does not support.
    #[serde(rename = "urn:ietf:params:jmap:error:unknownCapability")]
    UnknownCapability,
    /// The content type of the request was not "application/json" or the
    /// request did not parse as I-JSON.
    #[serde(rename = "urn:ietf:params:jmap:error:notJSON")]
    NotJson,
    /// The request parsed as JSON but did not match the type signature of
    /// the Request object.
    #[serde(rename = "urn:ietf:params:jmap:error:notRequest")]
    NotRequest,
    /// The request was not processed as it would have exceeded one of the
    /// request limits defined on the capability object, such as
    /// maxSizeRequest, maxCallsInRequest, or maxConcurrentRequests.  A
    /// "limit" property MUST also be present on the "problem details"
    /// object, containing the name of the limit being applied.
    #[serde(rename = "urn:ietf:params:jmap:error:limit")]
    OverLimit,
}

/// If a method encounters an error, the appropriate "error" response
/// MUST be inserted at the current point in the "methodResponses" array
/// and, unless otherwise specified, further processing MUST NOT happen
/// within that method call.
///
/// Any further method calls in the request MUST then be processed as
/// normal.  Errors at the method level MUST NOT generate an HTTP-level
/// error.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MethodError {
    /// Some internal server resource was temporarily unavailable.
    ///
    /// Attempting the same operation later (perhaps after a backoff with a
    /// random factor) may succeed.
    ServerUnavailable,
    /// An unexpected or unknown error occurred during the processing of the call.
    ///
    /// The method call made no changes to the server's state.  Attempting the
    /// same operation again is expected to fail again.  Contacting the service
    /// administrator is likely necessary to resolve this problem if it is
    /// persistent.
    ServerFail,
    /// Some, but not all, expected changes described by the method occurred.
    ///
    /// The client MUST resynchronise impacted data to determine server state.
    ///
    /// Use of this error is strongly discouraged.
    ServerPartialFail,
    /// The server does not recognise this method name.
    UnknownMethod,
    /// One of the arguments is of the wrong type or is otherwise invalid, or a
    /// required argument is missing. A "description" property MAY be present to
    /// help debug with an explanation of what the problem was.  This is a
    /// non-localised string, and it is not intended to be shown directly to
    /// end users.
    InvalidArguments,
    /// The method used a result reference for one of its arguments (see
    /// Section 3.7), but this failed to resolve.
    InvalidResultReference,
    /// The method and arguments are valid, but executing the method would
    /// violate an Access Control List (ACL) or other permissions policy.
    Forbidden,
    /// The accountId does not correspond to a valid account.
    AccountNotFound,
    /// The accountId given corresponds to a valid account, but the account
    /// does not support this method or data type.
    AccountNotSupportedByMethod,
    /// This method modifies state, but the account is read-only (as returned on
    /// the corresponding Account object in the JMAP Session resource).
    AccountReadOnly,
}
