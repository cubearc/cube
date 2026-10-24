//! Cryptographic hashes and non-cryptographic checksums through maintained crates.

use sha2::{Digest, Sha256};

/// Compute CRC-32/ISO-HDLC (IEEE CRC32) as eight lowercase hexadecimal digits.
///
/// This is the variant used by ZIP and PNG: width 32, polynomial `0x04C11DB7`,
/// reflected input and output, initial value `0xFFFFFFFF`, and final XOR
/// `0xFFFFFFFF`. Leading zeroes are preserved; empty input returns `00000000`.
/// Arbitrary bytes are accepted without UTF-8 conversion.
///
/// CRC32 is a checksum for accidental corruption, not a cryptographic hash or
/// authentication mechanism. The original bytes cannot be recovered from it.
///
/// ```
/// assert_eq!(cube_core::hashing::crc32(b"123456789"), "cbf43926");
/// assert_eq!(cube_core::hashing::crc32(b""), "00000000");
/// ```
pub fn crc32(input: &[u8]) -> String {
    format!("{:08x}", crc32fast::hash(input))
}

/// Compute SHA-256 and return its 64 lowercase hexadecimal digits.
///
/// Hashing is one-way; the digest cannot be decoded into the original bytes.
/// This is an unkeyed digest, not password hashing or message authentication.
///
/// ```
/// assert_eq!(
///     cube_core::hashing::sha256(b"abc"),
///     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
/// );
/// ```
pub fn sha256(input: &[u8]) -> String {
    hex::encode(Sha256::digest(input))
}

/// Static metadata for operations implemented in this module.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "hash.crc32",
        category: "hashing",
        description: "CRC-32/ISO-HDLC checksum, not a cryptographic hash.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash crc32",
        example: "cube hash crc32 --text Hello",
    },
    crate::operations::Descriptor {
        id: "hash.sha256",
        category: "hashing",
        description: "SHA-256 digest of exact bytes, not password hashing.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash sha256",
        example: "cube hash sha256 --text Hello",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_answers_include_empty_and_leading_zeroes() {
        for (input, expected) in [
            (&b"123456789"[..], "cbf43926"),
            (&b""[..], "00000000"),
            (&b"Hello"[..], "f7d18982"),
            (&b"Hello\n"[..], "31963516"),
            (&b"\""[..], "0762ae69"),
            (&b"\0"[..], "d202ef8d"),
            (&[0, 255, 128][..], "81dda740"),
        ] {
            let checksum = crc32(input);
            assert_eq!(checksum, expected);
            assert_eq!(crc32(input), checksum);
            assert_eq!(checksum.len(), 8);
            assert!(
                checksum
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
        }
    }

    #[test]
    fn crc32_accepts_every_byte_and_explicit_utf8() {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(crc32(&bytes), "29058c73");
        assert_eq!(crc32("Olá, 世界 🧊".as_bytes()), "08b240b4");
    }

    #[test]
    fn known_answers() {
        for (input, expected) in [
            (
                "",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                "abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                "Hello",
                "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969",
            ),
            (
                "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ] {
            assert_eq!(sha256(input.as_bytes()), expected);
        }
        assert_eq!(
            sha256(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }
}
