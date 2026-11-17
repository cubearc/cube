use cube_core::{
    asymmetric::{kyber, ml_kem},
    errors::ErrorCode,
};
use serde_json::{Value, json};

#[test]
fn original_round_three_valid_and_modified_ciphertexts_match() {
    let cases: Value =
        serde_json::from_str(include_str!("../src/asymmetric/kyber/vectors.json")).unwrap();
    let fips = [
        ml_kem::Profile::MlKem512,
        ml_kem::Profile::MlKem768,
        ml_kem::Profile::MlKem1024,
    ];
    for (case, fips_profile) in cases["profiles"].as_array().unwrap().iter().zip(fips) {
        let raw = |name: &str| hex::decode(case[name].as_str().unwrap()).unwrap();
        let profile =
            kyber::Profile::parse(&json!({"profile":case["profile"]}).to_string()).unwrap();
        let key = raw("dk");
        let ct = raw("ciphertext");
        assert!(kyber::decapsulate(&ct, &key, profile).unwrap()[..] == raw("shared_secret"));
        assert!(
            kyber::decapsulate(&raw("modified_ciphertext"), &key, profile).unwrap()[..]
                == raw("rejection_secret")
        );
        assert!(ml_kem::decapsulate(&ct, &key, fips_profile).unwrap()[..] != raw("shared_secret"));
        assert_eq!(
            kyber::decapsulate(&ct[1..], &key, profile)
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidInput
        );
        let mut malformed = key;
        let hash = malformed.len() - 64;
        malformed[hash] ^= 1;
        assert_eq!(
            kyber::decapsulate(&ct, &malformed, profile)
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidInput
        );
    }
}
