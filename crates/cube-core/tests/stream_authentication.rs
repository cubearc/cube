//! Exact original Grain128a authentication through public Core and executor.
use cube_core::{
    ciphers::stream::reference::{self, authentication as auth},
    errors::ErrorCode,
    operations::execution::{self, Budget},
};
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../src/ciphers/stream/reference/authentication/vectors.json"
    ))
    .unwrap()
}
fn bytes(hex: &Value) -> Vec<u8> {
    cube_core::encoding::hex_decode(hex.as_str().unwrap()).unwrap()
}
fn transport(data: &[u8]) -> Value {
    json!({"base64":cube_core::encoding::base64_encode(data)})
}
#[test]
fn independent_ciphertext_and_tag_answers_execute_all_named_ports() {
    for v in vectors() {
        let p = &v["parameters"];
        let plain = bytes(&v["plaintext_hex"]);
        let ciphertext = bytes(&v["ciphertext_hex"]);
        let tag = bytes(&v["tag_hex"]);
        let settings = auth::Settings::parse(&p.to_string()).unwrap();
        let encrypted = auth::encrypt(&plain, &settings).unwrap();
        assert_eq!(encrypted.ciphertext, ciphertext);
        assert_eq!(encrypted.tag, tag);
        assert_eq!(auth::decrypt(&ciphertext, &tag, &settings).unwrap(), plain);
        assert_eq!(
            execution::execute(
                auth::ENCRYPT,
                1,
                &json!({"input":transport(&plain)}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":transport(&ciphertext),"tag":transport(&tag)})
        );
        assert_eq!(
            execution::execute(
                auth::DECRYPT,
                1,
                &json!({"input":transport(&ciphertext),"tag":transport(&tag)}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":transport(&plain)})
        );
        let mut forged = tag.clone();
        forged[0] ^= 0x80;
        let error = auth::decrypt(&ciphertext, &forged, &settings).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.message, "authentication failed");
        assert!(
            execution::execute(
                auth::DECRYPT,
                1,
                &json!({"input":transport(&ciphertext),"tag":transport(&forged)}),
                p,
                Budget::default()
            )
            .is_err()
        );
    }
}
#[test]
fn modified_messages_contexts_and_tag_padding_release_no_plaintext() {
    let v = &vectors()[4];
    let p = &v["parameters"];
    let ciphertext = bytes(&v["ciphertext_hex"]);
    let tag = bytes(&v["tag_hex"]);
    let settings = auth::Settings::parse(&p.to_string()).unwrap();
    let mut corrupt = ciphertext.clone();
    corrupt[0] ^= 0x80;
    assert!(auth::decrypt(&corrupt, &tag, &settings).is_err());
    for (name, value) in [
        ("key_hex", json!("01".repeat(16))),
        ("nonce_hex", json!("01".repeat(12))),
    ] {
        let mut changed = p.clone();
        changed[name] = value;
        let settings = auth::Settings::parse(&changed.to_string()).unwrap();
        assert!(auth::decrypt(&ciphertext, &tag, &settings).is_err());
    }
    let mut bad_padding = ciphertext.clone();
    *bad_padding.last_mut().unwrap() |= 1;
    assert!(auth::decrypt(&bad_padding, &tag, &settings).is_err());
    for invalid in [vec![], vec![0; 5]] {
        assert!(auth::decrypt(&ciphertext, &invalid, &settings).is_err());
    }
    let mut p = p.clone();
    p["data_bits"] = json!(0);
    p["tag_bits"] = json!(1);
    let settings = auth::Settings::parse(&p.to_string()).unwrap();
    let result = auth::encrypt(&[], &settings).unwrap();
    let mut bad = result.tag.clone();
    bad[0] |= 1;
    assert!(auth::decrypt(&[], &bad, &settings).is_err());
}
#[test]
fn settings_are_required_private_and_match_contract_constraints() {
    let good = vectors()[4]["parameters"].clone();
    let contract = cube_core::operations::require(auth::ENCRYPT)
        .unwrap()
        .contract()
        .unwrap();
    for name in ["profile", "key_hex", "nonce_hex", "data_bits", "tag_bits"] {
        for value in [None, Some(json!(null))] {
            let mut p = good.clone();
            if let Some(value) = value {
                p[name] = value;
            } else {
                p.as_object_mut().unwrap().remove(name);
            }
            assert!(contract.validate_parameters(&p).is_err());
            assert!(auth::Settings::parse(&p.to_string()).is_err());
        }
    }
    for (name, value) in [
        ("key_hex", json!("SECRET")),
        ("key_hex", json!("00".repeat(17))),
        ("nonce_hex", json!("00".repeat(11))),
        ("data_bits", json!(524289)),
        ("tag_bits", json!(0)),
        ("tag_bits", json!(33)),
        ("tag_bits", json!(1.5)),
        ("associated_data", json!("SECRET")),
    ] {
        let mut p = good.clone();
        p[name] = value;
        assert!(contract.validate_parameters(&p).is_err());
        let error = auth::Settings::parse(&p.to_string()).unwrap_err();
        assert!(!format!("{error:?}").contains("SECRET"));
    }
    let duplicate = format!(
        "{},\"tag_bits\":32}}",
        good.to_string().trim_end_matches('}')
    );
    assert!(auth::Settings::parse(&duplicate).is_err());
    let settings = auth::Settings::parse(&good.to_string()).unwrap();
    assert!(!format!("{settings:?}").contains(good["key_hex"].as_str().unwrap()));
    let mut padded = good.to_string();
    padded.push_str(&" ".repeat(reference::MAX_PARAMETERS - padded.len()));
    assert!(auth::Settings::parse(&padded).is_ok());
    padded.push(' ');
    assert_eq!(
        auth::Settings::parse(&padded).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn full_packed_limit_and_shortened_tag_order_are_preserved() {
    let mut p = vectors()[4]["parameters"].clone();
    p["tag_bits"] = json!(16);
    let settings = auth::Settings::parse(&p.to_string()).unwrap();
    let result = auth::encrypt(&bytes(&vectors()[4]["plaintext_hex"]), &settings).unwrap();
    assert_eq!(cube_core::encoding::hex_encode(&result.tag), "cab1");
    p["data_bits"] = json!(reference::MAX_INPUT * 8);
    p["tag_bits"] = json!(32);
    assert_eq!(
        execution::execute(
            auth::ENCRYPT,
            1,
            &json!({"input":transport(&vec![0; reference::MAX_INPUT+1])}),
            &p,
            Budget::default()
        )
        .unwrap_err()
        .0,
        "resource_limit"
    );
    let settings = auth::Settings::parse(&p.to_string()).unwrap();
    let plain = vec![0xa5; reference::MAX_INPUT];
    let result = auth::encrypt(&plain, &settings).unwrap();
    assert_eq!(
        auth::decrypt(&result.ciphertext, &result.tag, &settings).unwrap(),
        plain
    );
    assert_eq!(
        auth::encrypt(&vec![0; reference::MAX_INPUT + 1], &settings)
            .err()
            .unwrap()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        auth::decrypt(&vec![0; reference::MAX_INPUT + 1], &[0; 4], &settings)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
