//! Independent synthetic answers executed through real typed Core functions.
use super::*;
use crate::{errors::ErrorCode, operations::contracts::Fixture};
use serde_json::Value;
pub(crate) fn book_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"scheme": "word_character", "index_base": 1, "matching": "round_robin"}"##,
    )
    .expect("parameters");
    let p: EncodeOptions = serde_json::from_value(parameters.clone()).expect("typed");
    let source = "aba cab\nA😀 a";
    let text = "aaa😀";
    let actual = serde_json::to_value(encode(source, text, &p)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"coordinates": "1:1,1:3,2:2,3:2", "source_sha256": "b66cd26f0139b0db53ffb85a65a095972bdd06fee0bc0c6dd6ceb271c313451f", "coordinate_count": 4, "indexed_units": 4}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("cipher.book.encode known answer mismatch: {actual} != {expected}"),
        )
        .operation("cipher.book.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"source": "aba cab\nA😀 a", "text": "aaa😀"}"##)
            .expect("input"),
        parameters,
        outputs: actual,
    })
}
