use crate::operations::execution::{self as e, Entry, Representation::*};
use serde_json::json;
pub(crate) const ENTRIES: &[Entry] = &[Entry::pure(
    "image.dimensions",
    &[ImageSamples],
    &[Json],
    |i, _, _| Ok(json!({"output":super::inspect(&e::image(&i["input"])?) })),
)];
