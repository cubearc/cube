use cube_core::{ciphers::aes_blocks as aes, errors::ErrorCode};

#[test]
fn requested_software_fallback_is_actually_selected() {
    if std::env::var_os("CUBE_EXPECT_AES_SOFT").is_some() {
        assert!(!::aes::hardware_accelerated());
    }
}

#[test]
fn transport_debug_schemas_and_capture_policy_do_not_leak_keys() {
    use cube_core::operations;
    use serde_json::json;
    let key = aes::parse_key(r#"{"key_hex":"000102030405060708090a0b0c0d0e0f"}"#).unwrap();
    assert_eq!(key.as_ref(), (0u8..16).collect::<Vec<_>>());
    assert_eq!(format!("{key:?}"), "ParsedKey { [REDACTED] }");
    for descriptor in aes::OPERATIONS {
        let contract = operations::require(descriptor.id)
            .unwrap()
            .contract()
            .unwrap();
        assert!(contract.policy.private_input && contract.policy.private_output);
        assert!(!contract.policy.automatic_capture);
        assert!(contract.pipeline_adapter.is_none());
        for count in [16, 24, 32] {
            contract
                .validate_parameters(&json!({"key_hex":"00".repeat(count)}))
                .unwrap();
        }
        for parameters in [
            json!({"key_hex":"00"}),
            json!({"key_hex":"00".repeat(16), "extra":0}),
        ] {
            assert!(contract.validate_parameters(&parameters).is_err());
        }
        for length in [0, 16, 32, 48, 64] {
            let value = serde_json::to_value(operations::contracts::values::Bytes {
                data: vec![255; length],
            })
            .unwrap();
            contract.validate_inputs(&json!({"input":value})).unwrap();
        }
        for length in [1, 15, 17] {
            let value = serde_json::to_value(operations::contracts::values::Bytes {
                data: vec![0; length],
            })
            .unwrap();
            assert!(contract.validate_inputs(&json!({"input":value})).is_err());
        }
    }
    let mut document = String::from(r#"{"key_hex":"000102030405060708090a0b0c0d0e0f"}"#);
    document.push_str(&" ".repeat(aes::MAX_PARAMETERS - document.len()));
    aes::parse_key(&document).unwrap();
    document.push(' ');
    assert_eq!(
        aes::parse_key(&document).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn fips_197_all_key_sizes_and_exact_binary_blocks() {
    let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    for (key, expected) in [
        (
            "000102030405060708090a0b0c0d0e0f",
            "69c4e0d86a7b0430d8cdb78070b4c55a",
        ),
        (
            "000102030405060708090a0b0c0d0e0f1011121314151617",
            "dda97ca4864cdfe06eaf70a0ec0d7191",
        ),
        (
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "8ea2b7ca516745bfeafc49904b496089",
        ),
    ] {
        let key = hex::decode(key).unwrap();
        let expected = hex::decode(expected).unwrap();
        assert_eq!(aes::encrypt(&input, &key).unwrap(), expected);
        assert_eq!(aes::decrypt(&expected, &key).unwrap(), input);
        assert!(aes::encrypt(&[], &key).unwrap().is_empty());
        assert!(aes::decrypt(&[], &key).unwrap().is_empty());
        let doubled = [input.as_slice(), input.as_slice()].concat();
        assert_eq!(aes::encrypt(&doubled, &key).unwrap(), expected.repeat(2));
    }
}

#[test]
fn boundaries_fail_without_echoing_secret_material() {
    let key = [0x42; 16];
    for size in [0, 1, 15, 17, 23, 25, 31, 33] {
        let error = aes::encrypt(&[], &vec![0x42; size]).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidParameter);
        assert_eq!(error.parameter, Some("key_hex"));
        assert!(!format!("{error:?}").contains("424242"));
    }
    for size in [1, 15, 17, 31] {
        assert_eq!(
            aes::encrypt(&vec![0; size], &key).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            aes::decrypt(&vec![0; size], &key).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
    let data = vec![0; aes::MAX_INPUT];
    let encrypted = aes::encrypt(&data, &key).unwrap();
    assert_eq!(encrypted.len(), aes::MAX_INPUT);
    assert_eq!(aes::decrypt(&encrypted, &key).unwrap(), data);
    assert_eq!(
        aes::encrypt(&vec![0; aes::MAX_INPUT + 1], &key)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        aes::decrypt(&vec![0; aes::MAX_INPUT + 16], &key)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(aes::parse_key(r#"{"key_hex":"000102030405060708090a0b0c0d0e0f"}"#).is_ok());
    for document in [
        r#"{"key_hex":"secret"}"#,
        r#"{"key_hex":"000102030405060708090a0b0c0d0e0f","extra":true}"#,
        r#"{"key_hex":"secret","key_hex":"secret"}"#,
    ] {
        let error = aes::parse_key(document).unwrap_err();
        assert!(!format!("{error:?}").contains("secret"));
    }
}
