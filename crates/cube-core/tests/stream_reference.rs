//! Published stream answers through Core's actual public executor.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn independent_answers_execute_both_directions_with_exact_profile_coverage() {
    use cube_core::ciphers::stream::reference::{self, Profile, Settings};
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/stream/reference/vectors.json")).unwrap();
    let profiles: std::collections::BTreeSet<_> = vectors
        .iter()
        .map(|v| v["parameters"]["profile"].as_str().unwrap())
        .collect();
    assert_eq!(profiles, Profile::NAMES.iter().copied().collect());
    for v in vectors {
        let p = &v["parameters"];
        let contract = cube_core::operations::require(reference::ENCRYPT)
            .unwrap()
            .contract()
            .unwrap();
        contract.validate_parameters(p).unwrap();
        let settings = Settings::parse(&p.to_string()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let ciphertext = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        for (id, input, expected) in [
            (reference::ENCRYPT, &plain, &ciphertext),
            (reference::DECRYPT, &ciphertext, &plain),
        ] {
            let invoke = |input: &[u8]| {
                let mut parameters = p.clone();
                if let Some(n) = p["data_bits"].as_u64() {
                    parameters["data_bits"] = json!(n.min((input.len() * 8) as u64));
                }
                execution::execute(
                    id,
                    1,
                    &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
                    &parameters,
                    Budget::default(),
                )
                .unwrap()
            };
            assert_eq!(
                invoke(input),
                json!({"output":{"base64":cube_core::encoding::base64_encode(expected)}}),
                "{p}"
            );
            assert_eq!(invoke(&[]), json!({"output":{"base64":""}}));
            let n = input.len().min(7);
            assert_eq!(
                invoke(&input[..n]),
                json!({"output":{"base64":cube_core::encoding::base64_encode(&expected[..n])}})
            );
        }
        assert_eq!(reference::encrypt(&plain, &settings).unwrap(), ciphertext);
        assert_eq!(reference::decrypt(&ciphertext, &settings).unwrap(), plain);
    }
}

#[test]
fn each_profile_rejects_invalid_material_in_schema_and_runtime() {
    use cube_core::ciphers::stream::reference::{self, Settings};
    let contract = cube_core::operations::require(reference::ENCRYPT)
        .unwrap()
        .contract()
        .unwrap();
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/stream/reference/vectors.json")).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for v in vectors {
        let p = &v["parameters"];
        if !seen.insert(p["profile"].as_str().unwrap().to_owned()) {
            continue;
        }
        for (field, value) in [
            ("key_hex", json!("")),
            ("key_hex", json!("00".repeat(257))),
            ("nonce_hex", json!("00".repeat(257))),
            ("nonce_hex", json!(null)),
            ("key_hex", json!("SECRET")),
            ("unrelated", json!(true)),
        ] {
            let mut changed = p.clone();
            changed[field] = value;
            assert!(
                contract.validate_parameters(&changed).is_err(),
                "{field}:{}",
                p["profile"]
            );
            let e = Settings::parse(&changed.to_string()).unwrap_err();
            assert!(!format!("{e:?}").contains("SECRET"));
        }
    }
}

#[test]
fn every_profile_preserves_its_declared_limit_and_refuses_one_over() {
    use cube_core::{
        ciphers::stream::reference::{self, Settings},
        errors::ErrorCode,
    };
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/stream/reference/vectors.json")).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for v in vectors {
        let mut p = v["parameters"].clone();
        let profile = p["profile"].as_str().unwrap().to_owned();
        if !seen.insert(profile.clone()) {
            continue;
        }
        let maximum = if profile.starts_with("a5") {
            228
        } else {
            reference::MAX_INPUT * 8
        };
        if p.get("data_bits").is_some() {
            p["data_bits"] = json!(maximum);
            let mut over = p.clone();
            over["data_bits"] = json!(maximum + 1);
            assert!(Settings::parse(&over.to_string()).is_err());
        }
        let mut data: Vec<u8> = (0..maximum.div_ceil(8))
            .map(|i| i.wrapping_mul(97) as u8)
            .collect();
        if maximum % 8 != 0 {
            *data.last_mut().unwrap() &= u8::MAX << (8 - maximum % 8);
        }

        let s = Settings::parse(&p.to_string()).unwrap();
        let c = reference::encrypt(&data, &s).unwrap();
        assert_eq!(
            reference::decrypt(&c, &s).unwrap(),
            data,
            "{}",
            p["profile"]
        );
        for op in [reference::encrypt, reference::decrypt] {
            assert_eq!(
                op(&vec![0; reference::MAX_INPUT + 1], &s).unwrap_err().code,
                ErrorCode::ResourceLimit
            );
        }
    }
}

#[test]
fn settings_boundaries_and_errors_preserve_private_material() {
    use cube_core::{
        ciphers::stream::reference::{self, Settings},
        errors::ErrorCode,
    };
    let good = json!({"profile":"rc4_rfc6229_v1","key_hex":"0102030405","nonce_hex":""});
    for (field, value) in [
        ("key_hex", json!("SECRET")),
        ("key_hex", json!("")),
        ("nonce_hex", json!(null)),
        ("nonce_hex", json!("00")),
        ("discard_bytes", json!(0)),
        ("unknown", json!("SECRET")),
        ("profile", json!("unknown")),
    ] {
        let mut p = good.clone();
        p[field] = value;
        let e = Settings::parse(&p.to_string()).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert!(!format!("{e:?}").contains("SECRET"));
    }
    assert!(
        Settings::parse(
            r#"{"profile":"rc4_rfc6229_v1","key_hex":"00","key_hex":"01","nonce_hex":""}"#
        )
        .is_err()
    );
    for value in [json!(null), json!(true), json!(-1), json!(65537)] {
        let mut p = good.clone();
        p["profile"] = json!("rc4_drop_v1");
        p["discard_bytes"] = value;
        assert!(Settings::parse(&p.to_string()).is_err());
    }
    for size in [1, 256] {
        let mut p = good.clone();
        p["key_hex"] = json!("42".repeat(size));
        let s = Settings::parse(&p.to_string()).unwrap();
        assert!(!format!("{s:?}").contains("4242"));
        let data = vec![0; reference::MAX_INPUT];
        let encrypted = reference::encrypt(&data, &s).unwrap();
        assert_eq!(reference::decrypt(&encrypted, &s).unwrap(), data);
        assert_eq!(
            reference::encrypt(&vec![0; reference::MAX_INPUT + 1], &s)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let mut document = good.to_string();
    document.push_str(&" ".repeat(reference::MAX_PARAMETERS - document.len()));
    assert!(Settings::parse(&document).is_ok());
    document.push(' ');
    assert_eq!(
        Settings::parse(&document).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn rc4_rfc6229_executes_through_the_public_registry() {
    let actual = execution::execute(
        "cipher.stream.encrypt",
        1,
        &json!({"input": {"base64": cube_core::encoding::base64_encode(&[0; 16])}}),
        &json!({"profile":"rc4_rfc6229_v1", "key_hex":"0102030405", "nonce_hex":""}),
        Budget::default(),
    )
    .expect("the named RC4 profile must be executable, not only catalogued");
    let expected = hex::decode("b2396305f03dc027ccc3524a0a1118a8").unwrap();
    assert_eq!(
        actual,
        json!({"output": {"base64": cube_core::encoding::base64_encode(&expected)}})
    );
}

#[test]
fn seal_last_position_is_complete_and_exhaustion_returns_no_output() {
    use cube_core::ciphers::stream::reference::{self, Settings};
    use cube_core::errors::ErrorCode;
    for profile in ["seal3_be_l32768_v1", "seal3_le_l32768_v1"] {
        for (nonce, available) in [("ffffffff", 4096), ("fffffffe", 8192)] {
            let settings = Settings::parse(
                &json!({"profile":profile,"key_hex":"00".repeat(20),"nonce_hex":nonce}).to_string(),
            )
            .unwrap();
            for function in [reference::encrypt, reference::decrypt] {
                assert_eq!(function(&[], &settings).unwrap(), Vec::<u8>::new());
                assert_eq!(
                    function(&vec![0; available], &settings).unwrap().len(),
                    available
                );
                assert_eq!(
                    function(&vec![0; available + 1], &settings)
                        .unwrap_err()
                        .code,
                    ErrorCode::ResourceLimit
                );
            }
        }
    }
}

#[test]
fn packed_bit_settings_and_padding_are_exact() {
    use cube_core::ciphers::stream::reference::{self, Settings};
    let vectors: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/stream/reference/vectors.json")).unwrap();
    let contract = cube_core::operations::require(reference::ENCRYPT)
        .unwrap()
        .contract()
        .unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for v in vectors {
        let mut p = v["parameters"].clone();
        let profile = p["profile"].as_str().unwrap().to_owned();
        if !seen.insert(profile.clone()) {
            continue;
        }
        if p.get("data_bits").is_none() {
            p["data_bits"] = json!(0);
            assert!(contract.validate_parameters(&p).is_err());
            assert!(Settings::parse(&p.to_string()).is_err());
            continue;
        }
        for replacement in [None, Some(json!(null)), Some(json!(524289))] {
            let mut bad = p.clone();
            if let Some(value) = replacement {
                bad["data_bits"] = value;
            } else {
                bad.as_object_mut().unwrap().remove("data_bits");
            }
            assert!(contract.validate_parameters(&bad).is_err());
            assert!(Settings::parse(&bad.to_string()).is_err());
        }
        p["data_bits"] = json!(1);
        let settings = Settings::parse(&p.to_string()).unwrap();
        let plain = if profile.starts_with("e0_") {
            [1]
        } else {
            [128]
        };
        let cipher = reference::encrypt(&plain, &settings).unwrap();
        assert_eq!(reference::decrypt(&cipher, &settings).unwrap(), plain);
        for function in [reference::encrypt, reference::decrypt] {
            assert!(function(&[0, 0], &settings).is_err());
            assert!(function(&[0x81], &settings).is_err());
        }
    }
    let good = json!({"profile":"mickey20_estream_msb_v1","key_hex":"00".repeat(10),"nonce_hex":"80","nonce_bits":1,"data_bits":0});
    assert!(Settings::parse(&good.to_string()).is_ok());
    for (name, value) in [
        ("nonce_hex", json!("81")),
        ("nonce_bits", json!(0)),
        ("nonce_bits", json!(81)),
        ("nonce_bits", json!(null)),
    ] {
        let mut p = good.clone();
        p[name] = value;
        assert!(Settings::parse(&p.to_string()).is_err());
    }
    for profile in [
        "a51_gsm_count22_be_key_msb_v1",
        "a52_gsm_count22_be_key_msb_v1",
        "a53_gsm_count22_be_key_msb_v1",
        "a54_gsm_count22_be_key_msb_v1",
    ] {
        let p = json!({"profile":profile,"key_hex":"00".repeat(if profile.starts_with("a54"){16}else{8}),"nonce_hex":"400000","data_bits":0});
        assert!(Settings::parse(&p.to_string()).is_err());
        assert!(contract.validate_parameters(&p).is_err());
    }
    let e0 = json!({"profile":"e0_core54_session128_lsb_v1","key_hex":"00".repeat(16),"nonce_hex":"00000000000000000004","data_bits":0});
    assert!(Settings::parse(&e0.to_string()).is_err());
    assert!(contract.validate_parameters(&e0).is_err());
}
