use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn nist_decimal_ff1_executes_with_exact_numeric_transport() {
    let p = json!({"profile":"ff1_sp800_38g_integer","radix":10,"key_hex":"2b7e151628aed2a6abf7158809cf4f3c","tweak_hex":""});
    let result = execution::execute(
        "cipher.fpe.encrypt",
        1,
        &json!({"input":[0,1,2,3,4,5,6,7,8,9]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result, json!({"output":[2,4,3,3,4,7,7,4,8,4]}));
}

#[test]
fn independent_nist_and_ff3_1_profiles_preserve_exact_digits() {
    use cube_core::ciphers::fpe;
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/fpe/vectors.json")).unwrap();
    for v in cases {
        let s = fpe::Settings::parse(&v["parameters"].to_string()).unwrap();
        let plain: Vec<u16> = serde_json::from_value(v["plaintext"].clone()).unwrap();
        let encrypted: Vec<u16> = serde_json::from_value(v["ciphertext"].clone()).unwrap();
        assert_eq!(
            fpe::encrypt(&plain, &s).unwrap(),
            encrypted,
            "{}",
            v["source"]
        );
        assert_eq!(
            fpe::decrypt(&encrypted, &s).unwrap(),
            plain,
            "{}",
            v["source"]
        );
    }
}
#[test]
fn domain_radix_tweak_limits_and_secret_errors_are_checked() {
    use cube_core::{ciphers::fpe, errors::ErrorCode};
    for profile in fpe::Profile::NAMES {
        let tweak = if profile.starts_with("ff1_") {
            "".into()
        } else if profile.starts_with("ff3_1") {
            "00".repeat(7)
        } else {
            "00".repeat(8)
        };
        let mut p =
            json!({"profile":profile,"radix":10,"key_hex":"42".repeat(16),"tweak_hex":tweak});
        let s = fpe::Settings::parse(&p.to_string()).unwrap();
        assert!(!format!("{s:?}").contains("424242"));
        for input in [
            &[][..],
            &[0, 0][..],
            &[0, 0, 0, 0, 0][..],
            &[0, 0, 0, 0, 0, 10][..],
        ] {
            assert!(fpe::encrypt(input, &s).is_err());
        }
        let plain = vec![0; 6];
        assert_eq!(
            fpe::decrypt(&fpe::encrypt(&plain, &s).unwrap(), &s).unwrap(),
            plain
        );
        assert_eq!(
            fpe::encrypt(&vec![0; 4097], &s).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        if !profile.starts_with("ff1_") {
            assert!(fpe::encrypt(&[0; 57], &s).is_err());
            p["tweak_hex"] = json!("");
            assert!(fpe::Settings::parse(&p.to_string()).is_err());
        }
        p["key_hex"] = json!("SECRET");
        assert!(
            !format!("{:?}", fpe::Settings::parse(&p.to_string()).unwrap_err()).contains("SECRET")
        );
    }
    let p = json!({"profile":"ff1_sp800_38g_integer","radix":65536,"key_hex":"00".repeat(32),"tweak_hex":"00".repeat(fpe::MAX_TWEAK)});
    let s = fpe::Settings::parse(&p.to_string()).unwrap();
    let input = vec![65535; 4096];
    assert_eq!(
        fpe::decrypt(&fpe::encrypt(&input, &s).unwrap(), &s).unwrap(),
        input
    );
    for radix in [2, 3, 10, 36, 65535, 65536] {
        let p = json!({"profile":"ff1_sp800_38g_integer","radix":radix,"key_hex":"00".repeat(24),"tweak_hex":"010203"});
        let s = fpe::Settings::parse(&p.to_string()).unwrap();
        let input = vec![0; 20];
        assert_eq!(
            fpe::decrypt(&fpe::encrypt(&input, &s).unwrap(), &s).unwrap(),
            input
        );
    }
}
