use cube_core::{errors::ErrorCode, numbers::digital::*};
fn repeated_sum(mut n: u128) -> u8 {
    while n >= 10 {
        let mut sum = 0;
        while n != 0 {
            sum += n % 10;
            n /= 10;
        }
        n = sum;
    }
    n as u8
}
#[test]
fn exact_roots_match_repeated_decimal_sums_and_signed_magnitude_bounds() {
    for n in 0..10000 {
        assert_eq!(digital_root(n, 10).unwrap(), repeated_sum(n));
    }
    for n in [
        0,
        9,
        10,
        38,
        999,
        u128::MAX,
        i128::MAX as u128,
        1u128 << 127,
    ] {
        let expected = repeated_sum(n);
        assert_eq!(digital_root(n, 10).unwrap(), expected);
        for sign in ["", "+", "-"] {
            assert_eq!(
                from_decimal(&format!("{sign}000{n}"), 10).unwrap(),
                expected
            );
        }
    }
    for (input, result) in [
        ("0", 0),
        ("9", 9),
        ("10", 1),
        ("38", 2),
        ("999", 9),
        ("-38", 2),
        ("-000", 0),
    ] {
        assert_eq!(from_decimal(input, 10).unwrap(), result);
    }
    assert_eq!(
        from_decimal(&format!("-{}38", "0".repeat(4094)), 10).unwrap(),
        2
    );
    assert_eq!(from_decimal(&"0".repeat(4096), 10).unwrap(), 0);
}
#[test]
fn syntax_magnitude_radix_and_resource_failures_are_contextual() {
    for input in [
        "", "+", "-", " 1", "1 ", "1\n", "1_0", "0x10", "1.0", "١", "--1", "+-1", "-12x",
    ] {
        let e = from_decimal(input, 10).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput, "{input}");
        assert_eq!(e.operation, Some("number.digital_root"));
    }
    assert_eq!(from_decimal("-12x", 10).unwrap_err().offset, Some(3));
    for input in [
        "0".repeat(4097),
        format!("+{}", "0".repeat(4097)),
        "340282366920938463463374607431768211456".into(),
        "-340282366920938463463374607431768211456".into(),
    ] {
        assert_eq!(
            from_decimal(&input, 10).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    for radix in [0, 1, 2, 9, 11, 36, u32::MAX] {
        let e = digital_root(38, radix).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParameter);
        assert_eq!(e.parameter, Some("radix"));
        assert!(from_decimal("38", radix).is_err());
    }
    let d = cube_core::operations::require("number.digital_root").unwrap();
    assert_eq!(d.reversible, Some(false));
    assert!(d.discovery().pipeline.is_none());
}
