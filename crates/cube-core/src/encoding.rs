//! Base32, Bitcoin Base58, Base64, and hexadecimal without implicit text conversion.
//!
//! These encodings are reversible. Decoding restores the original bytes:
//! ```
//! # fn main() -> Result<(), cube_core::Error> {
//! use cube_core::encoding::{hex_decode, hex_encode};
//! let bytes = [0, 128, 255];
//! assert_eq!(hex_decode(&hex_encode(&bytes))?, bytes);
//! # Ok(())
//! # }
//! ```

use base64::{Engine as _, engine::general_purpose::STANDARD};
use data_encoding::BASE32;

use crate::Error;

/// Encode bytes as RFC 4648 standard Base32 using uppercase A–Z and digits 2–7.
///
/// Padding (`=`) is added when the input length is not a multiple of five bytes.
/// Decoding the result restores the original bytes.
///
/// ```
/// assert_eq!(cube_core::encoding::base32_encode(b"foo"), "MZXW6===");
/// ```
pub fn base32_encode(input: &[u8]) -> String {
    BASE32.encode(input)
}

/// Decode canonical, padded RFC 4648 standard Base32 into bytes.
///
/// # Errors
/// Returns [`Error::InvalidBase32`] for lowercase letters, whitespace, invalid
/// alphabet characters, missing or incorrect padding, or nonzero unused trailing
/// bits. Input is not normalized. Empty input decodes to empty bytes.
///
/// ```
/// # fn main() -> Result<(), cube_core::Error> {
/// assert_eq!(cube_core::encoding::base32_decode("AD7Q====")?, [0, 255]);
/// # Ok(())
/// # }
/// ```
pub fn base32_decode(input: &str) -> Result<Vec<u8>, Error> {
    // The codec accepts concatenated padded blocks; canonical input pads only the final block.
    if let Some(position) = input.find('=')
        && position < input.len().saturating_sub(8)
    {
        return Err(Error::InvalidBase32(data_encoding::DecodeError {
            position,
            kind: data_encoding::DecodeKind::Padding,
        }));
    }
    BASE32
        .decode(input.as_bytes())
        .map_err(Error::InvalidBase32)
}

/// Encode bytes using the case-sensitive Bitcoin Base58 alphabet:
/// `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`.
///
/// Each leading zero byte becomes a leading `1`; empty input produces empty text.
/// Decoding restores the original bytes. No padding or Base58Check checksum is added.
///
/// ```
/// assert_eq!(cube_core::encoding::base58_encode(b"Hello World"), "JxF12TrwUP45BMd");
/// assert_eq!(cube_core::encoding::base58_encode(&[0, 0, 1]), "112");
/// ```
pub fn base58_encode(input: &[u8]) -> String {
    bs58::encode(input)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_string()
}

/// Decode case-sensitive Bitcoin Base58 into the original bytes.
///
/// Every leading `1` restores one leading zero byte; empty text produces empty
/// bytes. Input is not trimmed or normalized. No Base58Check checksum is verified.
///
/// # Errors
/// Returns [`Error::InvalidBase58`] for whitespace, non-ASCII input, or characters
/// outside the Bitcoin alphabet, including `0`, `O`, `I`, and `l`.
///
/// ```
/// # fn main() -> Result<(), cube_core::Error> {
/// assert_eq!(cube_core::encoding::base58_decode("15Q")?, [0, 255]);
/// # Ok(())
/// # }
/// ```
pub fn base58_decode(input: &str) -> Result<Vec<u8>, Error> {
    bs58::decode(input)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()
        .map_err(Error::InvalidBase58)
}

/// Encode bytes as padded RFC 4648 standard Base64.
///
/// ```
/// assert_eq!(cube_core::encoding::base64_encode(b"Hello"), "SGVsbG8=");
/// ```
pub fn base64_encode(input: &[u8]) -> String {
    STANDARD.encode(input)
}

/// Decode canonical, padded RFC 4648 standard Base64 into bytes.
///
/// # Errors
/// Returns [`Error::InvalidBase64`] for invalid alphabet, padding, or trailing bits.
/// Whitespace and the URL-safe alphabet are not accepted.
///
/// ```
/// # fn main() -> Result<(), cube_core::Error> {
/// assert_eq!(cube_core::encoding::base64_decode("AP8=")?, [0, 255]);
/// # Ok(())
/// # }
/// ```
pub fn base64_decode(input: &str) -> Result<Vec<u8>, Error> {
    STANDARD.decode(input).map_err(Error::InvalidBase64)
}

/// Encode bytes as lowercase hexadecimal digits, without a prefix.
pub fn hex_encode(input: &[u8]) -> String {
    hex::encode(input)
}

/// Decode an even number of ASCII hexadecimal digits, accepting either case.
///
/// # Errors
/// Returns [`Error::InvalidHex`] for odd length or invalid digits. Prefixes and
/// whitespace are not accepted.
pub fn hex_decode(input: &str) -> Result<Vec<u8>, Error> {
    hex::decode(input).map_err(Error::InvalidHex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_known_answers() -> Result<(), Error> {
        for (plain, encoded) in [("", ""), ("a", "2g"), ("Hello World", "JxF12TrwUP45BMd")] {
            assert_eq!(base58_encode(plain.as_bytes()), encoded);
            assert_eq!(base58_decode(encoded)?, plain.as_bytes());
        }
        assert_eq!(base58_decode("A")?, [9]);
        assert_eq!(base58_decode("a")?, [33]);
        Ok(())
    }

    #[test]
    fn base58_preserves_leading_zero_bytes() -> Result<(), Error> {
        for (bytes, encoded) in [
            (&[0][..], "1"),
            (&[0, 0, 0, 0][..], "1111"),
            (&[0, 0, 1][..], "112"),
            (&[0, 255][..], "15Q"),
            (&[255][..], "5Q"),
        ] {
            assert_eq!(base58_encode(bytes), encoded);
            assert_eq!(base58_decode(encoded)?, bytes);
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_base58_without_trimming() {
        for input in [
            "0",
            "O",
            "I",
            "l",
            "!",
            "2g=",
            "+",
            "/",
            "é",
            "世界",
            "🧊",
            " 2g",
            "2g ",
            "2 g",
            "2g\n",
            "\r2g",
            "2\tg",
            "\0",
            "2g\u{00a0}",
        ] {
            assert!(
                matches!(base58_decode(input), Err(Error::InvalidBase58(_))),
                "accepted {input:?}"
            );
        }
    }

    #[test]
    fn base32_rfc4648_vectors() -> Result<(), Error> {
        for (plain, encoded) in [
            ("", ""),
            ("f", "MY======"),
            ("fo", "MZXQ===="),
            ("foo", "MZXW6==="),
            ("foob", "MZXW6YQ="),
            ("fooba", "MZXW6YTB"),
            ("foobar", "MZXW6YTBOI======"),
        ] {
            assert_eq!(base32_encode(plain.as_bytes()), encoded);
            assert_eq!(base32_decode(encoded)?, plain.as_bytes());
        }
        assert_eq!(base32_encode(&[0, 255]), "AD7Q====");
        assert_eq!(base32_decode("AD7Q====")?, [0, 255]);
        Ok(())
    }

    #[test]
    fn rejects_noncanonical_base32() {
        for input in [
            "mzxw6===",
            "MZxW6===",
            "!",
            "0A======",
            "1A======",
            "8A======",
            "9A======",
            "é",
            "世界",
            " MY======",
            "MY======\n",
            "MY=====\t",
            "MY",
            "MY=====",
            "MY=======",
            "=Y======",
            "MY=====A",
            "========",
            "A=======",
            "AAA=====",
            "AAAAAA==",
            "MZXW6YTB=",
            "MY======MY======",
            "MY======MZXW6YTB",
        ] {
            assert!(
                matches!(base32_decode(input), Err(Error::InvalidBase32(_))),
                "accepted {input:?}"
            );
        }
    }

    #[test]
    fn rejects_nonzero_base32_trailing_bits_for_each_padding_length() {
        for input in [
            "MZ======",
            "MZXR====",
            "MZXW7===",
            "MZXW6YR=",
            "MZXW6YTBMZ======",
        ] {
            assert!(
                matches!(base32_decode(input), Err(Error::InvalidBase32(_))),
                "accepted {input:?}"
            );
        }
    }

    #[test]
    fn rfc4648_vectors() -> Result<(), Error> {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(plain.as_bytes()), encoded);
            assert_eq!(base64_decode(encoded)?, plain.as_bytes());
        }
        Ok(())
    }

    #[test]
    fn rejects_noncanonical_base64() {
        for input in ["!", "Zg", "Zg=", "Zh==", "Zg===", "Zg==\n", "-_8=", "é"] {
            assert!(matches!(base64_decode(input), Err(Error::InvalidBase64(_))));
        }
    }

    #[test]
    fn hex_vectors() -> Result<(), Error> {
        assert_eq!(hex_encode(b"Hello"), "48656c6c6f");
        assert_eq!(hex_encode(&[0, 15, 255]), "000fff");
        assert_eq!(hex_decode("00aAFF")?, [0, 170, 255]);
        assert_eq!(hex_encode(b""), "");
        assert!(hex_decode("")?.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_invalid_hex() {
        for input in ["0", "zz", "0x00", "00 ", "é", "\n"] {
            assert!(matches!(hex_decode(input), Err(Error::InvalidHex(_))));
        }
    }
}
