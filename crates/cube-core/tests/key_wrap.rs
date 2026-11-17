use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn rfc3394_known_wrapping_and_integrity_checked_unwrapping_execute() {
    let plain = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    let wrapped = hex::decode("1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5").unwrap();
    let parameters =
        json!({"profile":"aes_kw_rfc3394","key_hex":"000102030405060708090a0b0c0d0e0f"});
    for (op, input, expected) in [
        ("cipher.aes.key_wrap.wrap", &plain, &wrapped),
        ("cipher.aes.key_wrap.unwrap", &wrapped, &plain),
    ] {
        let result = execution::execute(
            op,
            1,
            &json!({"input":{"base64":cube_core::encoding::base64_encode(input)}}),
            &parameters,
            Budget::default(),
        )
        .unwrap();
        assert_eq!(
            result,
            json!({"output":{"base64":cube_core::encoding::base64_encode(expected)}})
        );
    }
}

fn cases(invalid: bool) -> Vec<serde_json::Value> {
    serde_json::from_str(if invalid {
        include_str!("../src/ciphers/key_wrap/invalid_vectors.json")
    } else {
        include_str!("../src/ciphers/key_wrap/vectors.json")
    })
    .unwrap()
}
fn bytes(v: &serde_json::Value, name: &str) -> Vec<u8> {
    hex::decode(v[name].as_str().unwrap()).unwrap()
}
#[test]
fn nist_all_kek_sizes_short_partial_and_aligned_kwp_integrity_cases() {
    use cube_core::ciphers::key_wrap as kw;
    for v in cases(false) {
        let s = kw::Settings::parse(&v["parameters"].to_string()).unwrap();
        let plain = bytes(&v, "plaintext_hex");
        let wrapped = bytes(&v, "wrapped_hex");
        assert_eq!(kw::wrap(&plain, &s).unwrap(), wrapped);
        assert_eq!(kw::unwrap(&wrapped, &s).unwrap(), plain);
    }
    for v in cases(true) {
        assert_eq!(v["valid"], false);
        let s = kw::Settings::parse(&v["parameters"].to_string()).unwrap();
        let wrapped = bytes(&v, "wrapped_hex");
        assert_eq!(
            kw::unwrap(&wrapped, &s).unwrap_err().message,
            "key unwrap integrity check failed"
        );
    }
}
#[test]
fn bounds_wrong_keys_lengths_and_secret_fields_are_exact() {
    use cube_core::{ciphers::key_wrap as kw, errors::ErrorCode};
    for profile in kw::Profile::NAMES {
        let p = json!({"profile":profile,"key_hex":"42".repeat(16)});
        let s = kw::Settings::parse(&p.to_string()).unwrap();
        assert!(!format!("{s:?}").contains("424242"));
        for n in [1, 7, 8, 9, 15, 16, 17, 24, 32, 65535, 65536] {
            let data = vec![0x24; n];
            if *profile == "aes_kw_rfc3394" && (n < 16 || !n.is_multiple_of(8)) {
                assert!(kw::wrap(&data, &s).is_err());
                continue;
            }
            let wrapped = kw::wrap(&data, &s).unwrap();
            assert_eq!(wrapped.len(), n.div_ceil(8) * 8 + 8);
            assert_eq!(kw::unwrap(&wrapped, &s).unwrap(), data);
            let mut bad = wrapped;
            bad[0] ^= 1;
            assert!(kw::unwrap(&bad, &s).is_err());
        }
        assert!(kw::wrap(&[], &s).is_err());
        assert_eq!(
            kw::wrap(&vec![0; 65537], &s).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        for n in [0, 1, 7, 8, 15, 17, 23] {
            assert_eq!(
                kw::unwrap(&vec![0; n], &s).unwrap_err().message,
                "key unwrap integrity check failed"
            );
        }
        let mut p = p.clone();
        p["extra"] = json!("SECRET");
        let error = kw::Settings::parse(&p.to_string()).unwrap_err();
        assert!(!format!("{error:?}").contains("SECRET"));
    }
    let v = &cases(false)[0];
    let wrapped = bytes(v, "wrapped_hex");
    let mut p = v["parameters"].clone();
    p["key_hex"] = json!("00".repeat(16));
    let s = kw::Settings::parse(&p.to_string()).unwrap();
    assert!(kw::unwrap(&wrapped, &s).is_err());
    let p = json!({"profile":"aes_kwp_rfc5649","key_hex":"42".repeat(16)});
    let mut doc = p.to_string();
    doc.push_str(&" ".repeat(kw::MAX_PARAMETERS - doc.len()));
    kw::Settings::parse(&doc).unwrap();
    doc.push(' ');
    assert!(kw::Settings::parse(&doc).is_err());
}
