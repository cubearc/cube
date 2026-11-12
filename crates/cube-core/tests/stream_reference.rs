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
        let settings = Settings::parse(&p.to_string()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        let ciphertext = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        for (id, input, expected) in [
            (reference::ENCRYPT, &plain, &ciphertext),
            (reference::DECRYPT, &ciphertext, &plain),
        ] {
            let invoke = |input: &[u8]| {
                execution::execute(
                    id,
                    1,
                    &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
                    p,
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
