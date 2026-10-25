use cube_core::text::{
    self,
    word::{self, Positions},
};
#[test]
fn word_positions_preserve_order_offsets_and_existing_selectors() {
    let input = "Cube Uses Binary Evidence";
    assert_eq!(
        word::extract(input, Positions::Shared(&[1, 3]), 1, "error")
            .unwrap()
            .extracted,
        "CbUeBnEi"
    );
    let last = word::extract(input, Positions::Shared(&[-1]), 1, "error").unwrap();
    assert_eq!(
        last.extracted,
        text::selection::select(input, text::selection::Selector::WordLast).unwrap()
    );
    let result = word::extract("éa\u{a0}🧊b", Positions::Shared(&[-1, 0, -1]), 0, "error").unwrap();
    assert_eq!(result.extracted, "aéab🧊b");
    assert_eq!(result.words[1].selections[1].byte_offset, Some(5));
    assert_eq!(result.words[1].selections[1].scalar_index, Some(3));
    let lists = vec![vec![2, 1, 2], vec![-1]];
    assert_eq!(
        word::extract("ab cd", Positions::PerWord(&lists), 1, "error")
            .unwrap()
            .extracted,
        "babd"
    );
}
#[test]
fn short_words_empty_requests_and_invalid_coordinates() {
    let r = word::extract("A bé", Positions::Shared(&[2]), 1, "skip").unwrap();
    assert_eq!(r.extracted, "é");
    assert_eq!(r.omitted_count, 1);
    assert_eq!(r.words[0].selections[0].scalar, None);
    assert!(word::extract("A bé", Positions::Shared(&[2]), 1, "error").is_err());
    assert_eq!(
        word::extract("", Positions::Shared(&[1]), 1, "error")
            .unwrap()
            .extracted,
        ""
    );
    assert!(word::extract("", Positions::PerWord(&[]), 1, "error").is_ok());
    assert!(word::extract("", Positions::Shared(&[0]), 1, "skip").is_err());
    assert!(word::extract("A", Positions::Shared(&[0]), 1, "skip").is_err());
    assert!(word::extract("A", Positions::Shared(&[1]), 2, "skip").is_err());
    assert!(word::extract("A", Positions::Shared(&[1]), 1, "bad").is_err());
    assert!(word::extract("A", Positions::Shared(&[i64::MIN, i64::MAX]), 1, "error").is_err());
    assert_eq!(
        word::extract("A", Positions::Shared(&[i64::MIN]), 1, "skip")
            .unwrap()
            .omitted_count,
        1
    );
    assert!(word::extract("A", Positions::PerWord(&[]), 1, "error").is_err());
    assert!(
        word::extract("e\u{301},\r\nX", Positions::Shared(&[-1]), 1, "error")
            .unwrap()
            .extracted
            == ",X"
    );
}
#[test]
fn word_coordinate_and_transport_limits() {
    assert!(word::extract("A", Positions::Shared(&vec![1; 65536]), 1, "error").is_ok());
    assert!(word::extract("A A", Positions::Shared(&vec![1; 32769]), 1, "error").is_err());
    assert!(word::extract(&"A ".repeat(65537), Positions::Shared(&[]), 1, "error").is_err());
    assert!(
        word::extract(
            &"A".repeat(text::MAX_INPUT + 1),
            Positions::Shared(&[]),
            1,
            "error"
        )
        .is_err()
    );
    assert!(word::parse_positions("[1,-1,0]").is_ok());
    assert!(word::parse_per_word("[[1],[],[-1]]").is_ok());
    for text in [
        "[1.0]",
        "[true]",
        "[[1]]",
        "[9223372036854775808]",
        "[null]",
    ] {
        assert!(word::parse_positions(text).is_err());
    }
    assert!(word::parse_per_word("[1,2]").is_err());
    assert!(word::parse_positions(&" ".repeat(2 * 1024 * 1024 + 1)).is_err());
}
