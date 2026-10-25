use cube_core::{
    errors::ErrorCode,
    numbers::constants::{self, Constant, DigitOptions},
};

#[test]
fn certified_defaults_and_selections_match_research_known_answers() {
    let cases = [
        (
            Constant::Pi,
            "3",
            "1415926535897932",
            "2724b1d7ccb7b276c1e7e72e665439089ecd182d872a672cf1d43a7c29d1ec63",
        ),
        (
            Constant::E,
            "2",
            "7182818284590452",
            "6f9693c7decc3f6204646c2678d7297b2e4bec7e2e9d1d812f3a1e83586ff826",
        ),
        (
            Constant::Phi,
            "1",
            "6180339887498948",
            "2d5d2d980a5fd80ce0b402d582b5dcad17eff0e9892e8e9411bb33545bc15cf3",
        ),
        (
            Constant::Sqrt2,
            "1",
            "4142135623730950",
            "4fa86ae93a34ebc6f34112d9f8621c62b2239329f324144b41c454da3d1dc74b",
        ),
    ];
    for (constant, integer, first, hash) in cases {
        assert_eq!(Constant::parse(constant.name()).unwrap(), constant);
        let result = constants::digits(constant, DigitOptions::default()).unwrap();
        assert_eq!(result.digits, first);
        assert_eq!(result.integer_part, integer);
        assert!(result.fractional_zero_based);
        assert_eq!(result.radix, 10);
        assert_eq!(result.max_digits, 256);
        assert_eq!(result.prefix_sha256, hash);
        let whole = constants::digits(
            constant,
            DigitOptions {
                count: Some(256),
                ..Default::default()
            },
        )
        .unwrap();
        let picked = constants::digits(
            constant,
            DigitOptions {
                indices: Some(&[255, 0, 2, 0]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            picked.digits.as_bytes(),
            [
                whole.digits.as_bytes()[255],
                first.as_bytes()[0],
                first.as_bytes()[2],
                first.as_bytes()[0]
            ]
        );
        let more = constants::digits(
            constant,
            DigitOptions {
                count: Some(256),
                max_digits: 512,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(more.digits, whole.digits);
        let short = constants::digits(
            constant,
            DigitOptions {
                count: Some(1),
                max_digits: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(short.digits, &first[..1]);
    }
}

#[test]
fn overrides_remain_bounded_and_invalid_selections_fail_before_computation() {
    let fail = |options| constants::digits(Constant::Pi, options).unwrap_err();
    for max_digits in [0, 4097, usize::MAX] {
        assert_eq!(
            fail(DigitOptions {
                max_digits,
                ..Default::default()
            })
            .parameter,
            Some("max_digits")
        );
    }
    for options in [
        DigitOptions {
            start: Some(256),
            count: Some(1),
            ..Default::default()
        },
        DigitOptions {
            indices: Some(&[256]),
            ..Default::default()
        },
        DigitOptions {
            start: Some(usize::MAX),
            count: Some(1),
            ..Default::default()
        },
        DigitOptions {
            start: Some(0),
            indices: Some(&[]),
            ..Default::default()
        },
        DigitOptions {
            count: Some(0),
            indices: Some(&[]),
            ..Default::default()
        },
    ] {
        assert_eq!(fail(options).code, ErrorCode::InvalidParameter);
    }
    let many = vec![0; 1025];
    assert_eq!(
        fail(DigitOptions {
            indices: Some(&many),
            ..Default::default()
        })
        .code,
        ErrorCode::ResourceLimit
    );
    let repeated = vec![0; 1024];
    assert_eq!(
        constants::digits(
            Constant::Pi,
            DigitOptions {
                indices: Some(&repeated),
                ..Default::default()
            }
        )
        .unwrap()
        .digits,
        "1".repeat(1024)
    );
    assert_eq!(
        constants::digits(
            Constant::Pi,
            DigitOptions {
                start: Some(256),
                count: Some(0),
                ..Default::default()
            }
        )
        .unwrap()
        .digits,
        ""
    );
    assert_eq!(
        constants::digits(
            Constant::Pi,
            DigitOptions {
                indices: Some(&[]),
                ..Default::default()
            }
        )
        .unwrap()
        .digits,
        ""
    );
    assert!(Constant::parse("PI").is_err());
    assert_eq!(constants::parse_indices("2,0,2").unwrap(), [2, 0, 2]);
    assert!(constants::parse_indices("").unwrap().is_empty());
    for invalid in [
        "-1",
        "1.5",
        "1,",
        " 1",
        "1, 2",
        "0xff",
        "999999999999999999999999999999999999999999",
    ] {
        assert!(constants::parse_indices(invalid).is_err(), "{invalid}");
    }
    assert!(constants::parse_indices(&"0,".repeat(1024)).is_err());
    assert!(constants::parse_indices(&"0".repeat(8193)).is_err());
}

#[test]
fn hard_ceiling_prefixes_match_independent_fraction_and_decimal_references() {
    // Independently checked with exact Fraction Machin intervals for pi and
    // 4160-digit Decimal exp/sqrt for e/sqrt2/phi. No reference dataset bundled.
    let cases = [
        (
            Constant::Pi,
            "4c68ac4b26a190dee1f40eb52d4b3e60e47e257136effa441fa5bb92c1f51bb4",
            "3606598764861179",
            "6",
        ),
        (
            Constant::E,
            "f7bd4554441748f9277165f900c47c2692c60bde351ab8e8859496039d072801",
            "7193648002173285",
            "0",
        ),
        (
            Constant::Phi,
            "6765e4f69076cf3841563918ae9368b6849eab51abcf7e3c5569d3931fb6c6fe",
            "4918323709085748",
            "3",
        ),
        (
            Constant::Sqrt2,
            "d85f0e71b4de78c1177de563d04803b299b9c43de1c7ca7180cf76fe0e31ebb6",
            "9439569367020794",
            "2",
        ),
    ];
    for (constant, hash, last, at256) in cases {
        let result = constants::digits(
            constant,
            DigitOptions {
                count: Some(4096),
                max_digits: 4096,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result.prefix_sha256, hash);
        assert_eq!(result.digits.len(), 4096);
        assert_eq!(&result.digits[4080..], last);
        assert_eq!(&result.digits[256..257], at256);
        assert!(serde_json::to_vec(&result).unwrap().len() < 16 * 1024);
        let boundary = constants::digits(
            constant,
            DigitOptions {
                indices: Some(&[4095, 256, 0, 4095]),
                max_digits: 4096,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            boundary.digits.as_bytes(),
            [
                result.digits.as_bytes()[4095],
                result.digits.as_bytes()[256],
                result.digits.as_bytes()[0],
                result.digits.as_bytes()[4095]
            ]
        );
    }
    assert_eq!(
        constants::digits(
            Constant::Sqrt2,
            DigitOptions {
                start: Some(4096),
                count: Some(0),
                max_digits: 4096,
                ..Default::default()
            }
        )
        .unwrap()
        .digits,
        ""
    );
    assert!(
        constants::digits(
            Constant::Sqrt2,
            DigitOptions {
                indices: Some(&[4096]),
                max_digits: 4096,
                ..Default::default()
            }
        )
        .is_err()
    );
}
