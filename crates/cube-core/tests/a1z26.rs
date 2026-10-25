use cube_core::{
    errors::ErrorCode,
    pipeline::{Kind, Limits, Plan, Step, Value},
    symbols::a1z26,
    text,
};
#[test]
fn canonical_alphabet_case_and_whitespace_round_trip() {
    assert_eq!(a1z26::encode("Hi Cube\r\n").unwrap(), "8-9 3-21-2-5\r\n");
    assert_eq!(a1z26::decode("8-9 3-21-2-5\r\n").unwrap(), "HI CUBE\r\n");
    for input in [
        "",
        "AZ",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        " \t\n\x0b\x0c\r",
        "A  Z\tB",
    ] {
        let encoded = a1z26::encode(input).unwrap();
        assert_eq!(a1z26::decode(&encoded).unwrap(), input);
        assert_eq!(
            a1z26::encode(&a1z26::decode(&encoded).unwrap()).unwrap(),
            encoded
        );
    }
    let max = "Z".repeat(text::MAX_INPUT);
    let encoded = a1z26::encode(&max).unwrap();
    assert_eq!(encoded.len(), a1z26::MAX_ENCODED - 1);
    assert_eq!(a1z26::decode(&encoded).unwrap(), max);
}
#[test]
fn malformed_tokens_unicode_and_limits_fail_with_context() {
    for input in [
        "0",
        "27",
        "-1",
        "+1",
        "01",
        "1--2",
        "1-",
        "999999999999999999",
        "1,2",
        "1.2",
        "é",
        "1-🧊",
    ] {
        assert!(a1z26::decode(input).is_err(), "{input}");
    }
    for input in ["A!", "é", "1", "A\u{a0}B"] {
        assert!(a1z26::encode(input).is_err());
    }
    assert_eq!(a1z26::decode("1-27").unwrap_err().offset, Some(2));
    assert_eq!(
        a1z26::encode(&"A".repeat(text::MAX_INPUT + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        a1z26::decode(&" ".repeat(a1z26::MAX_ENCODED + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn bounded_pipeline_adapters_compose_and_preflight_types_and_budgets() {
    let plan = Plan::validate(
        Kind::Text,
        vec![
            Step::new("symbol.a1z26.encode"),
            Step::new("symbol.a1z26.decode"),
        ],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        plan.execute(
            Value::Text {
                data: "Hi Cube".into()
            },
            &[0],
            None
        )
        .unwrap()
        .output,
        Value::Text {
            data: "HI CUBE".into()
        }
    );
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("symbol.a1z26.encode")],
            Limits::default()
        )
        .is_err()
    );
    let plan = Plan::validate(
        Kind::Text,
        vec![Step::new("symbol.a1z26.encode")],
        Limits {
            max_value_bytes: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(
        plan.execute(Value::Text { data: "AZ".into() }, &[], None)
            .is_err()
    );
}
