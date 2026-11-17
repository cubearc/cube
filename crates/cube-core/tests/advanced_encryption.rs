use cube_core::{
    asymmetric::{additive, lattice_reference as lattice},
    errors::ErrorCode,
};
use serde::Deserialize;

#[test]
fn public_ciphertext_sum_matches_independent_wrap_vectors() {
    let vectors = |family, role| {
        std::fs::read(format!(
            "{}/src/asymmetric/additive/vectors/{family}-{role}.bin",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    };
    for (name, profile) in [
        ("paillier", additive::Profile::Paillier),
        ("damgard-jurik", additive::Profile::DamgardJurik),
    ] {
        let modulus = vectors(name, "modulus");
        let left = vectors(name, "lhs");
        let right = vectors(name, "rhs");
        let expected = vectors(name, "sum");
        assert_eq!(
            additive::add(&left, &right, &modulus, profile).unwrap(),
            expected
        );
        let zero = vec![0; left.len()];
        assert_eq!(
            additive::add(&zero, &right, &modulus, profile)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            additive::add(&left[..left.len() - 1], &right, &modulus, profile)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
        let mut same_size_even = modulus.clone();
        *same_size_even.last_mut().unwrap() &= !1;
        assert!(additive::add(&left, &right, &same_size_even, profile).is_err());
        for bytes in [384, 512] {
            let mut full_width_odd = vec![0; bytes];
            full_width_odd[0] = 0x80;
            full_width_odd[bytes - 1] = 1;
            let width = profile.ciphertext_bytes(bytes);
            let mut identity = vec![0; width];
            identity[width - 1] = 1;
            assert_eq!(
                additive::add(&identity, &identity, &full_width_odd, profile).unwrap(),
                identity
            );
            identity.push(0);
            let error = additive::add(&identity, &identity, &full_width_odd, profile).unwrap_err();
            assert_eq!(
                error.code,
                if bytes == 512 {
                    ErrorCode::ResourceLimit
                } else {
                    ErrorCode::InvalidInput
                }
            );
        }
    }
}

#[derive(Deserialize)]
struct LatticeVector {
    plaintext: Vec<u8>,
    ciphertext: Vec<u8>,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

#[test]
fn lattice_teaching_profiles_match_independent_cases_and_fresh_round_trips() {
    for (profile, file) in [
        (
            lattice::Profile::NtruEncrypt,
            include_str!("../src/asymmetric/lattice_reference/ntru-encrypt.json"),
        ),
        (
            lattice::Profile::Regev,
            include_str!("../src/asymmetric/lattice_reference/regev.json"),
        ),
    ] {
        let vector: LatticeVector = serde_json::from_str(file).unwrap();
        assert_eq!(
            lattice::decrypt(&vector.ciphertext, &vector.private_key, profile).unwrap(),
            vector.plaintext
        );
        for _ in 0..16 {
            let ciphertext =
                lattice::encrypt(&vector.plaintext, &vector.public_key, profile).unwrap();
            assert_eq!(
                lattice::decrypt(&ciphertext, &vector.private_key, profile).unwrap(),
                vector.plaintext
            );
        }
        let mut bad_input = vector.plaintext.clone();
        bad_input[0] = 255;
        assert_eq!(
            lattice::encrypt(&bad_input, &vector.public_key, profile)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
        bad_input.push(0);
        assert_eq!(
            lattice::encrypt(&bad_input, &vector.public_key, profile)
                .unwrap_err()
                .code,
            ErrorCode::ResourceLimit
        );
        assert!(lattice::decrypt(&vector.ciphertext, &vector.private_key[1..], profile).is_err());
        assert!(
            profile
                .validate_parameters(r#"{"profile":"ml_kem_512_fips203"}"#)
                .is_err()
        );
    }
    assert!(lattice::decrypt(&[0, 0, 0, 0, 24], &[0; 4], lattice::Profile::Regev).is_err());
}
