use cube_core::{errors::ErrorCode, numbers::*};
#[test]
fn integer_radices_keep_exact_magnitudes_and_canonical_signs() {
    for text in ["0", "-0", "+000", "-000"] {
        assert_eq!(convert_base(text, 10, 2).unwrap(), "0");
    }
    assert_eq!(convert_base("+00ff", 16, 2).unwrap(), "11111111");
    assert_eq!(convert_base("-z", 36, 10).unwrap(), "-35");
    assert_eq!(
        convert_base("18446744073709551617", 10, 16).unwrap(),
        "10000000000000001"
    );
    let maximum = u128::MAX.to_string();
    assert_eq!(convert_base(&maximum, 10, 2).unwrap(), "1".repeat(128));
    assert_eq!(
        convert_base(&format!("-{maximum}"), 10, 2).unwrap(),
        format!("-{}", "1".repeat(128))
    );
    for source in 2..=36 {
        for target in 2..=36 {
            for number in ["0", "1", "-35", "18446744073709551617", maximum.as_str()] {
                let input = convert_base(number, 10, source).unwrap();
                let output = convert_base(&input, source, target).unwrap();
                assert_eq!(convert_base(&output, target, 10).unwrap(), number);
            }
        }
    }
    assert_eq!(convert_base(&"0".repeat(MAX_DIGITS), 10, 36).unwrap(), "0");
}
#[test]
fn invalid_integer_syntax_radices_and_budgets_are_errors() {
    for input in [
        "", "+", "-", " 1", "1 ", "1\n", "1_0", "0x10", "1.0", "١", "--1", "+-1", "G",
    ] {
        assert_eq!(
            convert_base(input, 16, 10).unwrap_err().code,
            ErrorCode::InvalidInput,
            "{input}"
        );
    }
    for (from, to) in [(1, 10), (37, 10), (10, 1), (10, 37)] {
        assert_eq!(
            convert_base("", from, to).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    assert_eq!(
        convert_base(&"0".repeat(MAX_DIGITS + 1), 10, 2)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        convert_base("340282366920938463463374607431768211456", 10, 2)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(convert_base("-12x", 10, 2).unwrap_err().offset, Some(3));
}
