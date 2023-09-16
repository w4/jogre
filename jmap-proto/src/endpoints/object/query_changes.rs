//! The "Foo/queryChanges" method allows a client to efficiently update
//! the state of a cached query to match the new state on the server.

use serde::{Deserialize, Serialize};

use crate::{
    common::{Id, UnsignedInt},
    endpoints::object::query::{Comparator, Filter, QueryParams, QueryState},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryChangesParams<'a> {
    /// The id of the account to use.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// The filter argument that was used with "Foo/query".
    filter: Option<Filter<'a>>,
    /// The sort argument that was used with "Foo/query".
    #[serde(default)]
    sort: Vec<Comparator<'a>>,
    /// The current state of the query in the client.  This is the string
    /// that was returned as the "queryState" argument in the "Foo/query"
    /// response with the same sort/filter.  The server will return the
    /// changes made to the query since this state.
    since_query_state: QueryState<'a>,
    /// The maximum number of changes to return in the response.  See
    /// error descriptions below for more details.
    max_changes: Option<UnsignedInt>,
    /// The last (highest-index) id the client currently has cached from
    /// the query results.  When there are a large number of results, in a
    /// common case, the client may have only downloaded and cached a
    /// small subset from the beginning of the results.  If the sort and
    /// filter are both only on immutable properties, this allows the
    /// server to omit changes after this point in the results, which can
    /// significantly increase efficiency.  If they are not immutable,
    /// this argument is ignored.
    up_to_id: Option<Id<'a>>,
    /// Does the client wish to know the total number of results now in
    /// the query?  This may be slow and expensive for servers to
    /// calculate, particularly with complex filters, so clients should
    /// take care to only request the total when needed.
    #[serde(default)]
    calculate_total: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryChangesResponse<'a> {
    /// The id of the account used for the call.
    #[serde(borrow)]
    account_id: Id<'a>,
    /// This is the "sinceQueryState" argument echoed back; that is, the
    /// state from which the server is returning changes.
    old_query_state: QueryState<'a>,
    /// This is the state the query will be in after applying the set of
    /// changes to the old state.
    new_query_state: QueryParams<'a>,
    /// The total number of Foos in the results (given the "filter").
    /// This argument MUST be omitted if the "calculateTotal" request
    /// argument is not true.
    total: Option<UnsignedInt>,
    /// The "id" for every Foo that was in the query results in the old
    /// state and that is not in the results in the new state.
    ///
    /// If the server cannot calculate this exactly, the server MAY return
    /// the ids of extra Foos in addition that may have been in the old
    /// results but are not in the new results.
    ///
    /// If the sort and filter are both only on immutable properties and
    /// an "upToId" is supplied and exists in the results, any ids that
    /// were removed but have a higher index than "upToId" SHOULD be
    /// omitted.
    ///
    /// If the "filter" or "sort" includes a mutable property, the server
    /// MUST include all Foos in the current results for which this
    /// property may have changed.  The position of these may have moved
    /// in the results, so they must be reinserted by the client to ensure
    /// its query cache is correct.
    removed: Vec<Id<'a>>,
    /// The id and index in the query results (in the new state) for every
    /// Foo that has been added to the results since the old state AND
    /// every Foo in the current results that was included in the
    /// "removed" array (due to a filter or sort based upon a mutable
    /// property).
    ///
    /// If the sort and filter are both only on immutable properties and
    /// an "upToId" is supplied and exists in the results, any ids that
    /// were added but have a higher index than "upToId" SHOULD be
    /// omitted.
    ///
    /// The array MUST be sorted in order of index, with the lowest index
    /// first.
    added: Vec<AddedItem<'a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddedItem<'a> {
    #[serde(borrow)]
    id: Id<'a>,
    index: UnsignedInt,
}
