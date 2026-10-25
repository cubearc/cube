//! Cryptographic hashes and non-cryptographic checksums through maintained crates.

use sha2::{Digest, Sha256, Sha512};

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

/// Compute SHA-512 as 128 lowercase hex digits of exact bytes.
/// An unkeyed digest, not password hashing or authentication.
pub fn sha512(input: &[u8]) -> String {
    hex::encode(Sha512::digest(input))
}
/// Legacy SHA-1 digest (40 lowercase hex digits). Unsuitable for collision-resistant
/// security; provided only for identifying historical artifacts and challenge data.
pub fn sha1(input: &[u8]) -> String {
    hex::encode(sha1::Sha1::digest(input))
}
/// Legacy MD5 digest (32 lowercase hex digits). Unsuitable for collision-resistant
/// security; provided only for identifying historical artifacts and challenge data.
pub fn md5(input: &[u8]) -> String {
    hex::encode(md5::Md5::digest(input))
}
/// CRC-32/ISCSI (CRC32C): polynomial 0x1EDC6F41, init/xor 0xFFFFFFFF,
/// reflected input/output. Eight lowercase hex digits, including leading zeroes.
/// A corruption checksum, not a cryptographic hash or authentication.
pub fn crc32c(input: &[u8]) -> String {
    const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);
    format!("{:08x}", CRC.checksum(input))
}
/// CRC-16/IBM-3740: polynomial 0x1021, init 0xFFFF, xor 0, no reflection.
/// Four lowercase hex digits. A corruption checksum, not a cryptographic hash.
pub fn crc16_ibm3740(input: &[u8]) -> String {
    const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_3740);
    format!("{:04x}", CRC.checksum(input))
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
    crate::operations::Descriptor {
        id: "hash.sha512",
        category: "hashing",
        description: "SHA-512 exact-byte digest (128 lowercase hex digits), not password hashing.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash sha512",
        example: "cube hash sha512 --text abc",
    },
    crate::operations::Descriptor {
        id: "hash.sha1",
        category: "hashing",
        description: "Legacy SHA-1 (40 lowercase hex digits); unsuitable for collision-resistant security.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash sha1",
        example: "cube hash sha1 --text abc",
    },
    crate::operations::Descriptor {
        id: "hash.md5",
        category: "hashing",
        description: "Legacy MD5 (32 lowercase hex digits); unsuitable for collision-resistant security.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash md5",
        example: "cube hash md5 --text abc",
    },
    crate::operations::Descriptor {
        id: "hash.crc32c",
        category: "hashing",
        description: "CRC-32/ISCSI (CRC32C) checksum, poly 0x1EDC6F41, init/xor 0xFFFFFFFF, reflected; not cryptographic.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash crc32c",
        example: "cube hash crc32c --text abc",
    },
    crate::operations::Descriptor {
        id: "hash.crc16_ibm3740",
        category: "hashing",
        description: "CRC-16/IBM-3740 checksum, poly 0x1021, init 0xFFFF, xor 0, unreflected; not cryptographic.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash crc16-ibm3740",
        example: "cube hash crc16-ibm3740 --text abc",
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
