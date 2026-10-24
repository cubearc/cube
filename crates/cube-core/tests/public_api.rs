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
