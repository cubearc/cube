//! Hashing through maintained cryptographic implementations.

use sha2::{Digest, Sha256};

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

#[cfg(test)]
mod tests {
    use super::*;

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
