//! Named-port adapters reuse the same byte codecs and shared execution budgets.
use crate::operations::execution::{self as e, Entry, Failure, Representation::*};
use serde_json::json;

pub(crate) const ENTRIES: &[Entry] = &[
    Entry::pure(
        "encoding.base64.encode",
        &[Bytes],
        &[Text],
        |inputs, _, _| Ok(json!({"output": super::base64_encode(&e::bytes(&inputs["input"])?)})),
    ),
    Entry::pure(
        "encoding.base64.decode",
        &[Text],
        &[Bytes],
        |inputs, _, _| {
            let data = super::base64_decode(e::text(inputs, "input")?)
                .map_err(|_| Failure("invalid_encoding"))?;
            Ok(json!({"output": e::byte_value(data)}))
        },
    ),
];
