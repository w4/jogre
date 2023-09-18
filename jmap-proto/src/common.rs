use std::borrow::Cow;

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// Where "Int" is given as a data type, it means an integer in the range
/// -2^53+1 <= value <= 2^53-1, the safe range for integers stored in a
/// floating-point double, represented as a JSON "Number".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
pub struct Int(i64);

/// Where "UnsignedInt" is given as a data type, it means an "Int" where
/// the value MUST be in the range 0 <= value <= 2^53-1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UnsignedInt(u64);

impl From<u64> for UnsignedInt {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// All record ids are assigned by the server and are immutable.
///
/// Where "Id" is given as a data type, it means a "String" of at least 1
/// and a maximum of 255 octets in size, and it MUST only contain
/// characters from the "URL and Filename Safe" base64 alphabet, as
/// defined in Section 5 of [RFC4648], excluding the pad character ("=").
/// This means the allowed characters are the ASCII alphanumeric
/// characters ("A-Za-z0-9"), hyphen ("-"), and underscore ("_").
///
/// These characters are safe to use in almost any context (e.g.,
/// filesystems, URIs, and IMAP atoms).  For maximum safety, servers
/// SHOULD also follow defensive allocation strategies to avoid creating
/// risks where glob completion or data type detection may be present
/// (e.g., on filesystems or in spreadsheets).  In particular, it is wise
/// to avoid:
///
/// - Ids starting with a dash
/// - Ids starting with digits
/// - Ids that contain only digits
/// - Ids that differ only by ASCII case (for example, A vs. a)
/// - the specific sequence of three characters "NIL" (because this sequence can be confused with
///   the IMAP protocol expression of the null value)
///
/// A good solution to these issues is to prefix every id with a single
/// alphabetical character.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Id<'a>(#[serde(borrow)] pub Cow<'a, str>);

/// Where "Date" is given as a type, it means a string in "date-time"
/// format [RFC3339].  To ensure a normalised form, the "time-secfrac"
/// MUST always be omitted if zero, and any letters in the string (e.g.,
/// "T" and "Z") MUST be uppercase.  For example,
/// "2014-10-30T14:12:00+08:00".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date(chrono::DateTime<FixedOffset>);

/// Where "UTCDate" is given as a type, it means a "Date" where the
/// "time-offset" component MUST be "Z" (i.e., it must be in UTC time).
/// For example, "2014-10-30T06:12:00Z".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtcDate(chrono::DateTime<Utc>);

/// A (preferably short) string representing the state of this object
/// on the server.  If the value of any other property on the Session
/// object changes, this string will change.  The current value is
/// also returned on the API Response object (see Section 3.4),
/// allowing clients to quickly determine if the session information
/// has changed (e.g., an account has been added or removed), so they
/// need to refetch the object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState<'a>(#[serde(borrow)] pub Cow<'a, str>);
