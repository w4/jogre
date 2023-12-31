use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extensions::{router::ExtensionRouter, Get, JmapDataExtension, JmapExtension};

pub struct Contacts {}

impl JmapExtension for Contacts {
    const EXTENSION: &'static str = "urn:ietf:params:jmap:contacts";

    fn router(&self) -> ExtensionRouter<Self> {
        ExtensionRouter::default().register(Get::<AddressBook>::default())
    }
}

impl JmapDataExtension<AddressBook> for Contacts {
    const ENDPOINT: &'static str = "AddressBook";
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContactMetadata {
    pub may_create_address_book: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddressBook {
    id: Uuid,
    name: String,
    is_subscribed: bool,
    owner: Uuid,
    share_with: HashMap<Uuid, AddressBookRights>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct AddressBookRights {
    may_read: bool,
    may_write: bool,
    may_admin: bool,
    may_delete: bool,
}
