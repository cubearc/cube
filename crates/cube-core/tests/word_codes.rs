//! Whole-word paginated codes and an ambiguity-preserving ACA phrase inverse.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn key_phrase_published_forward_and_repeated_letters_preserve_ambiguity() {
    let p = json!({"phrase":"GIVEMELIBERTYORGIVEMEDEATH"});
    let result = execution::execute(
        "cipher.key_phrase.encode",
        1,
        &json!({"text":"A CIPHERTEXT LETTER MAY STAND FOR MORE THAN ONE PLAINTEXT LETTER."}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        result["text"],
        "G VBGIMVMMAM TMMMMV YGT EMGOE ERV YRVM MIGO ROM GTGBOMMAM TMMMMV."
    );
    let result = execution::execute(
        "cipher.key_phrase.decode",
        1,
        &json!({"text":"I"}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["text"], serde_json::Value::Null);
    assert_eq!(result["records"][0]["alternatives"], json!(["B", "H", "Q"]));
}
#[test]
fn arnold_profile_selects_whole_words_from_exact_caller_pages() {
    let pages = json!(["THE ROAD\nTHE CUBE", "CUBE 🐈\r\nROAD"]);
    let p = json!({"profile":"arnold_paged_words_v1","source_id":"synthetic:two-pages-v1","index_base":1,"choice":"round_robin"});
    let encoded = execution::execute(
        "cipher.arnold.encode",
        1,
        &json!({"pages":pages,"words":["THE","THE","CUBE","CUBE","🐈"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        encoded["coordinates"],
        json!([{"page":1,"line":1,"word":1},{"page":1,"line":2,"word":1},{"page":1,"line":2,"word":2},{"page":2,"line":1,"word":1},{"page":2,"line":1,"word":2}])
    );
    let decoded = execution::execute(
        "cipher.arnold.decode",
        1,
        &json!({"pages":pages,"coordinates":encoded["coordinates"]}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        decoded["words"],
        json!(["THE", "THE", "CUBE", "CUBE", "🐈"])
    );
    assert_eq!(decoded["source_sha256"], encoded["source_sha256"]);
}

#[test]
fn phrase_unique_cases_retain_punctuation_and_unknown_letters_fail() {
    use cube_core::ciphers::mono::key_phrase as k;
    let p = k::Parameters {
        phrase: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into(),
    };
    let result = k::encode("*Ab - 19!", &p).unwrap();
    assert_eq!(result.text, "*AB - 19!");
    assert!(!result.ambiguous);
    assert_eq!(
        k::decode(&result.text, &p).unwrap().text.as_deref(),
        Some("*AB - 19!")
    );
    assert_eq!(k::decode("", &p).unwrap().text.as_deref(), Some(""));
    let repeated = k::Parameters {
        phrase: "A".repeat(26),
    };
    let report = k::decode("A A", &repeated).unwrap();
    assert!(report.ambiguous && report.text.is_none());
    assert_eq!(
        report.records[0].alternatives,
        (b'A'..=b'Z')
            .map(|c| char::from(c).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(report.records[1].alternatives, [" "]);
    assert_eq!(report.records[2].byte_offset, 2);
    let failure = k::decode("Z", &repeated).unwrap_err();
    assert_eq!(failure.operation, Some("cipher.key_phrase.decode"));
    assert_eq!(failure.offset, Some(0));
}
#[test]
fn phrase_bad_definitions_input_and_report_expansion_fail_predictably() {
    use cube_core::{ciphers::mono::key_phrase as k, errors::ErrorCode};
    for phrase in [
        "A".repeat(25),
        "A".repeat(27),
        "a".repeat(26),
        "1".repeat(26),
    ] {
        assert!(k::encode("A", &k::Parameters { phrase }).is_err());
    }
    let p = k::Parameters {
        phrase: "A".repeat(26),
    };
    for text in ["é", "\0", "\n"] {
        assert_eq!(
            k::encode(text, &p).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
    assert_eq!(
        k::encode(&"A".repeat(65537), &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        k::decode(&"A".repeat(65536), &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
fn arnold_parameters() -> cube_core::ciphers::book::words::Parameters {
    serde_json::from_value(json!({"profile":"arnold_paged_words_v1","source_id":"synthetic:two-pages-v1","index_base":1,"choice":"round_robin"})).unwrap()
}
#[test]
fn page_word_profile_keeps_case_punctuation_unicode_and_source_identity_exact() {
    use cube_core::ciphers::book::{MatchPolicy, words as w};
    let mut p = arnold_parameters();
    p.index_base = 0;
    p.choice = MatchPolicy::First;
    let pages = vec!["\nThe\u{a0}e\u{301}\r\nThe 猫.".into()];
    let words = vec!["The".into(), "e\u{301}".into(), "The".into(), "猫.".into()];
    let result = w::encode(&pages, &words, &p).unwrap();
    assert_eq!(
        result
            .coordinates
            .iter()
            .map(|c| [c.page, c.line, c.word])
            .collect::<Vec<_>>(),
        [[0, 1, 0], [0, 1, 1], [0, 1, 0], [0, 2, 1]]
    );
    assert_eq!(
        w::decode(&pages, &result.coordinates, &p).unwrap().words,
        words
    );
    for word in ["the", "é", "猫"] {
        assert!(w::encode(&pages, &[word.into()], &p).is_err());
    }
    let altered = vec![pages[0].replace("\r\n", "\n")];
    assert_ne!(
        w::encode(&altered, &[], &p).unwrap().source_sha256,
        result.source_sha256
    );
}
#[test]
fn page_word_profile_refuses_unavailable_coordinates_and_implicit_conventions() {
    use cube_core::ciphers::book::words as w;
    let mut p = arnold_parameters();
    assert!(w::encode(&[], &[], &p).unwrap().words.is_empty());
    let pages = vec!["ONE\n\nTWO\n".into()];
    for (page, line, word) in [
        (0, 1, 1),
        (1, 0, 1),
        (1, 1, 0),
        (1, 2, 1),
        (1, 4, 1),
        (2, 1, 1),
    ] {
        let failure = w::decode(&pages, &[w::Coordinate { page, line, word }], &p).unwrap_err();
        assert_eq!(failure.operation, Some("cipher.arnold.decode"));
    }
    p.index_base = 2;
    assert!(w::encode(&pages, &[], &p).is_err());
    p.index_base = 1;
    p.source_id.clear();
    assert!(w::encode(&pages, &[], &p).is_err());
}
#[test]
fn page_word_source_unit_and_exact_serialized_byte_limits_are_enforced() {
    use cube_core::{ciphers::book::words as w, errors::ErrorCode};
    let p = arnold_parameters();
    assert!(w::encode(&["X ".repeat(65534)], &["X".into()], &p).is_ok());
    assert_eq!(
        w::encode(&["X ".repeat(65535)], &[], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert!(w::encode(&[" ".repeat(1048576 - 4)], &[], &p).is_ok());
    assert_eq!(
        w::encode(&[" ".repeat(1048576 - 3)], &[], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(w::encode(&["A".repeat(256)], &["A".repeat(256)], &p).is_ok());
    assert_eq!(
        w::encode(&["A".repeat(257)], &[], &p).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn page_word_report_stops_before_unbounded_repeated_word_expansion() {
    use cube_core::{ciphers::book::words as w, errors::ErrorCode};
    let p = arnold_parameters();
    let pages = vec!["A".repeat(256)];
    let coordinate = w::Coordinate {
        page: 1,
        line: 1,
        word: 1,
    };
    assert_eq!(
        w::decode(&pages, &vec![coordinate; 10000], &p)
            .unwrap()
            .words
            .len(),
        10000
    );
    assert_eq!(
        w::decode(&pages, &vec![coordinate; 30000], &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
