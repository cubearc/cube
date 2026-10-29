use cube_core::operations;
use cube_core::{composition::structured::*, errors::ErrorCode};
use serde_json::{Value, json};

#[test]
fn structured_selection_and_numeric_composition_have_real_named_contracts() {
    for id in [
        "json.parse",
        "json.select",
        "json.text",
        "text.integer",
        "number.maximum",
        "number.add",
        "image.dimensions",
    ] {
        let contract = operations::require(id)
            .unwrap_or_else(|_| panic!("missing {id}"))
            .contract()
            .unwrap();
        assert_eq!(contract.operation_version, 1);
        assert!(!contract.inputs.is_empty());
        assert_eq!(contract.outputs.len(), 1);
        assert!(!contract.policy.grants_filesystem_access);
        assert!(!contract.policy.grants_network_access);
    }
}

#[test]
fn fields_preserve_types_escaped_keys_null_arrays_and_unicode_without_coercion() {
    assert_eq!(
        select_field(&parse_json("{\"ratio\":1.5}").unwrap(), "/ratio").unwrap(),
        json!(1.5)
    );
    let value=parse_json("{\"a/b\":{\"~key\":[null,{\"size\":7}]},\"text\":\"é\\u0000世界\\r\\n\",\"empty\":\"\",\"flag\":true}").unwrap();
    assert_eq!(
        select_field(&value, "/a~1b/~0key/1/size").unwrap(),
        json!(7)
    );
    assert_eq!(select_field(&value, "/a~1b/~0key/0").unwrap(), Value::Null);
    assert_eq!(select_field(&value, "").unwrap(), value);
    assert!(select_field(&value, "/a~1b").unwrap().is_object());
    assert!(select_field(&value, "/a~1b/~0key").unwrap().is_array());
    assert_eq!(
        json_text(&select_field(&value, "/text").unwrap()).unwrap(),
        "é\0世界\r\n"
    );
    assert_eq!(
        json_text(&select_field(&value, "/empty").unwrap()).unwrap(),
        ""
    );
    assert_eq!(select_field(&value, "/flag").unwrap(), json!(true));
    for pointer in [
        "/missing",
        "/a~1b/~0key/01",
        "/a~1b/~0key/-",
        "/a~1b/~0key/2",
    ] {
        assert_eq!(
            select_field(&value, pointer).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
    for pointer in ["size", "/bad~", "/bad~2"] {
        assert_eq!(
            select_field(&value, pointer).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    for value in [json!(7), Value::Null, json!([]), json!({}), json!(true)] {
        let error = json_text(&value).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.operation, Some("json.text"));
    }
}

#[test]
fn json_rejects_duplicates_trailing_data_and_exact_byte_depth_node_pointer_overruns() {
    for value in [
        json!(9007199254740992u64),
        json!(-9007199254740992i64),
        json!({"large":1e100}),
    ] {
        assert_eq!(
            select_field(&value, "").unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    let huge_key = "x".repeat(MAX_DOCUMENT_BYTES + 1);
    assert_eq!(
        select_field(&json!({huge_key:null}), "").unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for invalid in [
        "{\"key\":1,\"key\":2}",
        "{} null",
        "NaN",
        "[Infinity]",
        "{bad}",
    ] {
        let e = parse_json(invalid).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert_eq!(e.operation, Some("json.parse"));
    }
    let exact = Value::String("x".repeat(MAX_DOCUMENT_BYTES - 2));
    assert_eq!(select_field(&exact, "").unwrap(), exact);
    assert!(parse_json(&exact.to_string()).is_ok());
    assert_eq!(
        select_field(&Value::String("x".repeat(MAX_DOCUMENT_BYTES - 1)), "")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        parse_json(&" ".repeat(MAX_DOCUMENT_BYTES + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let exact = json!(vec![Value::Null; MAX_NODES - 1]);
    assert_eq!(select_field(&exact, "").unwrap(), exact);
    assert_eq!(
        select_field(&json!(vec![Value::Null; MAX_NODES]), "")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let mut nested = Value::Null;
    for _ in 0..MAX_DEPTH {
        nested = json!([nested]);
    }
    assert_eq!(select_field(&nested, "").unwrap(), nested);
    assert_eq!(
        select_field(&json!([nested]), "").unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let key = "x".repeat(MAX_POINTER_BYTES - 1);
    let value = json!({key.clone():17});
    assert_eq!(select_field(&value, &format!("/{key}")).unwrap(), json!(17));
    assert_eq!(
        select_field(&value, &format!("/{key}x")).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn exact_integer_maximum_and_addition_reproduce_dimension_and_branch_answers() {
    let image = cube_core::image::Image::from_bytes(
        2,
        3,
        cube_core::image::ColorModel::Rgb,
        &[0; 18],
        &[],
        &[],
    )
    .unwrap();
    let dimensions = serde_json::to_value(cube_core::image::dimensions::inspect(&image)).unwrap();
    assert_eq!(dimensions, json!({"dimensions":[2,3]}));
    let selected = select_field(&dimensions, "/dimensions").unwrap();
    let values: Vec<i64> = serde_json::from_value(selected).unwrap();
    assert_eq!(
        add(maximum(&values).unwrap(), parse_integer("7").unwrap()).unwrap(),
        10
    );
    assert_eq!(maximum(&[-9, -3, -7]).unwrap(), -3);
    assert_eq!(parse_integer("-0").unwrap(), 0);
    assert_eq!(
        parse_integer(&MAX_INTEGER.to_string()).unwrap(),
        MAX_INTEGER
    );
    assert_eq!(
        parse_integer(&(-MAX_INTEGER).to_string()).unwrap(),
        -MAX_INTEGER
    );
    assert_eq!(add(MAX_INTEGER, 0).unwrap(), MAX_INTEGER);
    assert_eq!(add(-MAX_INTEGER, MAX_INTEGER).unwrap(), 0);
    assert_eq!(maximum(&vec![1; MAX_NODES]).unwrap(), 1);
    for invalid in ["", " ", " 7", "7 ", "+7", "01", "-01", "1.0", "1e3", "１２"] {
        assert_eq!(
            parse_integer(invalid).unwrap_err().code,
            ErrorCode::InvalidInput,
            "{invalid}"
        );
    }
    for text in [
        "9007199254740992",
        "-9007199254740992",
        "999999999999999999",
    ] {
        assert_eq!(
            parse_integer(text).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert_eq!(maximum(&[]).unwrap_err().code, ErrorCode::InvalidInput);
    for result in [
        maximum(&[MAX_INTEGER + 1]),
        maximum(&vec![1; MAX_NODES + 1]),
        add(MAX_INTEGER, 1),
        add(-MAX_INTEGER, -1),
        add(i64::MAX, 0),
    ] {
        assert_eq!(result.unwrap_err().code, ErrorCode::ResourceLimit);
    }
}
