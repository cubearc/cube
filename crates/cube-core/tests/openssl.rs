use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn explicit_openssl_salted_bytes_execute() {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/openssl/vectors.json")).unwrap();
    let v = &cases[0];
    let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let result = execution::execute(
        "cipher.openssl.decrypt",
        1,
        &json!({"input":{"base64":cube_core::encoding::base64_encode(&input)}}),
        &v["parameters"],
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result, json!({"output":{"base64":""}}));
}
#[test]
fn cross_version_files_and_padding_success_are_not_authentication() {
    use cube_core::ciphers::openssl;
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/openssl/vectors.json")).unwrap();
    for v in cases {
        let s = openssl::Settings::parse(&v["parameters"].to_string()).unwrap();
        let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        assert_eq!(
            openssl::decrypt(&input, &s).unwrap(),
            plain,
            "{}",
            v["source"]
        );
        let result = execution::execute(
            "cipher.openssl.decrypt",
            1,
            &json!({"input":{"base64":cube_core::encoding::base64_encode(&input)}}),
            &v["parameters"],
            Budget::default(),
        )
        .unwrap();
        assert_eq!(
            result,
            json!({"output":{"base64":cube_core::encoding::base64_encode(&plain)}})
        );
        for n in [0, 1, 7, 8, 15, input.len() - 1] {
            let e = openssl::decrypt(&input[..n], &s).unwrap_err();
            assert_eq!(
                e.message,
                "OpenSSL legacy decryption failed (unauthenticated format)"
            );
        }
    }
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../src/ciphers/openssl/valid_pad_wrong_password.json"
    ))
    .unwrap();
    let s = openssl::Settings::parse(&v["parameters"].to_string()).unwrap();
    let plain = openssl::decrypt(
        &hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap(),
        &s,
    )
    .unwrap();
    assert_eq!(hex::encode(&plain), v["wrong_plaintext_hex"]);
    assert_ne!(hex::encode(&plain), v["original_plaintext_hex"]);
}
#[test]
fn exact_profiles_cost_limits_and_secrets_are_checked_before_work() {
    use cube_core::{ciphers::openssl, errors::ErrorCode};
    for profile in openssl::Profile::NAMES {
        let mut p = json!({"profile":profile,"password_hex":""});
        if profile.contains("pbkdf2") {
            p["iterations"] = json!(1);
        }
        let s = openssl::Settings::parse(&p.to_string()).unwrap();
        assert!(!format!("{s:?}").contains("password_hex"));
        assert_eq!(
            openssl::decrypt(&vec![0; openssl::MAX_INPUT + 1], &s)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
        if profile.contains("pbkdf2") {
            for n in [0, openssl::MAX_ITERATIONS + 1] {
                p["iterations"] = json!(n);
                assert!(openssl::Settings::parse(&p.to_string()).is_err());
            }
            p.as_object_mut().unwrap().remove("iterations");
            assert!(openssl::Settings::parse(&p.to_string()).is_err());
        } else {
            p["iterations"] = json!(1);
            assert!(openssl::Settings::parse(&p.to_string()).is_err());
        }
    }
    for document in [
        r#"{"profile":"aes128_cbc_pkcs7_evp_md5","password_hex":"SECRET"}"#,
        r#"{"profile":"aes128_cbc_pkcs7_evp_md5","password_hex":"","iterations":null}"#,
    ] {
        let e = openssl::Settings::parse(document).unwrap_err();
        assert!(!format!("{e:?}").contains("SECRET"));
    }
}
