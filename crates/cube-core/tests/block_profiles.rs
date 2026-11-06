//! Independent reference vectors through Core's actual public executor.
use cube_core::{
    ciphers::block_profiles::{self, Profile, Settings},
    errors::ErrorCode,
    operations::{
        self,
        execution::{self, Budget},
    },
};
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/ciphers/block_profiles/vectors.json")).unwrap()
}
#[test]
fn every_reference_vector_executes_both_directions_and_preserves_blocks() {
    let mut profiles = std::collections::BTreeSet::new();
    for vector in vectors() {
        let profile = vector["profile"].as_str().unwrap();
        profiles.insert(profile.to_string());
        let key = hex::decode(vector["key_hex"].as_str().unwrap()).unwrap();
        let bits = vector["effective_key_bits"].as_u64().map(|n| n as u16);
        let tweak = vector["tweak_hex"]
            .as_str()
            .map(|t| hex::decode(t).unwrap());
        let settings = Settings::new_with_tweak(
            Profile::parse(profile).unwrap(),
            &key,
            bits,
            tweak.as_deref(),
        )
        .unwrap();
        let mut parameters = json!({"profile":profile,"key_hex":vector["key_hex"]});
        if let Some(bits) = bits {
            parameters["effective_key_bits"] = json!(bits);
        }
        if let Some(tweak) = vector.get("tweak_hex") {
            parameters["tweak_hex"] = tweak.clone();
        }
        let plain = hex::decode(vector["plaintext_hex"].as_str().unwrap()).unwrap();
        let encrypted = hex::decode(vector["ciphertext_hex"].as_str().unwrap()).unwrap();
        for (operation, input, expected) in [
            (block_profiles::ENCRYPT, &plain, &encrypted),
            (block_profiles::DECRYPT, &encrypted, &plain),
        ] {
            let invoke = |input: &[u8]| {
                execution::execute(
                    operation,
                    1,
                    &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
                    &parameters,
                    Budget::default(),
                )
                .unwrap_or_else(|e| panic!("{profile} {operation}: {e:?}"))
            };
            assert_eq!(
                invoke(input),
                json!({"output":{"base64":cube_core::encoding::base64_encode(expected)}}),
                "{profile} {operation}"
            );
            assert_eq!(
                invoke(&input.repeat(3)),
                json!({"output":{"base64":cube_core::encoding::base64_encode(&expected.repeat(3))}})
            );
            assert_eq!(invoke(&[]), json!({"output":{"base64":""}}));
        }
        assert_eq!(
            block_profiles::encrypt(&plain, &settings).unwrap(),
            encrypted
        );
        assert_eq!(
            block_profiles::decrypt(&encrypted, &settings).unwrap(),
            plain
        );
    }
    assert_eq!(profiles.len(), Profile::ALL.len());
}
#[test]
fn invalid_keys_and_profile_settings_fail_without_private_excerpts() {
    for profile in Profile::ALL {
        for length in [0, 129] {
            assert_eq!(
                Settings::new(*profile, &vec![0x42; length], None)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidParameter
            );
        }
    }
    let good = json!({"profile":"des_fips46_v1","key_hex":"0113b970fd34f2ce"});
    for (field, value) in [
        ("profile", json!("unknown")),
        ("key_hex", json!("SECRET")),
        ("key_hex", json!("00")),
        ("extra", json!("SECRET")),
        ("effective_key_bits", json!(64)),
        ("effective_key_bits", Value::Null),
    ] {
        let mut p = good.clone();
        p[field] = value;
        let error = Settings::parse(&p.to_string()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidParameter);
        assert!(!format!("{error:?}").contains("SECRET"));
    }
    assert!(Settings::parse(r#"{"profile":"des_fips46_v1","key_hex":"0113b970fd34f2ce","key_hex":"0113b970fd34f2ce"}"#).is_err());
    for value in [json!(0), json!(1025), json!(true), Value::Null] {
        assert!(
            Settings::parse(
                &json!({"profile":"rc2_rfc2268_v1","key_hex":"88","effective_key_bits":value})
                    .to_string()
            )
            .is_err()
        );
    }
    assert!(Settings::parse(r#"{"profile":"rc2_rfc2268_v1","key_hex":"88"}"#).is_err());
    for size in [1, 3, 57, 128] {
        assert!(Settings::new(Profile::BlowfishBe, &vec![0; size], None).is_err());
    }
    for size in [4, 56] {
        assert!(Settings::new(Profile::BlowfishBe, &vec![0; size], None).is_ok());
    }
    for bits in [1, 1024] {
        let p = Settings::new(Profile::Rc2, &[0; 128], Some(bits)).unwrap();
        assert_eq!(block_profiles::encrypt(&[0; 8], &p).unwrap().len(), 8);
    }
}
#[test]
fn exact_limits_whole_blocks_and_private_contracts_are_preserved() {
    let settings = Settings::new(Profile::Idea, &[0; 16], None).unwrap();
    for length in [1, 7, 9, 15] {
        assert_eq!(
            block_profiles::encrypt(&vec![0; length], &settings)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
    }
    let data = vec![0; block_profiles::MAX_INPUT];
    let encrypted = block_profiles::encrypt(&data, &settings).unwrap();
    assert_eq!(encrypted.len(), data.len());
    assert_eq!(
        block_profiles::decrypt(&encrypted, &settings).unwrap(),
        data
    );
    for extra in [1, 8] {
        assert_eq!(
            block_profiles::encrypt(&vec![0; block_profiles::MAX_INPUT + extra], &settings)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    let mut document = r#"{"profile":"des_fips46_v1","key_hex":"0113b970fd34f2ce"}"#.to_string();
    document.push_str(&" ".repeat(block_profiles::MAX_PARAMETERS - document.len()));
    assert!(!format!("{:?}", Settings::parse(&document).unwrap()).contains("0113b970"));
    document.push(' ');
    assert_eq!(
        Settings::parse(&document).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for operation in [block_profiles::ENCRYPT, block_profiles::DECRYPT] {
        let c = operations::require(operation).unwrap().contract().unwrap();
        assert!(c.policy.private_input && c.policy.private_output && !c.policy.automatic_capture);
        assert!(c.pipeline_adapter.is_none());
        assert_eq!(c.policy.sensitive_parameters, ["key_hex", "tweak_hex"]);
        for v in vectors() {
            let mut p = json!({"profile":v["profile"],"key_hex":v["key_hex"]});
            if let Some(bits) = v.get("effective_key_bits") {
                p["effective_key_bits"] = bits.clone();
            }
            if let Some(tweak) = v.get("tweak_hex") {
                p["tweak_hex"] = tweak.clone();
            }
            c.validate_parameters(&p).unwrap();
        }
        for p in [
            json!({"profile":"des_fips46_v1","key_hex":"00"}),
            json!({"profile":"des_fips46_v1","key_hex":"00".repeat(8),"effective_key_bits":64}),
            json!({"profile":"rc2_rfc2268_v1","key_hex":"88"}),
            json!({"profile":"rc2_rfc2268_v1","key_hex":"88","effective_key_bits":null}),
        ] {
            assert!(c.validate_parameters(&p).is_err());
        }
        for length in 0usize..=32 {
            let input =
                json!({"input":{"base64":cube_core::encoding::base64_encode(&vec![255;length])}});
            assert!(c.validate_inputs(&input).is_ok(), "{length}");
        }
    }
}

#[test]
fn selected_gost_sboxes_use_the_explicit_little_endian_convention() {
    let parameters = json!({"profile":"gost28147_test_le_v1","key_hex":"75713134b60fec45a607bb83aa3746af4ff99da6d1b53b5b1b402a1baa030d1b"});
    let input = hex::decode("1122334455667788").unwrap();
    let actual = execution::execute(
        block_profiles::ENCRYPT,
        1,
        &json!({"input":{"base64":cube_core::encoding::base64_encode(&input)}}),
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        actual,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&hex::decode("03251e14f9d28acb").unwrap())}})
    );
}

#[test]
fn variable_width_alignment_key_gaps_and_reference_limits_are_enforced() {
    for profile in Profile::ALL {
        let vector = vectors()
            .into_iter()
            .find(|v| Profile::parse(v["profile"].as_str().unwrap()).unwrap() == *profile)
            .unwrap();
        let key = hex::decode(vector["key_hex"].as_str().unwrap()).unwrap();
        let bits = vector["effective_key_bits"].as_u64().map(|n| n as u16);
        let tweak = vector["tweak_hex"]
            .as_str()
            .map(|t| hex::decode(t).unwrap());
        let settings = Settings::new_with_tweak(*profile, &key, bits, tweak.as_deref()).unwrap();
        for length in [profile.block_bytes() - 1, profile.block_bytes() + 1] {
            assert_eq!(
                block_profiles::encrypt(&vec![0; length], &settings)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidInput
            );
            assert_eq!(
                block_profiles::decrypt(&vec![0; length], &settings)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidInput
            );
        }
        assert_eq!(
            block_profiles::encrypt(&vec![0; profile.max_input_bytes() + 1], &settings)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
    for (profile, key_size) in [
        (Profile::Rijndael128, 17),
        (Profile::Camellia, 20),
        (Profile::Cast256, 18),
        (Profile::Shacal2, 18),
        (Profile::Speck48_72, 10),
    ] {
        assert!(Settings::new(profile, &vec![0; key_size], None).is_err());
    }
    // Original Rijndael's five key widths for each of five block widths have
    // independently expected bytes, including the three non-AES gaps.
    let tuples: std::collections::BTreeSet<_> = vectors()
        .into_iter()
        .filter(|v| v["profile"].as_str().unwrap().starts_with("rijndael_"))
        .map(|v| {
            (
                v["profile"].as_str().unwrap().to_string(),
                v["key_hex"].as_str().unwrap().len() / 2,
            )
        })
        .collect();
    assert_eq!(tuples.len(), 25);
    let settings = Settings::new(Profile::Present80, &[0; 10], None).unwrap();
    let exact = vec![0; 64 * 1024];
    let encrypted = block_profiles::encrypt(&exact, &settings).unwrap();
    assert_eq!(
        block_profiles::decrypt(&encrypted, &settings).unwrap(),
        exact
    );
    let settings = Settings::new(Profile::Rijndael224, &[0; 20], None).unwrap();
    let aligned = vec![0; (64 * 1024 / 28) * 28];
    let encrypted = block_profiles::encrypt(&aligned, &settings).unwrap();
    assert_eq!(
        block_profiles::decrypt(&encrypted, &settings).unwrap(),
        aligned
    );
}
