use cube_core::{errors::ErrorCode, symbols::*, text::MAX_INPUT};
#[test]
fn morse_known_answers_table_and_canonicalized_round_trips() {
    assert_eq!(morse_encode("SOS 2").unwrap(), "... --- ... / ..---");
    assert_eq!(morse_decode("... --- ... / ..---").unwrap(), "SOS 2");
    assert_eq!(morse_encode("  sos   2  ").unwrap(), "... --- ... / ..---");
    assert_eq!(morse_encode("aB09").unwrap(), ".- -... ----- ----.");
    assert_eq!(
        morse_decode(&morse_encode(MORSE_ALPHABET).unwrap()).unwrap(),
        MORSE_ALPHABET
    );
    for text in ["", " ", "     ", "abc def 123", " A   B "] {
        let code = morse_encode(text).unwrap();
        assert_eq!(morse_encode(&morse_decode(&code).unwrap()).unwrap(), code);
    }
    for invalid in [
        " .", ". ", ".  .", "/ .", ". /", ". / / .", "......", ".", "...\n", ".../---",
    ] {
        if invalid != "." {
            assert!(morse_decode(invalid).is_err(), "{invalid}");
        }
    }
    for invalid in ["!", "é", "e\u{301}", "A\tB", "A\nB", "🧊"] {
        assert!(morse_encode(invalid).is_err());
    }
    assert_eq!(morse_encode("Aé").unwrap_err().offset, Some(1));
    assert_eq!(morse_decode(". ......").unwrap_err().offset, Some(2));
    assert_eq!(
        morse_encode(&"0".repeat(MAX_INPUT)).unwrap().len(),
        MAX_INPUT * 6 - 1
    );
    assert_eq!(
        morse_decode(&".".repeat(MAX_INPUT + 1)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn braille_all_masks_preserve_exact_bytes_and_report_byte_offsets() {
    let masks: Vec<_> = (0..=255).collect();
    let encoded = braille_encode(&masks).unwrap();
    assert_eq!(braille_decode(&encoded).unwrap(), masks);
    assert_eq!(
        braille_encode(&[0, 1, 2, 4, 8, 16, 32, 64, 128, 255]).unwrap(),
        "⠀⠁⠂⠄⠈⠐⠠⡀⢀⣿"
    );
    assert_eq!(braille_decode("").unwrap(), b"");
    assert_eq!(braille_encode(b"").unwrap(), "");
    assert_eq!(braille_decode("⠁x").unwrap_err().offset, Some(3));
    for input in [" ", "A", "\u{27ff}", "\u{2900}", "🧊"] {
        assert!(braille_decode(input).is_err());
    }
    assert_eq!(
        braille_encode(&vec![0; MAX_INPUT + 1]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
