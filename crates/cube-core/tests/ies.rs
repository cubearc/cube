use cube_core::asymmetric::ies;
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/asymmetric/ies/interoperability.json")).unwrap()
}
#[test]
fn bouncy_castle_complete_suites_match_empty_nonempty_and_context_vectors() {
    for v in vectors() {
        let doc = v["parameters"].to_string();
        let private = ies::DecryptionSettings::parse(
            &doc,
            &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        let public = ies::EncryptionSettings::parse(
            &doc,
            &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        assert_eq!(ies::decrypt(&input, &private).unwrap(), plain);
        let fresh = ies::encrypt(&plain, &public).unwrap();
        assert_ne!(fresh, ies::encrypt(&plain, &public).unwrap());
        assert_eq!(ies::decrypt(&fresh, &private).unwrap(), plain);
        for pos in [0, input.len() / 2, input.len() - 1] {
            let mut bad = input.clone();
            bad[pos] ^= 1;
            assert_eq!(
                ies::decrypt(&bad, &private).unwrap_err().message,
                "IES operation failed"
            );
        }
        for field in ["derivation_hex", "encoding_hex"] {
            let mut wrong = v["parameters"].clone();
            wrong[field] = json!(format!("{}00", v["parameters"][field].as_str().unwrap()));
            let s = ies::DecryptionSettings::parse(
                &wrong.to_string(),
                &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
            )
            .unwrap();
            assert!(ies::decrypt(&input, &s).is_err());
        }
        assert!(!format!("{private:?}").contains(v["private_key_hex"].as_str().unwrap()));
    }
}
#[test]
fn invalid_profiles_keys_and_boundaries_refuse_without_output() {
    for v in vectors().into_iter().step_by(6) {
        let doc = v["parameters"].to_string();
        let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
        let private = ies::DecryptionSettings::parse(&doc, &key).unwrap();
        for input in [
            vec![],
            vec![0; 32],
            vec![0; 288],
            vec![0; ies::MAX_CIPHERTEXT + 1],
        ] {
            assert!(ies::decrypt(&input, &private).is_err());
        }
        for key in [vec![], vec![0; 32], vec![1; 31], vec![1; 33], vec![1; 257]] {
            assert!(ies::DecryptionSettings::parse(&doc, &key).is_err());
        }
        for key in [vec![], vec![0; 65], vec![0; 256], vec![1; 257]] {
            assert!(ies::EncryptionSettings::parse(&doc, &key).is_err());
        }
        for p in [
            json!({"profile":"auto"}),
            json!({"profile":v["parameters"]["profile"],"extra":true}),
            json!({"profile":v["parameters"]["profile"],"derivation_hex":"aa".repeat(257)}),
            json!({"profile":v["parameters"]["profile"],"encoding_hex":"0"}),
        ] {
            assert!(ies::DecryptionSettings::parse(&p.to_string(), &key).is_err());
        }
        assert!(
            ies::DecryptionSettings::parse(&format!("{doc}{}", " ".repeat(2049)), &key).is_err()
        );
    }
}
