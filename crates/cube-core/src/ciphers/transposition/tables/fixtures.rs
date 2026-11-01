use super::*;
use crate::operations::contracts::Fixture;
use serde::de::DeserializeOwned;
use serde_json::Value;
fn fixture<T: DeserializeOwned>(
    op: &'static str,
    decode: bool,
    f: fn(&str, &T, bool) -> Result<BlockReport, OperationError>,
) -> Result<Fixture, OperationError> {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("../vectors.json")).expect("reviewed source vectors");
    let v = vectors
        .iter()
        .find(|v| v["operation"] == op)
        .expect("complete fixture");
    let parameters = serde_json::from_value(v["parameters"].clone()).expect("typed fixture");
    let actual = serde_json::to_value(f(
        v["inputs"]["text"].as_str().expect("source text"),
        &parameters,
        decode,
    )?)
    .expect("typed report");
    if actual != v["outputs"] {
        return Err(invalid(
            op,
            "fixture",
            "independent published answer mismatch",
        ));
    }
    Ok(Fixture {
        inputs: v["inputs"].clone(),
        parameters: v["parameters"].clone(),
        outputs: actual,
    })
}
macro_rules! fixture {
    ($name:ident, $id:literal, $function:path, $decode:literal) => {
        pub(crate) fn $name() -> Result<Fixture, OperationError> {
            fixture($id, $decode, $function)
        }
    };
}
fixture!(cadenus_encode, "cipher.cadenus.encode", cadenus, false);
fixture!(cadenus_decode, "cipher.cadenus.decode", cadenus, true);
fixture!(swagman_encode, "cipher.swagman.encode", swagman, false);
fixture!(swagman_decode, "cipher.swagman.decode", swagman, true);
fixture!(
    nihilist_encode,
    "cipher.nihilist_transposition.encode",
    nihilist,
    false
);
fixture!(
    nihilist_decode,
    "cipher.nihilist_transposition.decode",
    nihilist,
    true
);
