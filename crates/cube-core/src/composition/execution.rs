use crate::operations::execution::{self as e, Entry, Failure, Representation::*};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[
    Entry::pure("text.concat", &[Text, Text], &[Text], |i, p, _| {
        Ok(
            json!({"output":super::concat_text(e::text(i,"left")?,e::text(i,"right")?,e::text(p,"separator")?).map_err(|_|Failure("operation_failed"))?}),
        )
    }),
    Entry::pure("bytes.concat", &[Bytes, Bytes], &[Bytes], |i, _, _| {
        Ok(
            json!({"output":e::byte_value(super::concat_bytes(&e::bytes(&i["left"])?,&e::bytes(&i["right"])?).map_err(|_|Failure("operation_failed"))?)}),
        )
    }),
];
