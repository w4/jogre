//! The Session object (see Section 2) has a "downloadUrl" property
//! which is in URI Template (level 1) format [RFC6570].

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::common::Id;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest<'a> {
    /// The id of the account to which the record with the
    /// blobId belongs.
    account_id: Id<'a>,
    /// The blobId representing the data of the file to
    /// download.
    blob_id: Id<'a>,
    /// The type for the server to set in the "Content-Type"
    /// header of the response; the blobId only represents the binary data
    /// and does not have a content-type innately associated with it.
    #[serde(rename = "type", borrow)]
    type_: Cow<'a, str>,
    /// The name for the file; the server MUST return this as the
    /// filename if it sets a "Content-Disposition" header.
    name: Cow<'a, str>,
}
