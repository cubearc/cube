//! Independent synthetic answers executed through real typed Core functions.
use super::*;
use crate::{errors::ErrorCode, operations::contracts::Fixture};
use serde_json::Value;
pub(crate) fn keyboard_coordinates_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"layout": {"geometry": "original-ragged", "entries": [{"key": "k0", "layer": "base", "symbol": "a", "row": 0, "column": 0}, {"key": "k1", "layer": "base", "symbol": "b", "row": 0, "column": 1}, {"key": "k2", "layer": "base", "symbol": "😀", "row": 1, "column": 0}, {"key": "k0", "layer": "shift", "symbol": "A", "row": 0, "column": 0}, {"key": "k1", "layer": "shift", "symbol": "B", "row": 0, "column": 1}, {"key": "k2", "layer": "shift", "symbol": "β", "row": 1, "column": 0}]}, "index_base": 1}"##).expect("parameters");
    let p: Options = serde_json::from_value(parameters.clone()).expect("typed");
    let text = "a😀β";
    let actual = serde_json::to_value(encode(text, &p)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"coordinates": [[1, 1, "base"], [2, 1, "base"], [2, 1, "shift"]], "positions": [[0, 0], [1, 1], [5, 2]], "definition_sha256": "74c434e3b99dafe57a9335fa1456dbf6f009dd4424563df9a182cf4fd71a68e0"}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "text.keyboard.coordinates.encode known answer mismatch: {actual} != {expected}"
            ),
        )
        .operation("text.keyboard.coordinates.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "a😀β"}"##).expect("input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn keyboard_coordinates_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"layout": {"geometry": "original-ragged", "entries": [{"key": "k0", "layer": "base", "symbol": "a", "row": 0, "column": 0}, {"key": "k1", "layer": "base", "symbol": "b", "row": 0, "column": 1}, {"key": "k2", "layer": "base", "symbol": "😀", "row": 1, "column": 0}, {"key": "k0", "layer": "shift", "symbol": "A", "row": 0, "column": 0}, {"key": "k1", "layer": "shift", "symbol": "B", "row": 0, "column": 1}, {"key": "k2", "layer": "shift", "symbol": "β", "row": 1, "column": 0}]}, "index_base": 1}"##).expect("parameters");
    let p: Options = serde_json::from_value(parameters.clone()).expect("typed");
    let coordinates: Vec<Coordinate> =
        serde_json::from_str(r##"[[1, 1, "base"], [2, 1, "base"], [2, 1, "shift"]]"##)
            .expect("coordinates");
    let actual = serde_json::to_value(decode(&coordinates, &p)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "a😀β", "positions": [[0, 0], [1, 1], [5, 2]], "definition_sha256": "74c434e3b99dafe57a9335fa1456dbf6f009dd4424563df9a182cf4fd71a68e0"}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "text.keyboard.coordinates.decode known answer mismatch: {actual} != {expected}"
            ),
        )
        .operation("text.keyboard.coordinates.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(
            r##"{"coordinates": [[1, 1, "base"], [2, 1, "base"], [2, 1, "shift"]]}"##,
        )
        .expect("input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn keyboard_shift_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"layout": {"geometry": "original-ragged", "entries": [{"key": "k0", "layer": "base", "symbol": "a", "row": 0, "column": 0}, {"key": "k1", "layer": "base", "symbol": "b", "row": 0, "column": 1}, {"key": "k2", "layer": "base", "symbol": "😀", "row": 1, "column": 0}, {"key": "k0", "layer": "shift", "symbol": "A", "row": 0, "column": 0}, {"key": "k1", "layer": "shift", "symbol": "B", "row": 0, "column": 1}, {"key": "k2", "layer": "shift", "symbol": "β", "row": 1, "column": 0}]}, "dx": "-9223372036854775807", "dy": "0", "edge": "wrap"}"##).expect("parameters");
    let p: Shift = serde_json::from_value(parameters.clone()).expect("typed");
    let text = "ab😀ABβ";
    let actual = serde_json::to_value(shift(text, &p, false)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "ba😀BAβ", "source_coordinates": [[0, 0, "base"], [0, 1, "base"], [1, 0, "base"], [0, 0, "shift"], [0, 1, "shift"], [1, 0, "shift"]], "target_coordinates": [[0, 1, "base"], [0, 0, "base"], [1, 0, "base"], [0, 1, "shift"], [0, 0, "shift"], [1, 0, "shift"]], "positions": [[0, 0], [1, 1], [2, 2], [6, 3], [7, 4], [8, 5]], "bijective": true, "definition_sha256": "74c434e3b99dafe57a9335fa1456dbf6f009dd4424563df9a182cf4fd71a68e0"}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("text.keyboard.shift.encode known answer mismatch: {actual} != {expected}"),
        )
        .operation("text.keyboard.shift.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ab😀ABβ"}"##).expect("input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn keyboard_shift_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"layout": {"geometry": "original-ragged", "entries": [{"key": "k0", "layer": "base", "symbol": "a", "row": 0, "column": 0}, {"key": "k1", "layer": "base", "symbol": "b", "row": 0, "column": 1}, {"key": "k2", "layer": "base", "symbol": "😀", "row": 1, "column": 0}, {"key": "k0", "layer": "shift", "symbol": "A", "row": 0, "column": 0}, {"key": "k1", "layer": "shift", "symbol": "B", "row": 0, "column": 1}, {"key": "k2", "layer": "shift", "symbol": "β", "row": 1, "column": 0}]}, "dx": "-9223372036854775807", "dy": "0", "edge": "wrap"}"##).expect("parameters");
    let p: Shift = serde_json::from_value(parameters.clone()).expect("typed");
    let text = "ba😀BAβ";
    let actual = serde_json::to_value(shift(text, &p, true)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "ab😀ABβ", "source_coordinates": [[0, 1, "base"], [0, 0, "base"], [1, 0, "base"], [0, 1, "shift"], [0, 0, "shift"], [1, 0, "shift"]], "target_coordinates": [[0, 0, "base"], [0, 1, "base"], [1, 0, "base"], [0, 0, "shift"], [0, 1, "shift"], [1, 0, "shift"]], "positions": [[0, 0], [1, 1], [2, 2], [6, 3], [7, 4], [8, 5]], "bijective": true, "definition_sha256": "74c434e3b99dafe57a9335fa1456dbf6f009dd4424563df9a182cf4fd71a68e0"}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("text.keyboard.shift.decode known answer mismatch: {actual} != {expected}"),
        )
        .operation("text.keyboard.shift.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ba😀BAβ"}"##).expect("input"),
        parameters,
        outputs: actual,
    })
}
