use cube_core::ciphers::mono::params::Unmapped;
use cube_core::ciphers::poly::bellaso::{self, Bellaso1553};
use cube_core::errors::ErrorCode;
use cube_core::operations::execution::{Budget, execute};
use serde_json::json;

fn parameters(key: &str) -> Bellaso1553 {
    Bellaso1553 {
        key: key.into(),
        unmapped: Unmapped::Reject,
    }
}

#[test]
fn every_printed_pair_matches_the_full_primary_table_and_is_reciprocal() {
    let top = "ABCDEFGHILM";
    let alphabet = "ABCDEFGHILMNOPQRSTVXYZ";
    for (labels, lower) in [
        ("AB", "NOPQRSTVXYZ"),
        ("CD", "TVXYZNOPQRS"),
        ("EF", "ZNOPQRSTVXY"),
        ("GH", "STVXYZNOPQR"),
        ("IL", "YZNOPQRSTVX"),
        ("MN", "RSTVXYZNOPQ"),
        ("OP", "XYZNOPQRSTV"),
        ("QR", "QRSTVXYZNOP"),
        ("ST", "PQRSTVXYZNO"),
        ("VX", "VXYZNOPQRST"),
        ("YZ", "OPQRSTVXYZN"),
    ] {
        for key in labels.chars() {
            let p = parameters(&key.to_string());
            let plain = format!("{top}{lower}");
            let expected = format!("{lower}{top}");
            assert_eq!(bellaso::transform(&plain, &p).unwrap(), expected, "{key}");
            let out = bellaso::transform(alphabet, &p).unwrap();
            assert_eq!(bellaso::transform(&out, &p).unwrap(), alphabet, "{key}");
        }
    }
}

#[test]
fn canonical_spelling_preservation_and_continuous_key_alignment_are_explicit() {
    let mut p = parameters("ACD");
    assert_eq!(bellaso::transform("", &p).unwrap(), "");
    for invalid in ["U", "J", "K", "W", "a", "é", "A A"] {
        let error = bellaso::transform(invalid, &p).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.operation, Some("cipher.bellaso"));
    }
    p.unmapped = Unmapped::Preserve;
    let plain = "AA é🐈\nAA\0UJV";
    let cipher = bellaso::transform(plain, &p).unwrap();
    assert!(cipher.starts_with("NT é🐈\nTN\0UJ"));
    assert_eq!(bellaso::transform(&cipher, &p).unwrap(), plain);
    assert_eq!(bellaso::transform("AA AA", &p).unwrap(), "NT TN");
    assert_eq!(bellaso::transform("AA", &p).unwrap(), "NT");
    for bad in ["", "u", "U", "J", "K", "W", "A A", "é"] {
        assert_eq!(
            bellaso::transform("A", &parameters(bad)).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    for bad in [
        json!({}),
        json!({"key":"A","profile":"1555"}),
        json!({"key":"A","unmapped":"fold"}),
    ] {
        assert!(
            execute(
                "cipher.bellaso",
                1,
                &json!({"text":"A"}),
                &bad,
                Budget::default()
            )
            .is_err()
        );
    }
}

#[test]
fn direct_rust_callers_enforce_exact_text_and_key_byte_boundaries() {
    let p = parameters(&"A".repeat(65536));
    assert_eq!(
        bellaso::transform(&"A".repeat(1048576), &p).unwrap(),
        "N".repeat(1048576)
    );
    assert_eq!(
        bellaso::transform(&"A".repeat(1048577), &p)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        bellaso::transform("", &parameters(&"A".repeat(65537)))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let preserve = Bellaso1553 {
        key: "A".into(),
        unmapped: Unmapped::Preserve,
    };
    let text = "🐈".repeat(262144);
    assert_eq!(bellaso::transform(&text, &preserve).unwrap(), text);
    assert_eq!(
        bellaso::transform(&(text + "A"), &preserve)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn published_1553_example_executes_as_a_named_reciprocal_profile() {
    let plain = "LARMATATVRCHESCAPARTIRAACINQVEDILVGLIO";
    let cipher = "SYBOVEYLDANVOFSZLPIINCVPNSHMLRNXOIZNRD";
    let parameters = json!({"key":"VIRTVTIOMNIAPARENT"});
    let actual = execute(
        "cipher.bellaso",
        1,
        &json!({"text":plain}),
        &parameters,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(actual, json!({"text":cipher}));
    assert_eq!(
        execute("cipher.bellaso", 1, &actual, &parameters, Budget::default()).unwrap(),
        json!({"text":plain})
    );
}
