use cube_core::{
    errors::ErrorCode,
    pipeline::{self, Kind, Limits, Parameter, Plan, Step, Value},
};
use std::{error::Error as _, sync::atomic::AtomicBool};
fn bytes(data: &[u8]) -> Value {
    Value::Bytes {
        data: data.to_vec(),
    }
}
fn text(data: &str) -> Value {
    Value::Text { data: data.into() }
}
fn step(id: &str, parameters: &[(&str, Parameter)]) -> Step {
    Step {
        operation: id.into(),
        version: 1,
        parameters: parameters
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}
fn run(input: Value, steps: Vec<Step>) -> Value {
    Plan::validate(input.kind(), steps, Limits::default())
        .unwrap()
        .execute(input, &[], None)
        .unwrap()
        .output
}
#[test]
fn typed_values_round_trip_without_implicit_text_and_all_adapters_match_descriptors() {
    assert_eq!(pipeline::supported_adapters().len(), 53);
    for (encode, decode, known) in [
        (
            "encoding.base32.encode",
            "encoding.base32.decode",
            "AD7Q====",
        ),
        ("encoding.base58.encode", "encoding.base58.decode", "15Q"),
        ("encoding.base64.encode", "encoding.base64.decode", "AP8="),
        ("encoding.hex.encode", "encoding.hex.decode", "00ff"),
    ] {
        assert_eq!(run(bytes(&[0, 255]), vec![Step::new(encode)]), text(known));
        assert_eq!(
            run(bytes(&[0, 255]), vec![Step::new(encode), Step::new(decode)]),
            bytes(&[0, 255])
        );
    }
    assert_eq!(
        run(bytes(b"123456789"), vec![Step::new("hash.crc32")]),
        text("cbf43926")
    );
    assert_eq!(
        run(bytes(b""), vec![Step::new("hash.sha256")]),
        text("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
    assert_eq!(
        run(
            bytes(&[0, 255]),
            vec![step(
                "binary.xor",
                &[("key", Parameter::Bytes { data: vec![255] })]
            )]
        ),
        bytes(&[255, 0])
    );
    let input = "Hello é世界🧊\0\r\n";
    assert_eq!(
        run(
            text(input),
            vec![Step::new("text.utf8.encode"), Step::new("text.utf8.decode")]
        ),
        text(input)
    );
    assert_eq!(
        run(text("abcXYZ"), vec![Step::new("cipher.rot13")]),
        text("nopKLM")
    );
    assert_eq!(
        run(text("ABC xyz"), vec![Step::new("cipher.atbash")]),
        text("ZYX cba")
    );
    for (encode, decode, params, known) in [
        (
            "cipher.caesar.encode",
            "cipher.caesar.decode",
            vec![("shift", Parameter::Integer(3))],
            "DWWDFNDWGDZQ",
        ),
        (
            "cipher.vigenere.encode",
            "cipher.vigenere.decode",
            vec![("key", Parameter::Text("LEMON".into()))],
            "LXFOPVEFRNHR",
        ),
        (
            "cipher.affine.encode",
            "cipher.affine.decode",
            vec![("a", Parameter::Integer(5)), ("b", Parameter::Integer(8))],
            "IZZISGIZXIOV",
        ),
    ] {
        let encoded = run(text("ATTACKATDAWN"), vec![step(encode, &params)]);
        assert_eq!(encoded, text(known));
        assert_eq!(
            run(encoded, vec![step(decode, &params)]),
            text("ATTACKATDAWN")
        );
    }
    let params = [("key", Parameter::Text("B".into()))];
    assert_eq!(
        run(text("ABC xyz"), vec![step("cipher.beaufort", &params)]),
        text("BAZ edc")
    );
    let value = Value::ByteRange {
        offset: 3,
        length: Some(4),
    };
    assert_eq!(run(value.clone(), vec![]), value);
    for adapter in pipeline::supported_adapters() {
        assert_eq!(adapter.operation, adapter.descriptor.id);
        assert_eq!(adapter.version, 1);
        assert!(adapter.descriptor.deterministic);
        assert!(matches!(
            adapter.kinds().unwrap().0,
            Kind::Text | Kind::Bytes
        ));
    }
}
#[test]
fn preflight_rejects_unknown_versions_parameters_later_steps_and_adjacent_types() {
    let bad = Plan::validate(
        Kind::Text,
        vec![
            Step::new("encoding.hex.decode"),
            Step::new("file.range.extract"),
        ],
        Limits::default(),
    )
    .unwrap_err();
    assert_eq!(bad.step, Some(1));
    for (input, steps, index) in [
        (Kind::Bytes, vec![Step::new("cipher.rot13")], 0),
        (
            Kind::Bytes,
            vec![Step::new("encoding.hex.encode"), Step::new("hash.sha256")],
            1,
        ),
        (Kind::Text, vec![step("cipher.caesar.encode", &[])], 0),
        (
            Kind::Text,
            vec![step(
                "cipher.caesar.encode",
                &[("shift", Parameter::Text("3".into()))],
            )],
            0,
        ),
        (
            Kind::Text,
            vec![step(
                "cipher.vigenere.encode",
                &[("key", Parameter::Text("not a key".into()))],
            )],
            0,
        ),
        (
            Kind::Text,
            vec![step(
                "cipher.affine.encode",
                &[("a", Parameter::Integer(13)), ("b", Parameter::Integer(0))],
            )],
            0,
        ),
        (
            Kind::Bytes,
            vec![step(
                "binary.xor",
                &[("key", Parameter::Bytes { data: vec![] })],
            )],
            0,
        ),
        (Kind::Bytes, vec![Step::new("shell.exec")], 0),
    ] {
        let error = Plan::validate(input, steps, Limits::default()).unwrap_err();
        assert_eq!(error.step, Some(index));
    }
    let mut version = Step::new("encoding.hex.encode");
    version.version = 2;
    assert_eq!(
        Plan::validate(Kind::Bytes, vec![version], Limits::default())
            .unwrap_err()
            .parameter
            .as_deref(),
        Some("version")
    );
    let error = Plan::validate(
        Kind::Text,
        vec![step(
            "cipher.rot13",
            &[("unknown", Parameter::Boolean(true))],
        )],
        Limits::default(),
    )
    .unwrap_err();
    assert_eq!(error.parameter.as_deref(), Some("unknown"));
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("encoding.hex.encode"); 65],
            Limits::default()
        )
        .is_err()
    );
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![],
            Limits {
                max_value_bytes: pipeline::MAX_VALUE_BYTES + 1,
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn execution_limits_captures_cancellation_and_runtime_errors_are_contextual() {
    let plan = Plan::validate(
        Kind::Bytes,
        vec![
            Step::new("encoding.hex.encode"),
            Step::new("encoding.hex.decode"),
        ],
        Limits::default(),
    )
    .unwrap();
    let result = plan.execute(bytes(&[0, 255]), &[0], None).unwrap();
    assert_eq!(result.output, bytes(&[0, 255]));
    assert_eq!(result.intermediates.get(&0), Some(&text("00ff")));
    assert_eq!(
        plan.execute(bytes(&[0, 255]), &[], None)
            .unwrap()
            .intermediates
            .len(),
        0
    );
    for captures in [vec![2], vec![0, 0]] {
        assert!(plan.execute(bytes(&[]), &captures, None).is_err());
    }
    assert!(plan.execute(text("wrong type"), &[], None).is_err());
    let flag = AtomicBool::new(true);
    assert_eq!(
        plan.execute(bytes(&[]), &[], Some(&flag))
            .unwrap_err()
            .error
            .code,
        ErrorCode::Cancelled
    );
    let runtime = Plan::validate(
        Kind::Text,
        vec![
            Step::new("encoding.hex.decode"),
            Step::new("text.utf8.decode"),
        ],
        Limits::default(),
    )
    .unwrap()
    .execute(text("zz"), &[], None)
    .unwrap_err();
    assert_eq!(runtime.step, Some(0));
    assert_eq!(runtime.operation.as_deref(), Some("encoding.hex.decode"));
    assert!(runtime.source().unwrap().source().is_some());
    let one = vec![Step::new("encoding.hex.encode")];
    let limits = Limits {
        max_value_bytes: 6,
        max_work_bytes: 12,
        max_capture_bytes: 6,
        ..Default::default()
    };
    let exact = Plan::validate(Kind::Bytes, one.clone(), limits.clone())
        .unwrap()
        .execute(bytes(&[0, 1, 2]), &[0], None)
        .unwrap();
    assert_eq!(exact.work_bytes_upper_bound, 12);
    for limits in [
        Limits {
            max_value_bytes: 5,
            ..limits.clone()
        },
        Limits {
            max_work_bytes: 11,
            ..limits.clone()
        },
        Limits {
            max_capture_bytes: 5,
            ..limits.clone()
        },
    ] {
        let plan = Plan::validate(Kind::Bytes, one.clone(), limits).unwrap();
        assert_eq!(
            plan.execute(bytes(&[0, 1, 2]), &[0], None)
                .unwrap_err()
                .error
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let plan = Plan::validate(
        Kind::Bytes,
        vec![Step::new("encoding.base58.encode")],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        plan.execute(bytes(&vec![0; 4097]), &[], None)
            .unwrap_err()
            .error
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(
        pipeline::json::value(r#"{"type":"byte_range","offset":18446744073709551615,"length":1}"#)
            .is_err()
    );
    assert!(cube_core::text::from_utf8(&[0xff]).is_err());
    assert!(cube_core::text::to_utf8(&"x".repeat(1024 * 1024 + 1)).is_err());
}
#[test]
fn strict_transport_preserves_scalar_types_and_rejects_ambiguous_json() {
    let steps = pipeline::json::steps(
        r#"[{"operation":"binary.xor","version":1,"parameters":{"key":{"base64":"AP8="}}}]"#,
    )
    .unwrap();
    assert_eq!(run(bytes(&[0, 0]), steps), bytes(&[0, 255]));
    assert_eq!(
        pipeline::json::value(r#"{"type":"bytes","base64":"AP8="}"#).unwrap(),
        bytes(&[0, 255])
    );
    for invalid in [
        r#"{"x":1,"x":2}"#,
        r#"{"a":{"x":1,"x":2}}"#,
        "null garbage",
        r#"{"type":"bytes","base64":"AB=="}"#,
    ] {
        assert!(pipeline::json::value(invalid).is_err());
    }
    for invalid in [
        r#"[{"operation":"cipher.rot13"}]"#,
        r#"[{"operation":"cipher.rot13","version":1.0}]"#,
        r#"[{"operation":"cipher.rot13","version":1,"extra":true}]"#,
        r#"[{"operation":"cipher.caesar.encode","version":1,"parameters":{"shift":null}}]"#,
    ] {
        assert!(pipeline::json::steps(invalid).is_err());
    }
    let depth = format!("{}0{}", "[".repeat(17), "]".repeat(17));
    assert_eq!(
        pipeline::json::parse(&depth).unwrap_err().error.code,
        ErrorCode::ResourceLimit
    );
    let nodes = format!("[{}]", vec!["0"; 4097].join(","));
    assert_eq!(
        pipeline::json::parse(&nodes).unwrap_err().error.code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        pipeline::json::parse(&" ".repeat(pipeline::json::MAX_DOCUMENT_BYTES + 1))
            .unwrap_err()
            .error
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(pipeline::json::limits(r#"{"unknown":2}"#).is_err());
    let empty = Plan::validate(
        Kind::Text,
        vec![],
        Limits {
            max_steps: 0,
            max_value_bytes: 0,
            max_work_bytes: 0,
            max_capture_bytes: 0,
        },
    )
    .unwrap();
    assert_eq!(empty.execute(text(""), &[], None).unwrap().output, text(""));
}
