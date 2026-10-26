use std::error::Error as _;

use cube_core::{Error, binary, ciphers, encoding, hashing};

#[test]
fn arbitrary_bytes_round_trip() -> Result<(), Error> {
    let bytes: Vec<u8> = (0..=255).collect();
    assert_eq!(
        encoding::base32_decode(&encoding::base32_encode(&bytes))?,
        bytes
    );
    assert_eq!(
        encoding::base58_decode(&encoding::base58_encode(&bytes))?,
        bytes
    );
    assert_eq!(
        encoding::base64_decode(&encoding::base64_encode(&bytes))?,
        bytes
    );
    assert_eq!(encoding::hex_decode(&encoding::hex_encode(&bytes))?, bytes);
    for key in [&b"x"[..], &b"longer key"[..], &[0, 128, 255][..]] {
        let encrypted = binary::xor(&bytes, key)?;
        assert_eq!(binary::xor(&encrypted, key)?, bytes);
        assert_eq!(binary::xor(&bytes, key)?, encrypted);
    }
    assert_eq!(
        encoding::base32_encode(&bytes),
        encoding::base32_encode(&bytes)
    );
    assert_eq!(
        encoding::base58_encode(&bytes),
        encoding::base58_encode(&bytes)
    );
    assert_eq!(
        encoding::base64_encode(&bytes),
        encoding::base64_encode(&bytes)
    );
    assert_eq!(encoding::hex_encode(&bytes), encoding::hex_encode(&bytes));
    assert_eq!(hashing::sha256(&bytes), hashing::sha256(&bytes));
    assert_eq!(hashing::crc32(&bytes), "29058c73");
    assert_eq!(hashing::crc32(&bytes), hashing::crc32(&bytes));
    Ok(())
}

#[test]
fn unicode_is_only_text_at_the_boundary() -> Result<(), Error> {
    let text = "Olá, 世界 🧊";
    assert_eq!(
        encoding::base32_decode(&encoding::base32_encode(text.as_bytes()))?,
        text.as_bytes()
    );
    assert_eq!(
        encoding::base58_decode(&encoding::base58_encode(text.as_bytes()))?,
        text.as_bytes()
    );
    assert_eq!(
        encoding::base64_decode(&encoding::base64_encode(text.as_bytes()))?,
        text.as_bytes()
    );
    assert_eq!(
        encoding::hex_decode(&encoding::hex_encode(text.as_bytes()))?,
        text.as_bytes()
    );
    assert_eq!(ciphers::rot13(&ciphers::rot13(text)), text);
    assert_eq!(hashing::crc32(text.as_bytes()), "08b240b4");
    assert_ne!(hashing::sha256(text.as_bytes()), hashing::sha256(b"Ola"));
    Ok(())
}

#[test]
fn caesar_preserves_unicode_across_inverse_shifts() {
    let text = "Attack at dawn! Olá, 世界 🧊 e\u{301}\0\r\n";
    for shift in [i32::MIN, -27, -13, -1, 0, 3, 13, 26, 27, i32::MAX] {
        let encoded = ciphers::caesar_encode(text, shift);
        assert_eq!(ciphers::caesar_decode(&encoded, shift), text);
        assert_eq!(ciphers::caesar_encode(text, shift), encoded);
    }
    assert_eq!(ciphers::caesar_encode(text, 13), ciphers::rot13(text));
    assert_eq!(ciphers::caesar_decode(text, 13), ciphers::rot13(text));
}

#[test]
fn errors_keep_their_causes_and_context() {
    for result in [
        encoding::base32_decode("!"),
        encoding::base58_decode("0"),
        encoding::base64_decode("!"),
        encoding::hex_decode("zz"),
    ] {
        let error = result.unwrap_err();
        assert!(error.source().is_some());
        assert!(error.to_string().starts_with("invalid "));
    }
    assert_eq!(Error::EmptyKey.to_string(), "XOR key must not be empty");
    assert!(Error::EmptyKey.source().is_none());
}

#[test]
fn legacy_errors_keep_stable_categories_and_operation_context() {
    let failures = [
        cube_core::encoding::base32_decode("!").unwrap_err(),
        cube_core::encoding::base58_decode("0").unwrap_err(),
        cube_core::encoding::base64_decode("!").unwrap_err(),
        cube_core::encoding::hex_decode("!").unwrap_err(),
        cube_core::binary::xor(b"", b"").unwrap_err(),
    ];
    for (error, id) in failures.into_iter().zip([
        "encoding.base32.decode",
        "encoding.base58.decode",
        "encoding.base64.decode",
        "encoding.hex.decode",
        "binary.xor",
    ]) {
        assert_eq!(error.code(), cube_core::errors::ErrorCode::InvalidInput);
        assert_eq!(error.operation_id(), id);
    }
}

#[test]
fn file_parameter_descriptors_cover_exact_types_defaults_and_limits() {
    use cube_core::operations::{self, ParameterValue as V};
    assert_eq!(operations::catalog().len(), 154);
    assert_eq!(operations::filtered(Some("file")).unwrap().len(), 11);
    assert!(operations::filtered(None).is_ok());
    assert!(operations::filtered(Some("missing")).is_err());
    assert!(operations::require("missing").is_err());
    let scan = operations::require("file.signature.scan").unwrap();
    assert!(
        scan.validate(&[
            ("offset", V::Unsigned(u64::MAX)),
            ("max_results", V::Unsigned(1)),
            ("format", V::Text("png"))
        ])
        .is_ok()
    );
    assert!(scan.validate(&[("offset", V::Integer(-1))]).is_err());
    assert!(scan.validate(&[("format", V::Text("missing"))]).is_err());
    assert!(
        scan.validate(&[("max_results", V::Unsigned(10001))])
            .is_err()
    );
    assert!(
        operations::require("file.hex.inspect")
            .unwrap()
            .validate(&[("width", V::Unsigned(0))])
            .is_err()
    );
    assert!(
        operations::require("file.strings.extract")
            .unwrap()
            .validate(&[("min_length", V::Unsigned(0))])
            .is_err()
    );
    let extraction = operations::require("file.range.extract").unwrap();
    assert!(extraction.validate(&[]).is_err());
    assert!(
        extraction
            .validate(&[("offset", V::Integer(0)), ("length", V::Unsigned(0))])
            .is_ok()
    );
}

#[test]
fn metadata_is_readable_without_rust_debug_syntax() {
    use cube_core::operations::{ParameterType as T, ParameterValue as V};
    for (value, expected) in [
        (V::Integer(-1), "-1"),
        (V::Unsigned(u64::MAX), "18446744073709551615"),
        (V::Boolean(true), "true"),
        (V::Text("hello"), "\"hello\""),
        (V::Bytes(b"\0\xff"), "hex:00ff"),
    ] {
        assert_eq!(value.to_string(), expected);
    }
    for kind in [
        T::Integer { min: -1, max: 1 },
        T::Unsigned { min: 0, max: 1 },
        T::Boolean,
        T::Text { choices: &[] },
        T::Text {
            choices: &["raw", "hex"],
        },
        T::Bytes { min_length: 1 },
    ] {
        assert!(!kind.to_string().is_empty());
    }
}

#[test]
fn ordered_parameter_lists_validate_choices_ranges_lengths_and_uniqueness() {
    use cube_core::operations::{ParameterValue as V, require};
    let histogram = require("image.histogram").unwrap();
    assert!(
        histogram
            .validate(&[
                ("channels", V::TextList(&["red", "blue"])),
                ("region", V::UnsignedList(&[0, 0, 1, 1]))
            ])
            .is_ok()
    );
    for values in [&["red", "red"][..], &["unknown"][..], &[][..]] {
        assert!(
            histogram
                .validate(&[("channels", V::TextList(values))])
                .is_err()
        );
    }
    assert!(
        histogram
            .validate(&[("region", V::UnsignedList(&[0, 0, 1]))])
            .is_err()
    );
    assert!(
        histogram
            .validate(&[("region", V::UnsignedList(&[0, 0, 1, u64::MAX]))])
            .is_err()
    );
    assert_eq!(V::TextList(&["red", "blue"]).to_string(), "red, blue");
    assert_eq!(V::UnsignedList(&[0, 7]).to_string(), "[0, 7]");
    for parameter in histogram.parameters {
        assert!(!parameter.constraints.to_string().is_empty());
    }
}

#[test]
fn rotation_metadata_rejects_noncanonical_angles_and_wrong_types() {
    use cube_core::operations::{ParameterValue, describe};
    let operation = describe("image.rotate").unwrap();
    assert!(
        operation
            .validate(&[("degrees", ParameterValue::Unsigned(90))])
            .is_ok()
    );
    assert!(
        operation
            .validate(&[("degrees", ParameterValue::Integer(270))])
            .is_ok()
    );
    for value in [
        ParameterValue::Unsigned(45),
        ParameterValue::Integer(-90),
        ParameterValue::Text("90"),
    ] {
        assert!(operation.validate(&[("degrees", value)]).is_err());
    }
}
