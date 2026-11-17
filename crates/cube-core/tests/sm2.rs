use cube_core::asymmetric::sm2;
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/asymmetric/sm2/interoperability.json")).unwrap()
}
#[test]
fn openssl_sm2_vectors_and_fresh_encryptions_interoperate_in_all_three_encodings() {
    for v in vectors() {
        for (profile, field) in [
            ("sm2_sm3_c1c3c2", "c1c3c2_hex"),
            ("sm2_sm3_c1c2c3", "c1c2c3_hex"),
            ("sm2_sm3_der", "der_hex"),
        ] {
            let doc = json!({"profile":profile}).to_string();
            let private = sm2::DecryptionSettings::parse(
                &doc,
                &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
            let public = sm2::EncryptionSettings::parse(
                &doc,
                &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
            let expected = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
            let ciphertext = hex::decode(v[field].as_str().unwrap()).unwrap();
            assert_eq!(sm2::decrypt(&ciphertext, &private).unwrap(), expected);
            let first = sm2::encrypt(&expected, &public).unwrap();
            let second = sm2::encrypt(&expected, &public).unwrap();
            assert_ne!(first, second);
            assert_eq!(sm2::decrypt(&first, &private).unwrap(), expected);
            for pos in [0, 1, ciphertext.len() / 2, ciphertext.len() - 1] {
                let mut bad = ciphertext.clone();
                bad[pos] ^= 1;
                assert_eq!(
                    sm2::decrypt(&bad, &private).unwrap_err().message,
                    "SM2 operation failed"
                );
            }
            let mut trailing = ciphertext.clone();
            trailing.push(0);
            assert!(sm2::decrypt(&trailing, &private).is_err());
            for input in [&[][..], &[0][..], &ciphertext[..32]] {
                assert!(sm2::decrypt(input, &private).is_err());
            }
            assert!(sm2::encrypt(&[], &public).is_err());
            assert!(!format!("{private:?}").contains(v["private_key_hex"].as_str().unwrap()));
        }
    }
}
#[test]
fn wrong_curves_keys_profiles_orders_and_limits_fail() {
    let v = &vectors()[0];
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let doc = r#"{"profile":"sm2_sm3_c1c3c2"}"#;
    let private = sm2::DecryptionSettings::parse(doc, &key).unwrap();
    for key in [
        vec![],
        vec![0; 32],
        vec![255; 32],
        vec![1; 31],
        vec![1; 33],
        vec![1; 66],
    ] {
        assert!(sm2::DecryptionSettings::parse(doc, &key).is_err());
    }
    for key in [vec![],vec![0;65],vec![4;65],vec![4;66],hex::decode("046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5").unwrap()]{assert!(sm2::EncryptionSettings::parse(doc,&key).is_err());}
    for doc in [
        "{}".to_string(),
        json!({"profile":"p256"}).to_string(),
        json!({"profile":"sm2_sm3_c1c3c2","extra":true}).to_string(),
        format!("{doc}{}", " ".repeat(257)),
    ] {
        assert!(sm2::DecryptionSettings::parse(&doc, &key).is_err());
    }
    let wrong = sm2::DecryptionSettings::parse(r#"{"profile":"sm2_sm3_c1c2c3"}"#, &key).unwrap();
    assert!(
        sm2::decrypt(
            &hex::decode(v["c1c3c2_hex"].as_str().unwrap()).unwrap(),
            &wrong
        )
        .is_err()
    );
    assert!(sm2::decrypt(&vec![0; sm2::MAX_CIPHERTEXT + 1], &private).is_err());
}
