//! When something changes on the server, the server pushes a StateChange
//! object to the client.

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{common::Id, endpoints::object::ObjectState, events::Event};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StateChange<'a> {
    #[serde(borrow)]
    changed: HashMap<Id<'a>, HashMap<Cow<'a, str>, ObjectState<'a>>>,
}

impl<'a> Event for StateChange<'a> {
    const NAME: &'static str = "StateChange";
}
