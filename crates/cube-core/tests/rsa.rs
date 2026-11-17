use cube_core::{
    asymmetric::{keys, rsa},
    errors::ErrorCode,
    operations::execution::{self, Budget},
};
use serde_json::{Value, json};
fn bytes(v: &Value, name: &str) -> Vec<u8> {
    hex::decode(v[name].as_str().unwrap()).unwrap()
}
fn interop() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/asymmetric/rsa/interoperability.json")).unwrap()
}
#[test]
fn independent_wycheproof_oaep_ciphertext_checks_its_label_before_output() {
    let v: Value =
        serde_json::from_str(include_str!("../src/asymmetric/rsa/fixture.json")).unwrap();
    let ct = bytes(&v, "ciphertext_hex");
    let key = bytes(&v, "key_hex");
    let output=execution::execute(rsa::DECRYPT,1,&json!({"input":{"base64":cube_core::encoding::base64_encode(&ct)},"key":{"base64":cube_core::encoding::base64_encode(&key)}}),&v["parameters"],Budget::default()).unwrap();
    assert_eq!(
        output,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&bytes(&v,"plaintext_hex"))}})
    );
}
#[test]
fn all_selected_wycheproof_oaep_padding_and_label_classes_are_uniform() {
    let vectors: Value =
        serde_json::from_str(include_str!("../src/asymmetric/rsa/wycheproof_oaep.json")).unwrap();
    for group in vectors["testGroups"].as_array().unwrap() {
        let key = bytes(group, "privateKeyPkcs8");
        for test in group["tests"].as_array().unwrap() {
            let p = json!({"profile":"rsa_oaep_sha256_mgf1_sha256_cfrg10","key_format":"pkcs8_der","label_hex":test["label"]});
            let s = rsa::DecryptionSettings::parse(&p.to_string(), &key).unwrap();
            let result = rsa::decrypt(&bytes(test, "ct"), &s);
            if test["result"] == "valid" || test["tcId"] == 23 {
                assert_eq!(
                    hex::encode(result.unwrap()),
                    test["msg"],
                    "{}",
                    test["tcId"]
                )
            } else {
                assert_eq!(
                    result.unwrap_err().message,
                    "RSA operation failed",
                    "{}",
                    test["tcId"]
                )
            }
        }
    }
}
#[test]
fn legacy_invalid_padding_matches_the_independent_cfrg10_implicit_rejection_oracle() {
    let vectors: Value = serde_json::from_str(include_str!(
        "../src/asymmetric/rsa/implicit_rejection_vectors.json"
    ))
    .unwrap();
    for group in vectors["groups"].as_array().unwrap() {
        let settings =
            rsa::LegacySettings::parse(&group["parameters"].to_string(), &bytes(group, "key_hex"))
                .unwrap();
        for test in group["cases"].as_array().unwrap() {
            let ct = bytes(test, "ciphertext_hex");
            let result = rsa::legacy_decrypt(&ct, &settings);
            if test["expect"] == "public_error" {
                assert_eq!(result.unwrap_err().message, "RSA operation failed");
            } else {
                let actual = result.unwrap();
                assert_eq!(hex::encode(&actual), test["output_hex"], "{}", test["tcId"]);
                if test["expect"] == "synthetic" {
                    assert_eq!(rsa::legacy_decrypt(&ct, &settings).unwrap(), actual);
                }
            }
        }
    }
}
#[test]
fn independent_openssl_der_pem_crlf_and_modulus_boundaries_interoperate() {
    for v in interop() {
        let public = rsa::EncryptionSettings::parse(
            &v["public_parameters"].to_string(),
            &bytes(&v, "public_key_hex"),
        )
        .unwrap();
        let private = rsa::DecryptionSettings::parse(
            &v["private_parameters"].to_string(),
            &bytes(&v, "private_key_hex"),
        )
        .unwrap();
        let ct = bytes(&v, "ciphertext_hex");
        let expected = bytes(&v, "plaintext_hex");
        assert_eq!(rsa::decrypt(&ct, &private).unwrap(), expected);
        let first = rsa::encrypt(&expected, &public).unwrap();
        let second = rsa::encrypt(&expected, &public).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            first.len(),
            v["bits"].as_u64().unwrap().div_ceil(8) as usize
        );
        assert_eq!(rsa::decrypt(&first, &private).unwrap(), expected);
        assert!(!format!("{private:?}").contains(v["private_key_hex"].as_str().unwrap()));
        let maximum = ct.len() - 66;
        let longest = vec![0x42; maximum];
        let encrypted = rsa::encrypt(&longest, &public).unwrap();
        assert_eq!(rsa::decrypt(&encrypted, &private).unwrap(), longest);
        assert_eq!(
            rsa::encrypt(&vec![0; maximum + 1], &public)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
}
#[test]
fn canonical_key_limits_labels_wrong_algorithms_and_errors_are_explicit() {
    let v = &interop()[0];
    let private = bytes(v, "private_key_hex");
    let public = bytes(v, "public_key_hex");
    for key in [&private, &public] {
        let mut trailing = key.clone();
        trailing.push(0);
        assert!(keys::rsa_public(&trailing, keys::PublicFormat::SpkiDer).is_err());
        assert!(keys::rsa_private(&trailing, keys::PrivateFormat::Pkcs8Der).is_err());
    }
    let mut bad_crt = private.clone();
    let n = bad_crt.len();
    bad_crt[n - 1] ^= 1;
    assert!(keys::rsa_private(&bad_crt, keys::PrivateFormat::Pkcs8Der).is_err());
    assert!(keys::rsa_public(&private, keys::PublicFormat::SpkiDer).is_err());
    assert!(keys::rsa_private(&public, keys::PrivateFormat::Pkcs8Der).is_err());
    for n in [65536, 65537] {
        assert_eq!(
            keys::rsa_private(&vec![0; n], keys::PrivateFormat::Pkcs8Der)
                .unwrap_err()
                .code,
            if n == 65536 {
                ErrorCode::InvalidParameter
            } else {
                ErrorCode::ResourceLimit
            }
        );
    }
    let private_settings =
        rsa::DecryptionSettings::parse(&v["private_parameters"].to_string(), &private).unwrap();
    for length in [0, 1, 255, 257, 1024, 1025] {
        assert_eq!(
            rsa::decrypt(&vec![0; length], &private_settings)
                .unwrap_err()
                .code,
            if length > 1024 {
                ErrorCode::ResourceLimit
            } else {
                ErrorCode::InvalidInput
            }
        );
    }
    let mut p = v["public_parameters"].clone();
    p["label_hex"] = json!("42".repeat(4096));
    let public_settings = rsa::EncryptionSettings::parse(&p.to_string(), &public).unwrap();
    let encrypted = rsa::encrypt(b"label bound", &public_settings).unwrap();
    p = v["private_parameters"].clone();
    p["label_hex"] = json!("42".repeat(4096));
    let s = rsa::DecryptionSettings::parse(&p.to_string(), &private).unwrap();
    assert_eq!(rsa::decrypt(&encrypted, &s).unwrap(), b"label bound");
    p["label_hex"] = json!("42".repeat(4097));
    assert_eq!(
        rsa::DecryptionSettings::parse(&p.to_string(), &private)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let error = rsa::DecryptionSettings::parse(&v["private_parameters"].to_string(), b"SECRET")
        .unwrap_err();
    assert!(!format!("{error:?}").contains("SECRET"));
    p = v["private_parameters"].clone();
    p["extra"] = json!(true);
    assert!(rsa::DecryptionSettings::parse(&p.to_string(), &private).is_err());
    let mut doc = v["private_parameters"].to_string();
    doc.push_str(&" ".repeat(rsa::MAX_PARAMETERS - doc.len()));
    rsa::DecryptionSettings::parse(&doc, &private).unwrap();
    doc.push(' ');
    assert_eq!(
        rsa::DecryptionSettings::parse(&doc, &private)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
