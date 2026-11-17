use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn nist_cbc_executes_exact_bytes_with_explicit_padding() {
    let input = hex::decode("6bc1bee22e409f96e93d7e117393172a").unwrap();
    let expected = hex::decode("7649abac8119b246cee98e9b12e9197d").unwrap();
    let result = execution::execute("cipher.aes.mode.encrypt", 1,
        &json!({"input":{"base64":cube_core::encoding::base64_encode(&input)}}),
        &json!({"mode":"cbc", "padding":"none", "key_hex":"2b7e151628aed2a6abf7158809cf4f3c", "iv_hex":"000102030405060708090a0b0c0d0e0f"}), Budget::default()).unwrap();
    assert_eq!(
        result,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&expected)}})
    );
}

#[test]
fn independent_nist_answers_and_partial_feedback_blocks() {
    use cube_core::ciphers::aes_modes as aes;
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/aes_modes/vectors.json")).unwrap();
    for v in vectors {
        let settings = aes::Settings::parse(&v["parameters"].to_string()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let cipher = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        assert_eq!(
            aes::encrypt(&plain, &settings).unwrap(),
            cipher,
            "{}",
            v["source"]
        );
        assert_eq!(aes::decrypt(&cipher, &settings).unwrap(), plain);
        for (op, input, expected) in [
            (aes::ENCRYPT, &plain, &cipher),
            (aes::DECRYPT, &cipher, &plain),
        ] {
            let result = execution::execute(
                op,
                1,
                &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
                &v["parameters"],
                Budget::default(),
            )
            .unwrap();
            assert_eq!(
                result,
                json!({"output":{"base64":cube_core::encoding::base64_encode(expected)}})
            );
        }
    }
}
#[test]
fn padding_key_sizes_counter_wrap_and_uniform_invalid_plaintext() {
    use cube_core::{ciphers::aes_modes as aes, errors::ErrorCode};
    for size in [16, 24, 32] {
        for mode in ["ecb", "cbc"] {
            let mut p = json!({"mode":mode,"key_hex":"42".repeat(size),"padding":"pkcs7"});
            if mode == "cbc" {
                p["iv_hex"] = json!("24".repeat(16));
            }
            let settings = aes::Settings::parse(&p.to_string()).unwrap();
            for len in [0, 1, 15, 16, 17, 31, 32] {
                let plain = vec![0xff; len];
                let encrypted = aes::encrypt(&plain, &settings).unwrap();
                assert_eq!(encrypted.len(), (len / 16 + 1) * 16);
                assert_eq!(aes::decrypt(&encrypted, &settings).unwrap(), plain);
            }
            p["padding"] = json!("none");
            let raw = aes::Settings::parse(&p.to_string()).unwrap();
            for last in [0, 17, 255] {
                let mut invalid = [0; 16];
                invalid[15] = last;
                let encrypted = aes::encrypt(&invalid, &raw).unwrap();
                assert_eq!(
                    aes::decrypt(&encrypted, &settings).unwrap_err().message,
                    "invalid AES mode ciphertext or padding"
                );
            }
            for pad in 2..=16 {
                for offset in 0..pad - 1 {
                    let mut invalid = vec![pad as u8; 16];
                    invalid[16 - pad + offset] ^= 1;
                    let encrypted = aes::encrypt(&invalid, &raw).unwrap();
                    assert!(aes::decrypt(&encrypted, &settings).is_err());
                }
            }
            for len in [0, 1, 15, 17] {
                assert!(aes::decrypt(&vec![0; len], &settings).is_err());
            }
            assert!(aes::encrypt(&[0; 1], &raw).is_err());
            assert!(aes::encrypt(&[], &raw).unwrap().is_empty());
        }
    }
    let p = json!({"mode":"ctr128_be","key_hex":"00".repeat(16),"iv_hex":"ff".repeat(16)});
    let settings = aes::Settings::parse(&p.to_string()).unwrap();
    assert_eq!(
        aes::encrypt(&[0; 16], &settings).unwrap(),
        cube_core::ciphers::aes_blocks::encrypt(&[255; 16], &[0; 16]).unwrap()
    );
    assert_eq!(
        aes::encrypt(&[0; 17], &settings).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert!(aes::encrypt(&[], &settings).unwrap().is_empty());
    assert_eq!(
        aes::encrypt(&vec![0; aes::MAX_INPUT + 1], &settings)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn exact_private_settings_and_schemas_reject_ambiguity() {
    use cube_core::ciphers::aes_modes as aes;
    let good =
        json!({"mode":"cbc","key_hex":"00".repeat(16),"iv_hex":"00".repeat(16),"padding":"none"});
    let contract = cube_core::operations::require(aes::ENCRYPT)
        .unwrap()
        .contract()
        .unwrap();
    assert!(
        contract.policy.private_input
            && contract.policy.private_output
            && !contract.policy.automatic_capture
    );
    let settings = aes::Settings::parse(&good.to_string()).unwrap();
    assert!(!format!("{settings:?}").contains("000000"));
    for name in ["mode", "key_hex", "iv_hex", "padding"] {
        let mut p = good.clone();
        p.as_object_mut().unwrap().remove(name);
        assert!(aes::Settings::parse(&p.to_string()).is_err());
        assert!(contract.validate_parameters(&p).is_err());
    }
    for change in [
        json!({"mode":"cfb8"}),
        json!({"mode":"cbc","padding":"auto"}),
        json!({"key_hex":"SECRET"}),
        json!({"iv_hex":"00"}),
        json!({"extra":"SECRET"}),
    ] {
        let mut p = good.clone();
        p.as_object_mut()
            .unwrap()
            .extend(change.as_object().unwrap().clone());
        let error = aes::Settings::parse(&p.to_string()).unwrap_err();
        assert!(!format!("{error:?}").contains("SECRET"));
        assert!(contract.validate_parameters(&p).is_err());
    }
    let mut doc = good.to_string();
    doc.push_str(&" ".repeat(aes::MAX_PARAMETERS - doc.len()));
    aes::Settings::parse(&doc).unwrap();
    doc.push(' ');
    assert!(aes::Settings::parse(&doc).is_err());
}

#[test]
fn exact_body_limit_and_padded_ciphertext_overhead_match_contracts() {
    use cube_core::ciphers::aes_modes as aes;
    let data = vec![0x42; aes::MAX_INPUT];
    let p =
        json!({"mode":"cbc","padding":"pkcs7","key_hex":"00".repeat(16),"iv_hex":"24".repeat(16)});
    let settings = aes::Settings::parse(&p.to_string()).unwrap();
    let ciphertext = aes::encrypt(&data, &settings).unwrap();
    assert_eq!(ciphertext.len(), aes::MAX_INPUT + 16);
    assert_eq!(aes::decrypt(&ciphertext, &settings).unwrap(), data);
    let enc = cube_core::operations::require(aes::ENCRYPT)
        .unwrap()
        .contract()
        .unwrap();
    let dec = cube_core::operations::require(aes::DECRYPT)
        .unwrap()
        .contract()
        .unwrap();
    assert_eq!(enc.limits.max_input_bytes, Some(aes::MAX_INPUT as u64));
    assert_eq!(
        dec.limits.max_input_bytes,
        Some((aes::MAX_INPUT + 16) as u64)
    );
    assert_eq!(dec.limits.max_output_bytes, Some(aes::MAX_INPUT as u64));
    assert!(aes::encrypt(&vec![0; aes::MAX_INPUT + 1], &settings).is_err());
    assert!(aes::decrypt(&vec![0; aes::MAX_INPUT + 17], &settings).is_err());
}

#[test]
fn cbc_ciphertext_stealing_profiles_accept_the_exact_single_block_minimum() {
    use cube_core::ciphers::aes_modes as aes;
    let plain = hex::decode("6bc1bee22e409f96e93d7e117393172a").unwrap();
    let expected = hex::decode("7649abac8119b246cee98e9b12e9197d").unwrap();
    for mode in ["cbc_cs1", "cbc_cs2", "cbc_cs3"] {
        let p = json!({"mode":mode,"key_hex":"2b7e151628aed2a6abf7158809cf4f3c","iv_hex":"000102030405060708090a0b0c0d0e0f"});
        let settings = aes::Settings::parse(&p.to_string()).unwrap();
        assert_eq!(aes::encrypt(&plain, &settings).unwrap(), expected);
        assert_eq!(aes::decrypt(&expected, &settings).unwrap(), plain);
    }
}

#[test]
fn storage_key_halves_sizes_tweaks_and_full_block_xex_are_explicit() {
    use cube_core::ciphers::aes_modes as aes;
    for bits in [128, 256] {
        let half = bits / 8;
        let key = hex::encode([vec![0x42; half], vec![0x24; half]].concat());
        for mode in [
            format!("xts_aes{bits}"),
            format!("xex_aes{bits}_ieee1619_full_blocks"),
        ] {
            let mut p =
                json!({"mode":mode,"key_hex":key,"tweak_hex":"000102030405060708090a0b0c0d0e0f"});
            let s = aes::Settings::parse(&p.to_string()).unwrap();
            let contract = cube_core::operations::require(aes::ENCRYPT)
                .unwrap()
                .contract()
                .unwrap();
            contract.validate_parameters(&p).unwrap();
            for n in [0, 1, 15] {
                assert!(aes::encrypt(&vec![0; n], &s).is_err());
                assert!(aes::decrypt(&vec![0; n], &s).is_err());
            }
            for n in [16, 17, 31, 32, 33, 48, 63, 64] {
                let data = vec![0xff; n];
                if mode.starts_with("xex") && !n.is_multiple_of(16) {
                    assert!(aes::encrypt(&data, &s).is_err());
                    continue;
                }
                let ciphertext = aes::encrypt(&data, &s).unwrap();
                assert_eq!(ciphertext.len(), n);
                assert_eq!(aes::decrypt(&ciphertext, &s).unwrap(), data);
            }
            for change in [
                json!({"iv_hex":"00".repeat(16)}),
                json!({"padding":"none"}),
                json!({"tweak_hex":"00"}),
                json!({"key_hex":"42".repeat(half*2)}),
            ] {
                let mut changed = p.clone();
                changed
                    .as_object_mut()
                    .unwrap()
                    .extend(change.as_object().unwrap().clone());
                assert!(aes::Settings::parse(&changed.to_string()).is_err());
            }
            p["key_hex"] = json!("00".repeat(half));
            assert!(aes::Settings::parse(&p.to_string()).is_err());
            assert!(contract.validate_parameters(&p).is_err());
        }
    }
    for mode in ["cbc_cs1", "cbc_cs2", "cbc_cs3"] {
        for size in [16, 24, 32] {
            let p = json!({"mode":mode,"key_hex":"42".repeat(size),"iv_hex":"24".repeat(16)});
            let s = aes::Settings::parse(&p.to_string()).unwrap();
            for n in [16, 17, 31, 32, 33, 48, 63, 64] {
                let data = vec![0x42; n];
                let encrypted = aes::encrypt(&data, &s).unwrap();
                assert_eq!(encrypted.len(), n);
                assert_eq!(aes::decrypt(&encrypted, &s).unwrap(), data);
            }
        }
    }
}

#[test]
fn storage_data_unit_accepts_exact_standard_ceiling_and_rejects_one_over() {
    use cube_core::{ciphers::aes_modes as aes, errors::ErrorCode};
    let data = vec![0x42; aes::MAX_INPUT];
    for (mode, half) in [("xts_aes128", 16), ("xex_aes256_ieee1619_full_blocks", 32)] {
        let p = json!({"mode":mode,"key_hex":"42".repeat(half)+&"24".repeat(half),"tweak_hex":"ff".repeat(16)});
        let s = aes::Settings::parse(&p.to_string()).unwrap();
        let encrypted = aes::encrypt(&data, &s).unwrap();
        assert_eq!(encrypted.len(), aes::MAX_INPUT);
        assert_eq!(aes::decrypt(&encrypted, &s).unwrap(), data);
        assert_eq!(
            aes::encrypt(&vec![0; aes::MAX_INPUT + 1], &s)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
}
