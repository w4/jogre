//! There is a single endpoint that handles all file uploads for an
//! account, regardless of what they are to be used for.  The Session
//! object (see Section 2) has an "uploadUrl" property in URI Template
//! (level 1) format [RFC6570], which MUST contain a variable called
//! "accountId".  The client may use this template in combination with an
//! "accountId" to get the URL of the file upload resource.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::common::{Id, UnsignedInt};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse<'a> {
    /// The id of the account used for the call.
    account_id: Id<'a>,
    /// The id representing the binary data uploaded.  The data for this
    /// id is immutable.  The id *only* refers to the binary data, not any
    /// metadata.
    blob_id: Id<'a>,
    /// The media type of the file (as specified in [RFC6838],
    /// Section 4.2) as set in the Content-Type header of the upload HTTP
    /// request.
    #[serde(rename = "type", borrow)]
    type_: Cow<'a, str>,
    /// The size of the file in octets.
    size: UnsignedInt,
}
