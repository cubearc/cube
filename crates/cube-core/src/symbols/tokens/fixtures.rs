//! Independent synthetic answers executed through real typed Core functions.
use super::*;
use crate::{errors::ErrorCode, operations::contracts::Fixture};
use serde_json::Value;
pub(crate) fn symbol_tokens_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"table": {"source_id": "synthetic:original-v1", "profile_id": "example-token-table", "license_id": "original-fixture", "entries": [{"token": "△", "value": "A", "aliases": ["triangle"]}, {"token": "○", "value": "B", "aliases": []}]}, "boundaries": {"kind": "separated", "separator": "|"}, "output_separator": "|"}"##).expect("parameters");
    let p: Parameters = serde_json::from_value(parameters.clone()).expect("typed");
    let text = "triangle|○|△";
    let actual = serde_json::to_value(transform(text, &p, false)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "A|B|A", "records": [{"input_token": "triangle", "canonical_token": "△", "output_token": "A", "byte_offset": 0, "scalar_offset": 0, "alias_used": true}, {"input_token": "○", "canonical_token": "○", "output_token": "B", "byte_offset": 9, "scalar_offset": 9, "alias_used": false}, {"input_token": "△", "canonical_token": "△", "output_token": "A", "byte_offset": 13, "scalar_offset": 11, "alias_used": false}], "source_id": "synthetic:original-v1", "profile_id": "example-token-table", "license_id": "original-fixture", "definition_sha256": "2186a62559b8280a2fc5b3c6e121e835b2fb11f348cf361ca94f94fa02fee2d7", "bijective": false}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("symbol.tokens.decode known answer mismatch: {actual} != {expected}"),
        )
        .operation("symbol.tokens.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "triangle|○|△"}"##).expect("input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn symbol_tokens_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"table": {"source_id": "synthetic:original-v1", "profile_id": "example-token-table", "license_id": "original-fixture", "entries": [{"token": "△", "value": "A", "aliases": []}, {"token": "○", "value": "B", "aliases": []}]}, "boundaries": {"kind": "separated", "separator": "|"}, "output_separator": "|"}"##).expect("parameters");
    let p: Parameters = serde_json::from_value(parameters.clone()).expect("typed");
    let text = "A|B|A";
    let actual = serde_json::to_value(transform(text, &p, true)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "△|○|△", "records": [{"input_token": "A", "canonical_token": "△", "output_token": "△", "byte_offset": 0, "scalar_offset": 0, "alias_used": false}, {"input_token": "B", "canonical_token": "○", "output_token": "○", "byte_offset": 2, "scalar_offset": 2, "alias_used": false}, {"input_token": "A", "canonical_token": "△", "output_token": "△", "byte_offset": 4, "scalar_offset": 4, "alias_used": false}], "source_id": "synthetic:original-v1", "profile_id": "example-token-table", "license_id": "original-fixture", "definition_sha256": "703ea7ce8f303b5cf1bae7d837d904d025e859b9605adf60c9e0be4edbbf22cd", "bijective": true}"##).expect("answer");
    if actual != expected {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            format!("symbol.tokens.encode known answer mismatch: {actual} != {expected}"),
        )
        .operation("symbol.tokens.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "A|B|A"}"##).expect("input"),
        parameters,
        outputs: actual,
    })
}
