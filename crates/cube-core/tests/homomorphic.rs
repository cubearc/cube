use cube_core::{
    asymmetric::homomorphic::{self as h, Profile},
    errors::ErrorCode,
};

macro_rules! vector {
    ($variant:literal, $file:literal) => {
        include_bytes!(concat!(
            "../src/asymmetric/homomorphic/vectors/",
            $variant,
            "-",
            $file,
            ".bin"
        ))
        .as_slice()
    };
}

#[test]
fn seal_ciphertext_addition_matches_the_external_word_results() {
    for (profile, left, right, expected) in [
        (
            Profile::Bfv,
            vector!("bfv", "lhs"),
            vector!("bfv", "rhs"),
            vector!("bfv", "sum"),
        ),
        (
            Profile::Bgv,
            vector!("bgv", "lhs"),
            vector!("bgv", "rhs"),
            vector!("bgv", "sum"),
        ),
        (
            Profile::Ckks,
            vector!("ckks", "lhs"),
            vector!("ckks", "rhs"),
            vector!("ckks", "sum"),
        ),
    ] {
        assert_eq!(h::add(left, right, profile).unwrap(), expected);
        assert_eq!(
            h::add(&left[1..], right, profile).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        let mut noncanonical = left.to_vec();
        noncanonical[4..12].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            h::add(&noncanonical, right, profile).unwrap_err().code,
            ErrorCode::InvalidInput
        );
        noncanonical.push(0);
        assert_eq!(
            h::add(&noncanonical, right, profile).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert!(h::add(vector!("bfv", "lhs"), vector!("bgv", "rhs"), Profile::Bfv).is_err());
}

#[test]
fn tfhe_not_matches_encrypted_output_and_is_self_inverse() {
    let input = vector!("tfhe", "input");
    let negated = vector!("tfhe", "not");
    assert_eq!(h::tfhe_not(input).unwrap(), negated);
    assert_eq!(h::tfhe_not(negated).unwrap(), input);
    assert_eq!(
        h::tfhe_not(&input[1..]).unwrap_err().code,
        ErrorCode::InvalidInput
    );
}
