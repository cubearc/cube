use cube_core::{ciphers::*, errors::ErrorCode, text::MAX_INPUT};
#[test]
fn known_answers_and_exact_unicode_preservation() {
    assert_eq!(atbash("ABC xyz").unwrap(), "ZYX cba");
    assert_eq!(
        vigenere_encode("ATTACKATDAWN", "LEMON").unwrap(),
        "LXFOPVEFRNHR"
    );
    assert_eq!(
        vigenere_decode("LXFOPVEFRNHR", "lemon").unwrap(),
        "ATTACKATDAWN"
    );
    assert_eq!(vigenere_encode("A!éA🧊A", "BCD").unwrap(), "B!éC🧊D");
    assert_eq!(beaufort("A", "B").unwrap(), "B");
    assert_eq!(beaufort("B", "A").unwrap(), "Z");
    assert_eq!(beaufort("ABC xyz", "B").unwrap(), "BAZ edc");
    assert_eq!(affine_encode("AFFINECIPHER", 5, 8).unwrap(), "IHHWVCSWFRCP");
    let input = "AaZz e\u{301} K Ａ ﬀ 世界 🧊\0\r\n";
    assert_eq!(atbash(&atbash(input).unwrap()).unwrap(), input);
    for key in ["A", "z", "LeMoN", "ABCABCABCABCABCABCABCABCABCABCABCABC"] {
        assert_eq!(
            vigenere_decode(&vigenere_encode(input, key).unwrap(), key).unwrap(),
            input
        );
        assert_eq!(
            beaufort(&beaufort(input, key).unwrap(), key).unwrap(),
            input
        );
    }
    for a in [1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25] {
        for b in 0..26 {
            assert_eq!(
                affine_decode(&affine_encode(input, a, b).unwrap(), a, b).unwrap(),
                input
            );
        }
    }
    assert_eq!(atbash("").unwrap(), "");
    assert_eq!(vigenere_encode("", "A").unwrap(), "");
    assert_eq!(beaufort("", "A").unwrap(), "");
    assert_eq!(affine_encode("", 1, 0).unwrap(), "");
}
#[test]
fn invalid_keys_and_limits_fail_with_safe_context() {
    for key in ["", "A B", "é", "A1", "A\0"] {
        for f in [vigenere_encode, vigenere_decode, beaufort] {
            let error = f("", key).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidParameter);
            assert_eq!(error.parameter, Some("key"));
        }
    }
    for (a, b) in [(-1, 0), (26, 0), (0, 0), (2, 0), (13, 0), (1, -1), (1, 26)] {
        assert!(affine_encode("", a, b).is_err());
        assert!(affine_decode("", a, b).is_err());
    }
    assert_eq!(
        vigenere_encode("A", &"B".repeat(classical::MAX_KEY)).unwrap(),
        "B"
    );
    assert_eq!(
        beaufort("A", &"B".repeat(classical::MAX_KEY + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let boundary = "A".repeat(MAX_INPUT);
    assert_eq!(atbash(&boundary).unwrap().len(), MAX_INPUT);
    assert_eq!(
        atbash(&(boundary + "A")).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
