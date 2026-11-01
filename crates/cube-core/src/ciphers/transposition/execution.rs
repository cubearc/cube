use super::tables::*;
use super::ubchi;
use crate::operations::execution::{
    self as e, Entry, Failure,
    Representation::{Json, Text},
};
macro_rules! entry {
    ($id:literal, $function:path, $decode:literal) => {
        Entry::pure($id, &[Text], &[Text, Json, Json], |i, p, _| {
            let parameters = crate::ciphers::poly::parse_parameters(&p.to_string())
                .map_err(|error| Failure(error.code.as_str()))?;
            let report = $function(e::text(i, "text")?, &parameters, $decode)
                .map_err(|error| Failure(error.code.as_str()))?;
            serde_json::to_value(report).map_err(|_| Failure("invalid_input"))
        })
    };
}
pub(crate) const ENTRIES: &[Entry] = &[
    entry!("cipher.cadenus.encode", cadenus, false),
    entry!("cipher.cadenus.decode", cadenus, true),
    entry!("cipher.swagman.encode", swagman, false),
    entry!("cipher.swagman.decode", swagman, true),
    entry!("cipher.nihilist_transposition.encode", nihilist, false),
    entry!("cipher.nihilist_transposition.decode", nihilist, true),
    entry!("cipher.ubchi.encode", ubchi::transform, false),
    entry!("cipher.ubchi.decode", ubchi::transform, true),
];
