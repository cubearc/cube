use cube_core::{errors::ErrorCode, numbers::modular::*};
use num_bigint::BigInt;
#[test]
fn exact_euclidean_results_and_independent_small_divisor_oracle() {
    for a in 0..80u64 {
        for b in 0..80u64 {
            let expected = (1..=a.max(b))
                .rev()
                .find(|d| a % d == 0 && b % d == 0)
                .unwrap_or(0);
            assert_eq!(gcd(a, b).unwrap(), expected);
            let r = extended_gcd(a, b).unwrap();
            assert_eq!(r.gcd, expected);
            assert_eq!(
                i128::from(a) * r.x + i128::from(b) * r.y,
                i128::from(expected)
            );
        }
    }
    assert_eq!(
        extended_gcd(240, 46).unwrap(),
        ExtendedGcd {
            gcd: 2,
            x: -9,
            y: 47
        }
    );
    for (a, b, x, y) in [(0, 0, 0, 0), (10, 0, 1, 0), (0, 10, 0, 1), (10, 10, 0, 1)] {
        let r = extended_gcd(a, b).unwrap();
        assert_eq!((r.x, r.y), (x, y));
    }
}
#[test]
fn full_width_bezout_identity_uses_an_independent_big_integer_oracle() {
    let m = u64::MAX;
    let values = [
        0,
        1,
        2,
        3,
        m,
        m - 1,
        m - 2,
        m / 2,
        12_200_160_415_121_876_738,
        7_540_113_804_746_346_429,
    ];
    for a in values {
        for b in values {
            let r = extended_gcd(a, b).unwrap();
            assert_eq!(r.gcd, gcd(a, b).unwrap());
            assert_eq!(
                BigInt::from(a) * BigInt::from(r.x) + BigInt::from(b) * BigInt::from(r.y),
                BigInt::from(r.gcd)
            );
            assert!(r.x.unsigned_abs() <= u128::from(m) && r.y.unsigned_abs() <= u128::from(m));
        }
    }
    let json = serde_json::to_value(extended_gcd(m, 0).unwrap()).unwrap();
    assert_eq!(json["gcd"], m.to_string());
    assert_eq!(json["x"], "1");
}
#[test]
fn inverses_and_powers_match_independent_bounded_oracles() {
    assert_eq!(mod_inverse(3, 11).unwrap(), 4);
    for modulus in 2..50u64 {
        for a in 0..60u64 {
            let oracle = (0..modulus).find(|x| (a * x) % modulus == 1);
            match oracle {
                Some(n) => assert_eq!(mod_inverse(a, modulus).unwrap(), n),
                None => assert_eq!(
                    mod_inverse(a, modulus).unwrap_err().code,
                    ErrorCode::InvalidInput
                ),
            }
        }
    }
    for modulus in 1..25u64 {
        for base in 0..16u64 {
            for exponent in 0..16u64 {
                let mut oracle = 1 % modulus;
                for _ in 0..exponent {
                    oracle = oracle * base % modulus;
                }
                assert_eq!(mod_pow(base, exponent, modulus).unwrap(), oracle);
            }
        }
    }
    assert_eq!(mod_pow(4, 13, 497).unwrap(), 445);
    assert_eq!(mod_pow(0, 0, 7).unwrap(), 1);
    assert_eq!(mod_pow(0, 0, 1).unwrap(), 0);
    let m = u64::MAX;
    assert_eq!(mod_pow(m - 1, m, m).unwrap(), m - 1);
    assert_eq!(mod_pow(m, m, m).unwrap(), 0);
    assert_eq!(mod_inverse(m - 1, m).unwrap(), m - 1);
    for modulus in [0, 1] {
        let e = mod_inverse(1, modulus).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert_eq!(e.parameter, Some("modulus"));
        assert_eq!(e.operation, Some("number.mod_inverse"));
    }
    assert_eq!(
        mod_pow(1, 1, 0).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
}
#[test]
fn decimal_transport_is_strict_and_bounded_and_discovery_is_direct_only() {
    assert_eq!(
        parse_operand("18446744073709551615", "number.gcd", "a").unwrap(),
        u64::MAX
    );
    assert_eq!(
        parse_operand(&"0".repeat(20), "number.gcd", "a").unwrap(),
        0
    );
    for text in [
        "",
        "-1",
        "-18446744073709551615",
        "+1",
        " 1",
        "1 ",
        "1\n",
        "0x10",
        "1.0",
        "1_0",
        "١",
    ] {
        let e = parse_operand(text, "number.gcd", "a").unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert_eq!(e.parameter, Some("a"));
        assert_eq!(e.operation, Some("number.gcd"));
    }
    for text in ["18446744073709551616", "000000000000000000000"] {
        assert_eq!(
            parse_operand(text, "number.gcd", "a").unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    for d in OPERATIONS {
        assert!(
            cube_core::operations::require(d.id)
                .unwrap()
                .discovery()
                .pipeline
                .is_none()
        );
    }
}
