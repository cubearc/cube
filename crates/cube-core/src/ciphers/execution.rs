use crate::operations::execution::{self as e, Entry, Representation::*};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[Entry::pure("cipher.rot13", &[Text], &[Text], |i, _, _| {
    Ok(json!({"output":super::rot13(e::text(i,"input")?)}))
})];
