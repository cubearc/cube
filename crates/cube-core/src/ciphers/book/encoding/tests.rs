use super::*;
#[test]
fn shared_source_semantics_first_cycles_and_alias_free_inverse() {
    for (scheme, legacy) in [
        (CoordinateScheme::WordCharacter, Scheme::WordCharacter),
        (CoordinateScheme::LineCharacter, Scheme::LineCharacter),
        (
            CoordinateScheme::LineWordCharacter,
            Scheme::LineWordCharacter,
        ),
    ] {
        for index_base in [0, 1] {
            for matching in [MatchPolicy::First, MatchPolicy::RoundRobin] {
                let p = EncodeOptions {
                    scheme,
                    index_base,
                    matching,
                };
                let source = "aé! a😀\r\n\r\na a\rX\n";
                let r = encode(source, "aa😀é!", &p).unwrap();
                assert_eq!(
                    super::super::decode(source, &r.coordinates, legacy, index_base).unwrap(),
                    "aa😀é!"
                );
                assert_eq!(r.coordinate_count, 5);
                assert_eq!(r.source_sha256, crate::hashing::sha256(source.as_bytes()));
                assert!(encode(source, "?", &p).is_err());
                assert_eq!(encode(source, "", &p).unwrap().coordinates, "");
            }
        }
    }
    let p = EncodeOptions {
        scheme: CoordinateScheme::WordCharacter,
        index_base: 1,
        matching: MatchPolicy::RoundRobin,
    };
    assert_eq!(
        encode("aa aa", "aaaaa", &p).unwrap().coordinates,
        "1:1,1:2,2:1,2:2,1:1"
    );
    let first = EncodeOptions {
        matching: MatchPolicy::First,
        ..p
    };
    assert_eq!(
        encode("aa aa", "aaaaa", &first).unwrap().coordinates,
        "1:1,1:1,1:1,1:1,1:1"
    );
    let line = EncodeOptions {
        scheme: CoordinateScheme::LineCharacter,
        index_base: 1,
        matching: MatchPolicy::First,
    };
    assert_eq!(
        encode("a b\r\nc", " \rc", &line).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        encode("a b\nc\rZ", " \rc", &line).unwrap().coordinates,
        "1:2,2:2,2:1"
    );
}
#[test]
fn source_token_definition_and_message_budgets_are_exact() {
    let p = EncodeOptions {
        scheme: CoordinateScheme::WordCharacter,
        index_base: 0,
        matching: MatchPolicy::First,
    };
    let text = "a".repeat(65536);
    assert_eq!(encode("a", &text, &p).unwrap().coordinate_count, 65536);
    assert!(encode("a", &(text + "a"), &p).is_err());
    assert!(encode(&"a".repeat(1048577), "", &p).is_err());
    assert!(encode("a", &"a".repeat(1048577), &p).is_err());
    assert_eq!(
        encode(&"a ".repeat(65536), "a", &p).unwrap().indexed_units,
        65536
    );
    assert!(encode(&"a ".repeat(65537), "", &p).is_err());
    let p = EncodeOptions {
        scheme: CoordinateScheme::LineWordCharacter,
        ..p
    };
    assert_eq!(
        encode(&"a\n".repeat(32768), "a", &p).unwrap().indexed_units,
        65536
    );
    assert!(encode(&"a\n".repeat(32769), "", &p).is_err());
    let p = EncodeOptions { index_base: 2, ..p };
    assert!(encode("", "", &p).is_err());
}
