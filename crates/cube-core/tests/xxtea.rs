use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn corrected_block_tea_executes_whole_little_endian_word_sequences() {
    let output = execution::execute(
        "cipher.xxtea.encrypt_words",
        1,
        &json!({"input":{"base64":"AAAAAAAAAAA="}}),
        &json!({"key_hex":"00".repeat(16)}),
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        output,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&hex::decode("ab043705808c5d57").unwrap())}})
    );
}

#[test]
fn independent_raw_kernel_answers_and_exact_word_boundaries() {
    use cube_core::ciphers::xxtea::{self, Key};
    use cube_core::errors::ErrorCode;
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/xxtea/vectors.json")).unwrap();
    for row in rows {
        let key = Key::new(&hex::decode(row["key_hex"].as_str().unwrap()).unwrap()).unwrap();
        let plain = hex::decode(row["plaintext_hex"].as_str().unwrap()).unwrap();
        let encrypted = hex::decode(row["ciphertext_hex"].as_str().unwrap()).unwrap();
        assert_eq!(xxtea::encrypt(&plain, &key).unwrap(), encrypted);
        assert_eq!(xxtea::decrypt(&encrypted, &key).unwrap(), plain);
        for (op, input, expected) in [
            (xxtea::ENCRYPT, &plain, &encrypted),
            (xxtea::DECRYPT, &encrypted, &plain),
        ] {
            let actual = execution::execute(
                op,
                1,
                &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
                &json!({"key_hex":row["key_hex"]}),
                Budget::default(),
            )
            .unwrap();
            assert_eq!(
                actual,
                json!({"output":{"base64":cube_core::encoding::base64_encode(expected)}})
            );
        }
    }
    for len in [0, 1, 15, 17, 128] {
        assert_eq!(
            Key::new(&vec![0; len]).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    let key = Key::new(&[0; 16]).unwrap();
    assert_eq!(xxtea::encrypt(&[], &key).unwrap(), Vec::<u8>::new());
    for len in [1, 4, 7, 9, 11] {
        for f in [xxtea::encrypt, xxtea::decrypt] {
            assert_eq!(
                f(&vec![0; len], &key).unwrap_err().code,
                ErrorCode::InvalidInput
            );
        }
    }
    let input = vec![0x95; xxtea::MAX_INPUT];
    let ciphertext = xxtea::encrypt(&input, &key).unwrap();
    assert_eq!(xxtea::decrypt(&ciphertext, &key).unwrap(), input);
    for len in [xxtea::MAX_INPUT + 1, xxtea::MAX_INPUT + 4] {
        for f in [xxtea::encrypt, xxtea::decrypt] {
            assert_eq!(
                f(&vec![0; len], &key).unwrap_err().code,
                ErrorCode::ResourceLimit
            );
        }
    }
    // A sixteen-byte sequence is a single block, never two eight-byte blocks.
    let whole = xxtea::encrypt(&[0; 16], &key).unwrap();
    assert_ne!(whole, xxtea::encrypt(&[0; 8], &key).unwrap().repeat(2));
}

#[test]
fn private_key_parser_and_canonical_policies_remain_strict() {
    use cube_core::{ciphers::xxtea, errors::ErrorCode, operations};
    for p in [r#"{"key_hex":"SECRET"}"#.to_string(),json!({"key_hex":null}).to_string(),json!({"key_hex":"00".repeat(16),"extra":true}).to_string(),r#"{"key_hex":"00000000000000000000000000000000","key_hex":"00000000000000000000000000000000"}"#.to_string()] {
        let error=xxtea::parse_key(&p).unwrap_err();assert_eq!(error.code,ErrorCode::InvalidParameter);assert!(!format!("{error:?}").contains("SECRET"));
    }
    let mut p = json!({"key_hex":"00".repeat(16)}).to_string();
    p.push_str(&" ".repeat(xxtea::MAX_PARAMETERS - p.len()));
    assert!(!format!("{:?}", xxtea::parse_key(&p).unwrap()).contains(&"00".repeat(16)));
    p.push(' ');
    assert_eq!(
        xxtea::parse_key(&p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for op in [xxtea::ENCRYPT, xxtea::DECRYPT] {
        let contract = operations::require(op).unwrap().contract().unwrap();
        assert!(contract.pipeline_adapter.is_none());
        assert!(
            contract.policy.private_input
                && contract.policy.private_output
                && !contract.policy.automatic_capture
        );
        assert_eq!(contract.policy.sensitive_parameters, ["key_hex"]);
        assert!(
            contract
                .validate_parameters(&json!({"key_hex":"00".repeat(16)}))
                .is_ok()
        );
        assert!(
            contract
                .validate_parameters(&json!({"key_hex":"00".repeat(15)}))
                .is_err()
        );
    }
}
