use cube_core::text::{self, values};
#[test]
fn word_lengths_codepoints_and_sums_are_distinct_exact_views() {
    let lengths = values::word_lengths("A é e\u{301} 🧊!").unwrap();
    assert_eq!(
        lengths.iter().map(|w| w.scalar_length).collect::<Vec<_>>(),
        [1, 1, 2, 2]
    );
    assert_eq!(lengths[1].byte_offset, 2);
    assert_eq!(lengths[2].scalar_index, 4);
    for format in ["decimal", "hex", "unicode"] {
        let points = values::codepoints("A é", format).unwrap();
        assert_eq!(
            points.iter().map(|p| p.value).collect::<Vec<_>>(),
            [65, 32, 233]
        );
        assert_eq!(points[2].byte_offset, 2);
    }
    assert_eq!(
        values::codepoints("é", "hex").unwrap()[0].presentation,
        "E9"
    );
    assert_eq!(
        values::codepoints("é", "unicode").unwrap()[0].presentation,
        "U+00E9"
    );
    let words = values::word_values("A é", None, "error", true, u64::MAX).unwrap();
    assert_eq!(
        words.iter().map(|w| w.sum.unwrap()).collect::<Vec<_>>(),
        [65, 233]
    );
    assert_eq!(words[1].values[0].scalar, 'é');
    assert!(values::word_lengths("").unwrap().is_empty());
    assert!(values::codepoints("", "decimal").unwrap().is_empty());
    assert!(
        values::word_values("", None, "error", true, 0)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn custom_values_keep_exact_u64s_omissions_and_checked_sums() {
    let table = [('A', u64::MAX), ('b', 0)];
    assert_eq!(
        values::word_values("A", Some(&table), "error", true, u64::MAX).unwrap()[0].sum,
        Some(u64::MAX)
    );
    assert!(values::word_values("AA", Some(&table), "error", true, u64::MAX).is_err());
    assert!(values::word_values("A", Some(&table), "error", true, u64::MAX - 1).is_err());
    let result = values::word_values("Ab?", Some(&table), "skip", true, u64::MAX).unwrap();
    assert_eq!(result[0].omitted_count, 1);
    assert_eq!(result[0].values[2].value, None);
    assert_eq!(result[0].sum, Some(u64::MAX));
    assert!(values::word_values("a", Some(&table), "error", false, u64::MAX).is_err());
    assert!(
        values::word_values("", Some(&[('A', 0), ('A', 1)]), "error", false, u64::MAX).is_err()
    );
    assert!(values::word_values("", None, "bad", false, u64::MAX).is_err());
    assert!(values::word_values("", None, "error", false, 1).is_err());
}
#[test]
fn numeric_record_and_table_limits_reject_excess() {
    assert!(values::codepoints(&"A".repeat(65536), "decimal").is_ok());
    assert!(values::codepoints(&"A".repeat(65537), "decimal").is_err());
    assert!(values::word_values(&"A".repeat(65537), None, "error", false, u64::MAX).is_err());
    assert!(values::codepoints("", "unknown").is_err());
    assert!(values::word_lengths(&"A".repeat(text::MAX_INPUT + 1)).is_err());
    for json in [
        "[[\"A\",true]]",
        "[[\"A\",1.0]]",
        "[[\"A\",-1]]",
        "[[\"AB\",1]]",
    ] {
        assert!(values::parse_table(json).is_err());
    }
    assert!(values::parse_table("[[\"A\",18446744073709551615]]").is_ok());
    assert!(values::parse_table(&" ".repeat(65537)).is_err());
    let table: Vec<_> = (0x1000..0x2001)
        .map(|c| (char::from_u32(c).unwrap(), 0))
        .collect();
    assert!(values::word_values("", Some(&table), "skip", false, u64::MAX).is_err());
}
