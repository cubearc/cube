use cube_core::{
    encoding::{self, base58check as check},
    errors::ErrorCode,
    pipeline::{Kind, Limits, Parameter, Plan, Step, Value},
};
use std::collections::BTreeMap;
#[test]
fn bitcoin_core_vectors_and_binary_round_trips_keep_version_and_leading_zeros() {
    // Bitcoin Core src/test/data/key_io_valid.json, public protocol vectors.
    for (text, version, payload) in [
        (
            "1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo",
            0,
            "a31c06bd463e3923bc1aadbde48b16976c080717",
        ),
        (
            "36j4NfKv6Akva9amjWrLG6MuSQym1GuEmm",
            5,
            "373b819a068f32b7a6b38b6b38729647cfde01c2",
        ),
    ] {
        let bytes = encoding::hex_decode(payload).unwrap();
        assert_eq!(check::encode(version, &bytes).unwrap(), text);
        let result = check::decode(text).unwrap();
        assert_eq!(result.version, version);
        assert_eq!(result.payload, bytes);
        assert!(result.checksum_verified);
    }
    for version in [0, 1, 128, 239, 255] {
        for bytes in [
            vec![],
            vec![0; 32],
            (0..=255).collect(),
            vec![255; check::MAX_PAYLOAD],
        ] {
            let text = check::encode(version, &bytes).unwrap();
            assert!(text.len() <= check::MAX_ENCODED);
            let decoded = check::decode(&text).unwrap();
            assert_eq!((decoded.version, decoded.payload), (version, bytes));
        }
    }
    assert_eq!(encoding::base58_encode(&[0, 0, 1]), "112");
    assert!(encoding::base58_decode("").unwrap().is_empty());
}
#[test]
fn malformed_corrupted_and_excessive_inputs_fail_with_non_input_diagnostics() {
    for text in ["", "1", "1111", "0OIl", "é", " 11111", "11111\n", "11111"] {
        let error = check::decode(text).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.operation, Some("encoding.base58check.decode"));
    }
    let valid = "1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo";
    let corrupted = format!("{}1", &valid[..valid.len() - 1]);
    assert!(
        check::decode(&corrupted)
            .unwrap_err()
            .message
            .contains("checksum")
    );
    assert_eq!(
        check::encode(0, &vec![0; 1025]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        check::decode(&"1".repeat(check::MAX_ENCODED + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        check::decode(&"1".repeat(check::MAX_FRAME + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let body = [0u8, 1, 2, 3];
    let mut wrong = body.to_vec();
    let first = cube_core::hashing::sha256(&body);
    let bad = cube_core::hashing::sha256(first.as_bytes());
    wrong.extend(encoding::hex_decode(&bad).unwrap().into_iter().take(4));
    assert!(check::decode(&encoding::base58_encode(&wrong)).is_err());
}
#[test]
fn encoder_adapter_is_typed_versioned_and_preflights_quadratic_work() {
    let step = Step {
        operation: "encoding.base58check.encode".into(),
        version: 1,
        parameters: BTreeMap::from([("version".into(), Parameter::Integer(0))]),
    };
    assert!(Plan::validate(Kind::Text, vec![step.clone()], Limits::default()).is_err());
    assert!(
        Plan::validate(
            Kind::Text,
            vec![Step::new("encoding.base58check.decode")],
            Limits::default()
        )
        .is_err()
    );
    let input = Value::Bytes {
        data: b"abc".to_vec(),
    };
    let result = Plan::validate(Kind::Bytes, vec![step.clone()], Limits::default())
        .unwrap()
        .execute(input.clone(), &[0], None)
        .unwrap();
    assert_eq!(
        result.output,
        Value::Text {
            data: check::encode(0, b"abc").unwrap()
        }
    );
    assert_eq!(result.intermediates[&0], result.output);
    assert!(result.work_bytes_upper_bound >= 64);
    let limits = Limits {
        max_work_bytes: 10,
        ..Default::default()
    };
    assert!(
        Plan::validate(Kind::Bytes, vec![step.clone()], limits)
            .unwrap()
            .execute(input, &[], None)
            .is_err()
    );
    assert!(
        Plan::validate(Kind::Bytes, vec![step], Limits::default())
            .unwrap()
            .execute(
                Value::Bytes {
                    data: vec![0; 1025]
                },
                &[],
                None
            )
            .is_err()
    );
}
