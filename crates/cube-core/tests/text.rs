use cube_core::{errors::ErrorCode, text::*};
#[test]
fn normalization_has_explicit_forms_version_and_idempotence() {
    assert_eq!(UNICODE_VERSION, (18, 0, 0));
    assert_eq!(normalize("e\u{301}", "NFC").unwrap(), "é");
    assert_eq!(normalize("é", "NFD").unwrap(), "e\u{301}");
    assert_eq!(normalize("ﬀ Ａ ①", "NFKC").unwrap(), "ff A 1");
    assert_eq!(normalize("Å", "NFKD").unwrap(), "A\u{30a}");
    assert_eq!(normalize("각", "NFD").unwrap(), "\u{1100}\u{1161}\u{11a8}");
    assert_eq!(
        normalize("a\u{315}\u{300}", "NFD").unwrap(),
        "a\u{300}\u{315}"
    );
    for form in ["NFC", "NFD", "NFKC", "NFKD"] {
        for text in ["", "é\0\r\n世界🧊", "ﬀＡÅ각", "a\u{315}\u{300}"] {
            let n = normalize(text, form).unwrap();
            assert_eq!(normalize(&n, form).unwrap(), n);
        }
    }
    assert_eq!(normalize("", "nfc").unwrap_err().parameter, Some("form"));
    assert_eq!(
        normalize(&"x".repeat(MAX_INPUT + 1), "NFC")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    // U+FDFA compatibility decomposition expands beyond the separate output budget.
    assert_eq!(
        normalize(&"\u{fdfa}".repeat(MAX_INPUT / 3), "NFKD")
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn reversal_and_selection_use_scalars_without_trimming_or_normalization() {
    for input in ["", "abc", "é世界🧊", "e\u{301}👩\u{200d}💻\0\r\n"] {
        assert_eq!(reverse(&reverse(input).unwrap()).unwrap(), input);
    }
    assert_eq!(reverse("e\u{301}\r\n").unwrap(), "\n\r\u{301}e");
    let text = "Aé🧊e\u{301}Z";
    assert_eq!(
        extract(text, Some(&[5, 1, 1, 2]), None, None, None).unwrap(),
        "Zéé🧊"
    );
    assert_eq!(extract(text, Some(&[]), None, None, None).unwrap(), "");
    assert_eq!(extract("", Some(&[]), None, None, None).unwrap(), "");
    assert_eq!(
        extract(text, None, Some(0), Some(2), None).unwrap(),
        "A🧊\u{301}"
    );
    assert_eq!(extract(text, None, Some(1), Some(2), None).unwrap(), "éeZ");
    assert_eq!(extract(text, None, None, Some(1), Some(6)).unwrap(), text);
    assert_eq!(extract(text, None, Some(6), Some(1), None).unwrap(), "");
    assert_eq!(extract(text, None, Some(4), Some(1), Some(2)).unwrap(), "");
    assert_eq!(
        extract(text, None, None, Some(usize::MAX), None).unwrap(),
        "A"
    );
    assert_eq!(
        extract(text, Some(&[usize::MAX]), None, None, None)
            .unwrap_err()
            .parameter,
        Some("indices")
    );
    for (indices, start, stride, stop) in [
        (None, None, None, None),
        (None, None, Some(0), None),
        (None, Some(7), Some(1), None),
        (None, None, Some(1), Some(7)),
        (Some(&[0][..]), Some(0), None, None),
    ] {
        assert!(extract(text, indices, start, stride, stop).is_err());
    }
    assert_eq!(
        extract("🧊", Some(&vec![0; MAX_INDICES]), None, None, None)
            .unwrap()
            .len(),
        MAX_INDICES * 4
    );
    assert_eq!(
        extract("x", Some(&vec![0; MAX_INDICES + 1]), None, None, None)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(reverse(&"x".repeat(MAX_INPUT + 1)).is_err());
}
