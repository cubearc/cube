use crate::operations::execution::{self as e, Entry, Failure, Representation::*};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[
    Entry::pure("text.reverse", &[Text], &[Text], |i, _, _| {
        Ok(
            json!({"output":super::reverse(e::text(i,"input")?).map_err(|_|Failure("operation_failed"))?}),
        )
    }),
    Entry::pure("text.utf8.encode", &[Text], &[Bytes], |i, _, _| {
        Ok(
            json!({"output":e::byte_value(super::to_utf8(e::text(i,"input")?).map_err(|_|Failure("operation_failed"))?)}),
        )
    }),
    Entry::pure("text.utf8.decode", &[Bytes], &[Text], |i, _, _| {
        Ok(
            json!({"output":super::from_utf8(&e::bytes(&i["input"])?).map_err(|_|Failure("invalid_encoding"))?}),
        )
    }),
];
