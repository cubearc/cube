use cube_core::asymmetric::historical_public_key::{
    benaloh, goldwasser_micali, merkle_hellman, rabin,
};
use serde_json::Value;
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../src/asymmetric/historical_public_key/vectors.json"
    ))
    .unwrap()
}
#[test]
fn independent_historical_vectors_execute_exact_full_results_and_round_trips() {
    for v in vectors() {
        let doc = v["parameters"].to_string();
        let public = serde_json::to_vec(&v["public_key"]).unwrap();
        let private = serde_json::to_vec(&v["private_key"]).unwrap();
        let input: Vec<String> = serde_json::from_value(v["tokens"].clone()).unwrap();
        let cipher: Vec<String> = serde_json::from_value(v["ciphertext_tokens"].clone()).unwrap();
        match v["scheme"].as_str().unwrap() {
            "rabin" => {
                let p = rabin::EncryptionSettings::parse(&doc, &public).unwrap();
                let s = rabin::DecryptionSettings::parse(&doc, &private).unwrap();
                assert_eq!(rabin::encrypt(&input, &p).unwrap(), cipher);
                assert_eq!(
                    serde_json::to_value(rabin::decrypt(&cipher, &s).unwrap()).unwrap(),
                    v["candidate_tokens"]
                );
                assert!(rabin::encrypt(&[], &p).unwrap().is_empty());
            }
            "goldwasser_micali" => {
                let p = goldwasser_micali::EncryptionSettings::parse(&doc, &public).unwrap();
                let s = goldwasser_micali::DecryptionSettings::parse(&doc, &private).unwrap();
                assert_eq!(goldwasser_micali::decrypt(&cipher, &s).unwrap(), input);
                let fresh = goldwasser_micali::encrypt(&input, &p).unwrap();
                assert_eq!(goldwasser_micali::decrypt(&fresh, &s).unwrap(), input);
                assert!(goldwasser_micali::encrypt(&[], &p).unwrap().is_empty());
            }
            "benaloh" => {
                let p = benaloh::EncryptionSettings::parse(&doc, &public).unwrap();
                let s = benaloh::DecryptionSettings::parse(&doc, &private).unwrap();
                assert_eq!(benaloh::decrypt(&cipher, &s).unwrap(), input);
                let fresh = benaloh::encrypt(&input, &p).unwrap();
                assert_eq!(benaloh::decrypt(&fresh, &s).unwrap(), input);
                assert!(benaloh::encrypt(&[], &p).unwrap().is_empty());
            }
            "merkle_hellman" => {
                let p = merkle_hellman::EncryptionSettings::parse(&doc, &public).unwrap();
                let s = merkle_hellman::DecryptionSettings::parse(&doc, &private).unwrap();
                assert_eq!(
                    s.public_weights(),
                    serde_json::from_value::<Vec<String>>(v["public_key"]["weights"].clone())
                        .unwrap()
                );
                assert_eq!(merkle_hellman::encrypt(&input, &p).unwrap(), cipher);
                assert_eq!(merkle_hellman::decrypt(&cipher, &s).unwrap(), input);
                assert!(merkle_hellman::encrypt(&[], &p).unwrap().is_empty());
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn malformed_ciphertexts_bits_root_ambiguity_and_knapsack_congruences_fail_wholly() {
    let v = &vectors()[0];
    let doc = v["parameters"].to_string();
    let s = rabin::DecryptionSettings::parse(&doc, &serde_json::to_vec(&v["private_key"]).unwrap())
        .unwrap();
    assert!(rabin::decrypt(&["15".into(), "2".into()], &s).is_err());
    let v = &vectors()[2];
    let doc = v["parameters"].to_string();
    let p = goldwasser_micali::EncryptionSettings::parse(
        &doc,
        &serde_json::to_vec(&v["public_key"]).unwrap(),
    )
    .unwrap();
    let s = goldwasser_micali::DecryptionSettings::parse(
        &doc,
        &serde_json::to_vec(&v["private_key"]).unwrap(),
    )
    .unwrap();
    for bit in ["2", "01", "١"] {
        assert!(goldwasser_micali::encrypt(&[bit.into()], &p).is_err());
    }
    assert!(goldwasser_micali::decrypt(&["1".into(), "499".into()], &s).is_err());
    let mut bad = v["private_key"].clone();
    bad["y"] = serde_json::json!("4");
    assert!(
        goldwasser_micali::DecryptionSettings::parse(&doc, &serde_json::to_vec(&bad).unwrap())
            .is_err()
    );
    let v = &vectors()[3];
    let doc = v["parameters"].to_string();
    for r in [2, 15, 4095] {
        let mut key = v["public_key"].clone();
        key["r"] = serde_json::json!(r);
        assert!(
            benaloh::EncryptionSettings::parse(&doc, &serde_json::to_vec(&key).unwrap()).is_err()
        );
    }
    let mut bad = v["private_key"].clone();
    bad["y"] = serde_json::json!("8");
    assert!(benaloh::DecryptionSettings::parse(&doc, &serde_json::to_vec(&bad).unwrap()).is_err());
    let p =
        benaloh::EncryptionSettings::parse(&doc, &serde_json::to_vec(&v["public_key"]).unwrap())
            .unwrap();
    assert!(benaloh::encrypt(&["3".into()], &p).is_err());
    let v = &vectors()[7];
    let doc = v["parameters"].to_string();
    let s = merkle_hellman::DecryptionSettings::parse(
        &doc,
        &serde_json::to_vec(&v["private_key"]).unwrap(),
    )
    .unwrap();
    assert!(merkle_hellman::decrypt(&["0".into(), "491".into()], &s).is_err());
    for (field, value) in [
        ("permutation", serde_json::json!([0, 0, 2, 3, 4, 5, 6, 7])),
        ("modulus", serde_json::json!("484")),
        ("multiplier", serde_json::json!("0")),
        ("weights", serde_json::json!(["2", "3", "4"])),
    ] {
        let mut key = v["private_key"].clone();
        key[field] = value;
        assert!(
            merkle_hellman::DecryptionSettings::parse(&doc, &serde_json::to_vec(&key).unwrap())
                .is_err()
        );
    }
}
