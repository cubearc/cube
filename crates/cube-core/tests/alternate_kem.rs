use cube_core::{
    asymmetric::alternate_kem::{self, Profile},
    errors::ErrorCode,
};
use serde_json::Value;

#[test]
fn independent_portable_kem_answers_and_exact_byte_boundaries() {
    for (profile, document) in [
        (
            Profile::Frodo640Shake,
            include_str!("../src/asymmetric/alternate_kem/frodo640.json"),
        ),
        (
            Profile::Hqc128February2026,
            include_str!("../src/asymmetric/alternate_kem/hqc128.json"),
        ),
        (
            Profile::McEliece460896f,
            include_str!("../src/asymmetric/alternate_kem/mceliece460896f.json"),
        ),
        (
            Profile::BikeL1,
            include_str!("../src/asymmetric/alternate_kem/bike-l1.json"),
        ),
        (
            Profile::NtruHps2048509,
            include_str!("../src/asymmetric/alternate_kem/ntruhps2048509.json"),
        ),
    ] {
        let case: Value = serde_json::from_str(document).unwrap();
        let raw = |field: &str| hex::decode(case[field].as_str().unwrap()).unwrap();
        let mut ciphertext = raw("ct");
        let key = raw("sk");
        let expected = raw("ss");
        let actual = alternate_kem::decapsulate(&ciphertext, &key, profile).unwrap();
        assert!(*actual == expected, "{} fixed reference", profile.name());
        assert_eq!(
            alternate_kem::decapsulate(&ciphertext, &key[1..], profile)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
        ciphertext[0] ^= 1;
        let rejected = alternate_kem::decapsulate(&ciphertext, &key, profile).unwrap();
        assert!(rejected.len() == expected.len() && *rejected != expected);
        if case["rejection_ss"].is_string() {
            assert!(*rejected == raw("rejection_ss"));
        }
        ciphertext.push(0);
        assert_eq!(
            alternate_kem::decapsulate(&ciphertext, &key, profile)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
    }
}

#[test]
fn bike_rejects_out_of_domain_key_rotations() {
    let (key_bytes, ciphertext_bytes, _) = Profile::BikeL1.widths();
    let mut key = vec![0; key_bytes];
    key[..4].copy_from_slice(&12_323_u32.to_le_bytes());
    let ciphertext = vec![0; ciphertext_bytes];
    assert_eq!(
        alternate_kem::decapsulate(&ciphertext, &key, Profile::BikeL1)
            .unwrap_err()
            .code,
        ErrorCode::InvalidInput
    );
}
