use cube_core::asymmetric::group_encryption as g;
use serde_json::Value;
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../src/asymmetric/group_encryption/vectors.json"
    ))
    .unwrap()
}
#[test]
fn independent_group_vectors_match_private_public_derivation_and_fresh_ciphertexts() {
    for v in vectors() {
        let doc = v["parameters"].to_string();
        let private = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
        let public = hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap();
        let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
        let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
        if v["scheme"] == "elgamal" {
            let s = g::ElGamalDecryptionSettings::parse(&doc, &private).unwrap();
            assert_eq!(s.public_key(), public);
            assert_eq!(g::elgamal_decrypt(&input, &s).unwrap(), plain);
            let p = g::ElGamalEncryptionSettings::parse(&doc, &public).unwrap();
            let first = g::elgamal_encrypt(&plain, &p).unwrap();
            assert_ne!(first, g::elgamal_encrypt(&plain, &p).unwrap());
            assert_eq!(g::elgamal_decrypt(&first, &s).unwrap(), plain);
        } else {
            let s = g::CramerShoupDecryptionSettings::parse(&doc, &private).unwrap();
            assert_eq!(s.public_key(), public);
            assert_eq!(g::cramer_shoup_decrypt(&input, &s).unwrap(), plain);
            let p = g::CramerShoupEncryptionSettings::parse(&doc, &public).unwrap();
            let first = g::cramer_shoup_encrypt(&plain, &p).unwrap();
            assert_ne!(first, g::cramer_shoup_encrypt(&plain, &p).unwrap());
            assert_eq!(g::cramer_shoup_decrypt(&first, &s).unwrap(), plain);
            let mut bad = input.clone();
            bad[1023] ^= 1;
            assert!(g::cramer_shoup_decrypt(&bad, &s).is_err());
        }
    }
}
#[test]
fn zero_wrong_subgroup_unknown_profile_and_encoding_bounds_fail_without_output() {
    let v = &vectors()[1];
    let doc = v["parameters"].to_string();
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let s = g::ElGamalDecryptionSettings::parse(&doc, &key).unwrap();
    for input in [vec![], vec![0; 256], vec![0; 512], vec![0; 1025]] {
        assert!(g::elgamal_decrypt(&input, &s).is_err());
    }
    for key in [
        vec![],
        vec![0; 256],
        vec![255; 256],
        vec![1; 255],
        vec![1; 257],
        vec![1; 1281],
    ] {
        assert!(g::ElGamalDecryptionSettings::parse(&doc, &key).is_err());
    }
    assert!(g::ElGamalEncryptionSettings::parse(&doc, &vec![0; 256]).is_err());
    assert!(g::ElGamalDecryptionSettings::parse("{}", &key).is_err());
    assert!(
        g::ElGamalDecryptionSettings::parse(&format!("{doc}{}", " ".repeat(1025)), &key).is_err()
    );
    let v = &vectors()[5];
    let doc = v["parameters"].to_string();
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let s = g::CramerShoupDecryptionSettings::parse(&doc, &key).unwrap();
    assert!(!format!("{s:?}").contains(v["private_key_hex"].as_str().unwrap()));
    for input in [vec![], vec![0; 1024], vec![0; 1025]] {
        assert!(g::cramer_shoup_decrypt(&input, &s).is_err());
    }
    let mut wrong = v["parameters"].clone();
    wrong["label_hex"] = serde_json::json!("00");
    let wrong = g::CramerShoupDecryptionSettings::parse(&wrong.to_string(), &key).unwrap();
    assert!(
        g::cramer_shoup_decrypt(
            &hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap(),
            &wrong
        )
        .is_err()
    );
}
