use cube_core::{
    ciphers::book::{self, Scheme},
    errors::ErrorCode,
    operations, text,
};
#[test]
fn exact_book_coordinates_reuse_word_and_line_semantics() {
    let source = "Cube Uses\r\n\nBinary Evidence!\n雪 e\u{301}";
    assert_eq!(
        book::decode(source, "1:1,2:1,3:1,4:1", Scheme::WordCharacter, 1).unwrap(),
        "CUBE"
    );
    assert_eq!(
        book::decode(source, "0:0,1:0,2:0,3:0", Scheme::WordCharacter, 0).unwrap(),
        "CUBE"
    );
    assert_eq!(
        book::decode(source, "1:1,3:1,4:3,4:4", Scheme::LineCharacter, 1).unwrap(),
        "CBe\u{301}"
    );
    assert_eq!(
        book::decode(
            source,
            " 1 : 1 : 1 , 3:2:9,4:2:2,4:1:1 ",
            Scheme::LineWordCharacter,
            1
        )
        .unwrap(),
        "C!\u{301}雪"
    );
    assert_eq!(
        book::decode(" !a\r\n\nZ\rX\n", "1:1,1:2,3:2", Scheme::LineCharacter, 1).unwrap(),
        " !\r"
    );
    assert_eq!(
        book::decode("  a\t雪", "02:01,2:1", Scheme::WordCharacter, 1).unwrap(),
        "雪雪"
    );
    assert_eq!(
        book::decode("", " \t\n", Scheme::WordCharacter, 1).unwrap(),
        ""
    );
    assert!(book::Scheme::parse("guess").is_err());
    let descriptor = operations::describe("cipher.book.decode").unwrap();
    assert!(descriptor.discovery().pipeline.is_none());
}
#[test]
fn invalid_book_coordinates_fail_without_guessing_or_source_disclosure() {
    for coord in [
        "1",
        "1:1,",
        ":1",
        "1:",
        "+1:1",
        "-1:1",
        "0:1",
        "1:0",
        "2:1",
        "1:7",
        "1:1:1",
        "1:1,,1:1",
        "184467440737095516160:1",
        "1.0:1",
    ] {
        let error = book::decode("secret", coord, Scheme::WordCharacter, 1).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput, "{coord}");
        assert!(!error.to_string().contains("secret"));
        assert!(error.to_string().contains("coordinate"));
    }
    assert!(book::decode("\na", "1:1", Scheme::LineCharacter, 1).is_err());
    assert!(book::decode("a", "2:1:1", Scheme::LineWordCharacter, 1).is_err());
    assert!(book::decode("a", "1:1", Scheme::WordCharacter, 2).is_err());
}
#[test]
fn book_source_coordinate_and_index_budgets_are_enforced() {
    let max = "雪".repeat(text::MAX_INPUT / 3);
    let coords = vec!["1:1"; book::MAX_COORDINATES].join(",");
    assert_eq!(
        book::decode(&max, &coords, Scheme::WordCharacter, 1).unwrap(),
        "雪".repeat(book::MAX_COORDINATES)
    );
    let too_many = format!("{coords},1:1");
    assert_eq!(
        book::decode("a", &too_many, Scheme::WordCharacter, 1)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        book::decode(
            &"x".repeat(text::MAX_INPUT + 1),
            "",
            Scheme::WordCharacter,
            1
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        book::decode(
            "a",
            &" ".repeat(text::MAX_INPUT + 1),
            Scheme::WordCharacter,
            1
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let words = "a ".repeat(book::MAX_UNITS);
    assert!(book::decode(&words, "65536:1", Scheme::WordCharacter, 1).is_ok());
    assert!(book::decode(&(words + "b"), "1:1", Scheme::WordCharacter, 1).is_err());
    assert!(
        book::decode(
            &"\n".repeat(book::MAX_UNITS + 1),
            "",
            Scheme::LineCharacter,
            1
        )
        .is_err()
    );
    assert!(
        book::decode(
            &"a\n".repeat(book::MAX_UNITS / 2 + 1),
            "",
            Scheme::LineWordCharacter,
            1
        )
        .is_err()
    );
}
