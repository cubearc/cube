use cube_core::numbers::{self, alphabet::convert};
#[test]
fn exact_alphabets_and_existing_radices_agree() {
    assert_eq!(convert("1010", "01", "abcdefghij", 2, 10).unwrap(), "ba");
    assert_eq!(
        convert("-٠٠١٠", "٠١٢٣٤٥٦٧٨٩", "01", 10, 2).unwrap(),
        "-1010"
    );
    assert_eq!(convert("+000", "01", "αβ", 2, 2).unwrap(), "α");
    assert_eq!(convert("-000", "01", "αβ", 2, 2).unwrap(), "α");
    assert_eq!(convert("A", "aAb", "01", 2, 2).unwrap(), "1");
    for from in [2, 10, 16, 36] {
        for to in [2, 10, 16, 36] {
            for n in [0, 1, 10, 255, u128::MAX] {
                let input = numbers::convert_base(&n.to_string(), 10, from).unwrap();
                assert_eq!(
                    convert(
                        &input,
                        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                        from,
                        to
                    )
                    .unwrap(),
                    numbers::convert_base(&input, from, to).unwrap()
                );
            }
        }
    }
    let symbols = "αβ";
    let encoded = convert(&u128::MAX.to_string(), "0123456789", symbols, 10, 2).unwrap();
    assert_eq!(encoded.chars().count(), 128);
    assert_eq!(
        convert(&encoded, symbols, "0123456789", 2, 10).unwrap(),
        u128::MAX.to_string()
    );
}
#[test]
fn invalid_digits_signs_and_finite_bounds() {
    for alphabet in ["0", "001", "0+", "0-", "0 ", "0\u{a0}"] {
        assert!(convert("0", alphabet, "01", 2, 2).is_err());
    }
    for input in ["", "+", "-", " 1", "1 ", "0x1", "1.0", "2", "--1"] {
        assert!(convert(input, "01", "01", 2, 2).is_err());
    }
    assert!(convert("b", "01b", "01", 2, 2).is_err());
    assert!(convert("1", "01", "01", 3, 2).is_err());
    assert!(convert("1", "01", "01", 2, 1).is_err());
    assert!(convert(&format!("{}0", u128::MAX), "0123456789", "01", 10, 2).is_err());
    assert!(convert(&"1".repeat(129), "01", "01", 2, 2).is_err());
    assert_eq!(convert(&"0".repeat(4096), "01", "01", 2, 2).unwrap(), "0");
    assert!(convert(&"0".repeat(4097), "01", "01", 2, 2).is_err());
    let symbols: String = (0x1f300..0x1f400)
        .map(|c| char::from_u32(c).unwrap())
        .collect();
    assert!(convert(&"🌀".repeat(4096), &symbols, "01", 256, 2).is_ok());
    assert!(convert(&"🌀".repeat(4097), &symbols, "01", 256, 2).is_err());
    assert!(convert("0", &(symbols + "x"), "01", 2, 2).is_err());
    assert_eq!(convert("αx", "αβ", "01", 2, 2).unwrap_err().offset, Some(2));
    // Canonically equivalent whole strings are not implicitly normalized digits.
    assert!(convert("e\u{301}", "éx", "01", 2, 2).is_err());
}
