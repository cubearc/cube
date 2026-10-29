use cube_core::operations;
use serde_json::json;

#[test]
fn joins_preserve_order_empty_operands_unicode_and_binary_content() {
    use cube_core::composition::{concat_bytes, concat_text};
    assert_eq!(
        concat_text("e\u{301}\0", "世界\r\n", " | ").unwrap(),
        "e\u{301}\0 | 世界\r\n"
    );
    assert_eq!(concat_text("", "", ":").unwrap(), ":");
    assert_eq!(concat_text("B", "A", "").unwrap(), "BA");
    assert_eq!(
        concat_bytes(&[0, 255], &[128, 10]).unwrap(),
        [0, 255, 128, 10]
    );
    assert!(concat_bytes(&[], &[]).unwrap().is_empty());
}

#[test]
fn the_combined_bound_includes_the_separator_and_is_checked_before_allocation() {
    use cube_core::{composition::*, errors::ErrorCode};
    let prefix = "x".repeat(MAX_BYTES - 1);
    assert_eq!(
        concat_text(&prefix, "", "é").unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(concat_text(&prefix, "y", "").unwrap().len(), MAX_BYTES);
    assert_eq!(
        concat_bytes(prefix.as_bytes(), &[255]).unwrap().len(),
        MAX_BYTES
    );
    let error = concat_text(&prefix, "y", "z").unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.operation, Some("text.concat"));
    let error = concat_bytes(&vec![0; MAX_BYTES], &[1]).unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert_eq!(error.operation, Some("bytes.concat"));
}

#[test]
fn ordered_joins_have_two_required_named_operands_and_no_implicit_coercion() {
    for (id, kind, valid) in [
        ("text.concat", "text", json!({"left":"é", "right":"A"})),
        (
            "bytes.concat",
            "bytes",
            json!({"left":{"base64":"AP8="}, "right":{"base64":"gA=="}}),
        ),
    ] {
        let contract = operations::require(id).unwrap().contract().unwrap();
        assert_eq!(contract.operation_version, 1);
        assert_eq!(
            contract
                .inputs
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["left", "right"]
        );
        assert!(
            contract
                .inputs
                .iter()
                .all(|p| p.required && p.semantic_type == kind)
        );
        assert_eq!(contract.outputs[0].semantic_type, kind);
        contract.validate_inputs(&valid).unwrap();
        assert!(
            contract
                .validate_inputs(&json!({"left":42, "right":null}))
                .is_err()
        );
        assert!(contract.validate_inputs(&json!({"left":"x"})).is_err());
        assert!(!contract.policy.grants_filesystem_access);
        assert!(!contract.policy.grants_network_access);
    }
}
