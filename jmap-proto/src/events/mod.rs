//! Push notifications allow clients to efficiently update (almost)
//! instantly to stay in sync with data changes on the server.  The
//! general model for push is simple and sends minimal data over the push
//! channel: just enough for the client to know whether it needs to
//! resync.  The format allows multiple changes to be coalesced into a
//! single push update and the frequency of pushes to be rate limited by
//! the server.  It doesn't matter if some push events are dropped before
//! they reach the client; the next time it gets/sets any records of a
//! changed type, it will discover the data has changed and still sync
//! all changes.
//!
//! There are two different mechanisms by which a client can receive push
//! notifications, to allow for the different environments in which a
//! client may exist.  An event source resource (see Section 7.3) allows
//! clients that can hold transport connections open to receive push
//! notifications directly from the JMAP server.  This is simple and
//! avoids third parties, but it is often not feasible on constrained
//! platforms such as mobile devices.  Alternatively, clients can make
//! use of any push service supported by their environment.  A URL for
//! the push service is registered with the JMAP server (see
//! Section 7.2); the server then POSTs each notification to that URL.
//! The push service is then responsible for routing these to the client.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub mod state_change;

pub trait Event {
    const NAME: &'static str;

    fn into_event(self) -> BuiltEvent<'static, Self>
    where
        Self: Sized,
    {
        BuiltEvent {
            type_: Self::NAME.into(),
            inner: self,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuiltEvent<'a, T> {
    #[serde(borrow, rename = "@type")]
    type_: Cow<'a, str>,
    #[serde(flatten)]
    inner: T,
}
