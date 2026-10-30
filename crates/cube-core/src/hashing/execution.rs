use crate::operations::execution::{self as e, Entry, Representation::*};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[Entry::pure("hash.sha256", &[Bytes], &[Text], |i, _, _| {
    Ok(json!({"output":super::sha256(&e::bytes(&i["input"])?)}))
})];
