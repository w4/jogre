use std::borrow::Cow;

pub fn strip_prefix_from_cow<'a>(input: Cow<'a, str>, prefix: &str) -> Option<Cow<'a, str>> {
    match input {
        Cow::Borrowed(v) => v.strip_prefix(prefix).map(Cow::Borrowed),
        Cow::Owned(v) => v
            .strip_prefix(prefix)
            .map(ToString::to_string)
            .map(Cow::Owned),
    }
}
