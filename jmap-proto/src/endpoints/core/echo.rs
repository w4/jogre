use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, BorrowCow};

#[serde_as]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EchoParams<'a>(#[serde_as(as = "HashMap<BorrowCow, _>")] HashMap<Cow<'a, str>, Value>);

#[serde_as]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EchoResult<'a>(#[serde_as(as = "HashMap<BorrowCow, _>")] HashMap<Cow<'a, str>, Value>);
