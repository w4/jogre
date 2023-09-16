//! Binary data may be copied *between* two different accounts using the
//! "Blob/copy" method rather than having to download and then reupload
//! on the client.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{common::Id, endpoints::object::set::SetError};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CopyRequest<'a> {
    /// The id of the account to copy blobs from.
    #[serde(borrow)]
    from_account_id: Id<'a>,
    /// The id of the account to copy blobs to.
    account_id: Id<'a>,
    /// A list of ids of blobs to copy to the other account.
    blob_ids: Vec<Id<'a>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CopyResponse<'a> {
    /// The id of the account blobs were copied from.
    #[serde(borrow)]
    from_account_id: Id<'a>,
    /// The id of the account blobs were copied to.
    account_id: Id<'a>,
    /// A map of the blobId in the fromAccount to the id for the blob in
    /// the account it was copied to, or null if none were successfully
    /// copied.
    #[serde(default)]
    copied: HashMap<Id<'a>, Id<'a>>,
    /// A map of blobId to a SetError object for each blob that failed to
    /// be copied, or null if none.
    #[serde(default)]
    not_copied: HashMap<Id<'a>, SetError<'a>>,
}
