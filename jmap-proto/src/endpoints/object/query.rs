//! For data sets where the total amount of data is expected to be very
//! small, clients can just fetch the complete set of data and then do
//! any sorting/filtering locally.  However, for large data sets (e.g.,
//! multi-gigabyte mailboxes), the client needs to be able to
//! search/sort/window the data type on the server.
//!
//! A query on the set of Foos in an account is made by calling "Foo/
//! query".  This takes a number of arguments to determine which records
//! to include, how they should be sorted, and which part of the result
//! should be returned (the full list may be *very* long).  The result is
//! returned as a list of Foo ids.

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::{Id, Int, UnsignedInt};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams<'a> {
    /// The id of the account to use.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// Determines the set of Foos returned in the results.  If null, all
    /// objects in the account of this type are included in the results.
    filter: Filter<'a>,
    /// Lists the names of properties to compare between two Foo records,
    /// and how to compare them, to determine which comes first in the
    /// sort.  If two Foo records have an identical value for the first
    /// comparator, the next comparator will be considered, and so on.  If
    /// all comparators are the same (this includes the case where an
    /// empty array or null is given as the "sort" argument), the sort
    /// order is server dependent, but it MUST be stable between calls to
    /// "Foo/query".
    #[serde(default)]
    sort: Vec<Comparator<'a>>,
    /// Offset into the list of results to return.
    #[serde(default, flatten)]
    offset: Offset<'a>,
    /// The maximum number of results to return.  If null, no limit
    /// presumed.  The server MAY choose to enforce a maximum "limit"
    /// argument.  In this case, if a greater value is given (or if it is
    /// null), the limit is clamped to the maximum; the new limit is
    /// returned with the response so the client is aware.
    limit: Option<UnsignedInt>,
    /// Does the client wish to know the total number of results in the
    /// query?  This may be slow and expensive for servers to calculate,
    /// particularly with complex filters, so clients should take care to
    /// only request the total when needed.
    #[serde(default)]
    calculate_total: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse<'a> {
    /// The id of the account used for the call.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// A string encoding the current state of the query on the server.
    /// This string MUST change if the results of the query (i.e., the
    /// matching ids and their sort order) have changed.  The queryState
    /// string MAY change if something has changed on the server, which
    /// means the results may have changed but the server doesn't know for
    /// sure.
    query_state: QueryState<'a>,
    /// This is true if the server supports calling "Foo/queryChanges"
    /// with these "filter"/"sort" parameters.  Note, this does not
    /// guarantee that the "Foo/queryChanges" call will succeed, as it may
    /// only be possible for a limited time afterwards due to server
    /// internal implementation details.
    can_calculate_changes: bool,
    /// The zero-based index of the first result in the "ids" array within
    /// the complete list of query results.
    position: UnsignedInt,
    /// The list of ids for each Foo in the query results, starting at the
    /// index given by the "position" argument of this response and
    /// continuing until it hits the end of the results or reaches the
    /// "limit" number of ids.  If "position" is >= "total", this MUST be
    /// the empty list.
    ids: Vec<Id<'a>>,
    /// The total number of Foos in the results (given the "filter").
    /// This argument MUST be omitted if the "calculateTotal" request
    /// argument is not true.
    total: Option<UnsignedInt>,
    /// The limit enforced by the server on the maximum number of results
    /// to return.  This is only returned if the server set a limit or
    /// used a different limit than that given in the request.
    limit: Option<UnsignedInt>,
}

/// The queryState string only represents the ordered list of ids that
/// match the particular query (including its sort/filter).  There is
/// no requirement for it to change if a property on an object
/// matching the query changes but the query results are unaffected
/// (indeed, it is more efficient if the queryState string does not
/// change in this case).  The queryState string only has meaning when
/// compared to future responses to a query with the same type/sort/
/// filter or when used with /queryChanges to fetch changes.
///
/// Should a client receive back a response with a different
/// queryState string to a previous call, it MUST either throw away
/// the currently cached query and fetch it again (note, this does not
/// require fetching the records again, just the list of ids) or call
/// "Foo/queryChanges" to get the difference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryState<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum Offset<'a> {
    Position {
        /// The zero-based index of the first id in the full list of results
        /// to return.
        ///
        /// If a negative value is given, it is an offset from the end of the
        /// list.  Specifically, the negative value MUST be added to the total
        /// number of results given the filter, and if still negative, it's
        /// clamped to "0".  This is now the zero-based index of the first id
        /// to return.
        ///
        /// If the index is greater than or equal to the total number of
        /// objects in the results list, then the "ids" array in the response
        /// will be empty, but this is not an error.
        position: Int,
    },
    Anchor {
        /// A Foo id.  If supplied, the "position" argument is ignored.  The
        /// index of this id in the results will be used in combination with
        /// the "anchorOffset" argument to determine the index of the first
        /// result to return (see below for more details).
        #[serde(borrow)]
        anchor: Id<'a>,
        /// The index of the first result to return relative to the index of
        /// the anchor, if an anchor is given.  This MAY be negative.  For
        /// example, "-1" means the Foo immediately preceding the anchor is
        /// the first result in the list returned (see below for more
        /// details).
        #[serde(default)]
        anchor_offset: Int,
    },
    #[default]
    Default,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comparator<'a> {
    /// The name of the property on the Foo objects to compare.
    property: Cow<'a, str>,
    /// If true, sort in ascending order.  If false, reverse the
    /// comparator's results to sort in descending order.
    #[serde(default = "default_is_ascending")]
    is_ascending: bool,
    /// The identifier, as registered in the collation registry defined
    /// in [RFC4790], for the algorithm to use when comparing the order
    /// of strings.  The algorithms the server supports are advertised
    /// in the capabilities object returned with the Session object
    /// (see Section 2).
    collation: Option<Cow<'a, str>>,
}

const fn default_is_ascending() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Filter<'a> {
    Operator(FilterOperator<'a>),
    Condition(HashMap<Cow<'a, str>, Value>),
}

/// A *FilterCondition* is an "object" whose allowed properties and
/// semantics depend on the data type and is defined in the /query
/// method specification for that type.  It MUST NOT have an
/// "operator" property.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterCondition<'a>(HashMap<Cow<'a, str>, Value>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterOperator<'a> {
    operator: Operator,
    conditions: Vec<Filter<'a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operator {
    /// All of the conditions must match for the filter to match.
    And,
    /// At least one of the conditions must match for the
    /// filter to match.
    Or,
    /// None of the conditions must match for the filter to
    /// match.
    Not,
}
