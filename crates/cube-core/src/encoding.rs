//! Standard Base64 and hexadecimal encodings without implicit text conversion.
//!
//! Both encodings are reversible. Decoding restores the original bytes:
//! ```
//! # fn main() -> Result<(), cube_core::Error> {
//! use cube_core::encoding::{hex_decode, hex_encode};
//! let bytes = [0, 128, 255];
//! assert_eq!(hex_decode(&hex_encode(&bytes))?, bytes);
//! # Ok(())
//! # }
//! ```

use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::Error;

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
