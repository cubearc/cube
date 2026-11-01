//! Pure named-port execution for native hosts; schemas validate transport first.
use crate::operations::execution::{self as e, Entry, Failure, Representation::Text};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[Entry::pure(
    "cipher.rot8000",
    &[Text],
    &[Text],
    |i, _, _| {
        super::rotations::rot8000(e::text(i, "text")?)
            .map(|text| json!({"text":text}))
            .map_err(|error| Failure(error.code.as_str()))
    },
)];
