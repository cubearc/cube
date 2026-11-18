//! Fixed-width digests and checksums with a 16 MiB input ceiling.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::json;
use sha2::Digest;

pub use super::protocol::MAX_INPUT;

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
/// Named digest or checksum with independent byte and output conventions.
pub enum Profile {
    /// SHA-224. SHA-224 with the initial values specified in FIPS 180-4.
    Sha224,
    /// SHA-384. SHA-384 with the initial values specified in FIPS 180-4.
    Sha384,
    /// SHA-512/224. SHA-512/224 with the initial values specified in FIPS 180-4.
    Sha512224,
    /// SHA-512/256. SHA-512/256 with the initial values specified in FIPS 180-4.
    Sha512256,
    /// SHA3-224. FIPS 202 SHA3-224 with delimited suffix 0x06.
    Sha3224,
    /// SHA3-256. FIPS 202 SHA3-256 with delimited suffix 0x06.
    Sha3256,
    /// SHA3-384. FIPS 202 SHA3-384 with delimited suffix 0x06.
    Sha3384,
    /// SHA3-512. FIPS 202 SHA3-512 with delimited suffix 0x06.
    Sha3512,
    /// Keccak-224. Original Keccak-224 with delimited suffix 0x01.
    Keccak224,
    /// Keccak-384. Original Keccak-384 with delimited suffix 0x01.
    Keccak384,
    /// Keccak-512. Original Keccak-512 with delimited suffix 0x01.
    Keccak512,
    /// RIPEMD-128. RIPEMD-128 compatibility digest.
    Ripemd128,
    /// RIPEMD-256. RIPEMD-256 compatibility digest.
    Ripemd256,
    /// RIPEMD-320. RIPEMD-320 compatibility digest.
    Ripemd320,
    /// BLAKE2b-512. Unkeyed BLAKE2b with a 64-byte digest and sequential default parameters.
    Blake2B512,
    /// BLAKE2s-256. Unkeyed BLAKE2s with a 32-byte digest and sequential default parameters.
    Blake2S256,
    /// BLAKE3. Unkeyed BLAKE3 with the fixed 32-byte digest.
    Blake3,
    /// MD4. RFC 1320 legacy MD4 compatibility digest.
    Md4,
    /// Whirlpool. Final 2003 Whirlpool compatibility digest.
    Whirlpool,
    /// CRC-8/SMBUS. Polynomial 0x07, init 0x00, no reflection, xorout 0x00.
    Crc8Smbus,
    /// CRC-16/ARC. Polynomial 0x8005, init 0x0000, reflected input and output, xorout 0x0000.
    Crc16Arc,
    /// CRC-64/ECMA-182. Polynomial 0x42f0e1eba9ea3693, init 0, no reflection, xorout 0.
    Crc64Ecma182,
    /// Adler-32. RFC 1950 Adler-32 with sums initialized to 1 and 0, modulo 65521.
    Adler32,
    /// Fletcher-16. Eight-bit words and zero-initialized sums modulo 255, second sum in the high byte.
    Fletcher16,
    /// Fletcher-32. Big-endian 16-bit words and zero-initialized sums modulo 65535, second sum in the high half. Odd byte counts fail.
    Fletcher32,
}

impl Profile {
    pub(crate) fn find(operation: &str) -> Option<Self> {
        const ALL: &[Profile] = &[
            Profile::Sha224,
            Profile::Sha384,
            Profile::Sha512224,
            Profile::Sha512256,
            Profile::Sha3224,
            Profile::Sha3256,
            Profile::Sha3384,
            Profile::Sha3512,
            Profile::Keccak224,
            Profile::Keccak384,
            Profile::Keccak512,
            Profile::Ripemd128,
            Profile::Ripemd256,
            Profile::Ripemd320,
            Profile::Blake2B512,
            Profile::Blake2S256,
            Profile::Blake3,
            Profile::Md4,
            Profile::Whirlpool,
            Profile::Crc8Smbus,
            Profile::Crc16Arc,
            Profile::Crc64Ecma182,
            Profile::Adler32,
            Profile::Fletcher16,
            Profile::Fletcher32,
        ];
        ALL.iter()
            .copied()
            .find(|profile| profile.id() == operation)
    }

    /// The exact operation identifier used by discovery and execution.
    pub const fn id(self) -> &'static str {
        PROFILED[self as usize].descriptor.id
    }

    /// The number of lowercase hexadecimal characters in the result.
    pub const fn hex_length(self) -> usize {
        OUTPUT_LENGTHS[self as usize]
    }

    /// Compute the profile over exact bytes. Fletcher-32 requires complete words.
    pub fn digest(self, data: &[u8]) -> Result<String, OperationError> {
        let id = self.id();
        if data.len() > MAX_INPUT {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "digest input exceeds 16 MiB",
            )
            .operation(id));
        }
        Ok(match self {
            Self::Sha224 => hex::encode(sha2::Sha224::digest(data)),
            Self::Sha384 => hex::encode(sha2::Sha384::digest(data)),
            Self::Sha512224 => hex::encode(sha2::Sha512_224::digest(data)),
            Self::Sha512256 => hex::encode(sha2::Sha512_256::digest(data)),
            Self::Sha3224 => hex::encode(sha3::Sha3_224::digest(data)),
            Self::Sha3256 => hex::encode(sha3::Sha3_256::digest(data)),
            Self::Sha3384 => hex::encode(sha3::Sha3_384::digest(data)),
            Self::Sha3512 => hex::encode(sha3::Sha3_512::digest(data)),
            Self::Keccak224 => hex::encode(sha3::Keccak224::digest(data)),
            Self::Keccak384 => hex::encode(sha3::Keccak384::digest(data)),
            Self::Keccak512 => hex::encode(sha3::Keccak512::digest(data)),
            Self::Ripemd128 => hex::encode(ripemd::Ripemd128::digest(data)),
            Self::Ripemd256 => hex::encode(ripemd::Ripemd256::digest(data)),
            Self::Ripemd320 => hex::encode(ripemd::Ripemd320::digest(data)),
            Self::Blake2B512 => hex::encode(blake2::Blake2b512::digest(data)),
            Self::Blake2S256 => hex::encode(blake2::Blake2s256::digest(data)),
            Self::Blake3 => blake3::hash(data).to_hex().to_string(),
            Self::Md4 => hex::encode(md4::Md4::digest(data)),
            Self::Whirlpool => hex::encode(whirlpool::Whirlpool::digest(data)),
            Self::Crc8Smbus => {
                const CRC: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
                format!("{:02x}", CRC.checksum(data))
            }
            Self::Crc16Arc => {
                const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_ARC);
                format!("{:04x}", CRC.checksum(data))
            }
            Self::Crc64Ecma182 => {
                const CRC: crc::Crc<u64> = crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182);
                format!("{:016x}", CRC.checksum(data))
            }
            Self::Adler32 => format!("{:08x}", adler2::adler32_slice(data)),
            Self::Fletcher16 => format!("{:04x}", fletcher(data.iter().copied().map(u32::from), 8)),
            Self::Fletcher32 => {
                if !data.len().is_multiple_of(2) {
                    return Err(OperationError::new(
                        ErrorCode::InvalidInput,
                        "Fletcher-32 requires complete big-endian words",
                    )
                    .operation(id)
                    .offset((data.len() - 1) as u64));
                }
                let words = data
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| u32::from(u16::from_be_bytes(*pair)));
                format!("{:08x}", fletcher(words, 16))
            }
        })
    }
}

fn fletcher(words: impl Iterator<Item = u32>, width: u32) -> u32 {
    let modulo = (1 << width) - 1;
    let (mut first, mut second) = (0, 0);
    for word in words {
        first = (first + word) % modulo;
        second = (second + first) % modulo;
    }
    second << width | first
}

macro_rules! digest_api {
    ($name:ident, $profile:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(data: &[u8]) -> Result<String, OperationError> {
            Profile::$profile.digest(data)
        }
    };
}
digest_api!(
    sha224,
    Sha224,
    "SHA-224 with the initial values specified in FIPS 180-4. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha384,
    Sha384,
    "SHA-384 with the initial values specified in FIPS 180-4. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha512_224,
    Sha512224,
    "SHA-512/224 with the initial values specified in FIPS 180-4. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha512_256,
    Sha512256,
    "SHA-512/256 with the initial values specified in FIPS 180-4. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha3_224,
    Sha3224,
    "FIPS 202 SHA3-224 with delimited suffix 0x06. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha3_256,
    Sha3256,
    "FIPS 202 SHA3-256 with delimited suffix 0x06. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha3_384,
    Sha3384,
    "FIPS 202 SHA3-384 with delimited suffix 0x06. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    sha3_512,
    Sha3512,
    "FIPS 202 SHA3-512 with delimited suffix 0x06. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    keccak224,
    Keccak224,
    "Original Keccak-224 with delimited suffix 0x01. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    keccak384,
    Keccak384,
    "Original Keccak-384 with delimited suffix 0x01. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    keccak512,
    Keccak512,
    "Original Keccak-512 with delimited suffix 0x01. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    ripemd128,
    Ripemd128,
    "RIPEMD-128 compatibility digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    ripemd256,
    Ripemd256,
    "RIPEMD-256 compatibility digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    ripemd320,
    Ripemd320,
    "RIPEMD-320 compatibility digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    blake2b512,
    Blake2B512,
    "Unkeyed BLAKE2b with a 64-byte digest and sequential default parameters. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    blake2s256,
    Blake2S256,
    "Unkeyed BLAKE2s with a 32-byte digest and sequential default parameters. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    blake3,
    Blake3,
    "Unkeyed BLAKE3 with the fixed 32-byte digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    md4,
    Md4,
    "RFC 1320 legacy MD4 compatibility digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    whirlpool,
    Whirlpool,
    "Final 2003 Whirlpool compatibility digest. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    crc8_smbus,
    Crc8Smbus,
    "Polynomial 0x07, init 0x00, no reflection, xorout 0x00. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    crc16_arc,
    Crc16Arc,
    "Polynomial 0x8005, init 0x0000, reflected input and output, xorout 0x0000. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    crc64_ecma182,
    Crc64Ecma182,
    "Polynomial 0x42f0e1eba9ea3693, init 0, no reflection, xorout 0. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    adler32,
    Adler32,
    "RFC 1950 Adler-32 with sums initialized to 1 and 0, modulo 65521. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    fletcher16,
    Fletcher16,
    "Eight-bit words and zero-initialized sums modulo 255, second sum in the high byte. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);
digest_api!(
    fletcher32,
    Fletcher32,
    "Big-endian 16-bit words and zero-initialized sums modulo 65535, second sum in the high half. Odd byte counts fail. Exact bytes up to 16 MiB, fixed lowercase hexadecimal output."
);

#[allow(clippy::too_many_arguments)]
const fn row(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    classification: m::Classification,
    width: usize,
    block_bits: Option<u16>,
    command: &'static str,
    example: &'static str,
    rust_api: &'static str,
    python_api: &'static str,
    references: &'static [&'static str],
) -> m::Entry {
    let (family, limitations) = match classification {
        m::Classification::Checksum => (
            "checksums",
            "Corruption checksum without cryptographic authentication.",
        ),
        m::Classification::LegacyDigest => (
            "hash_functions",
            "Compatibility digest unsuitable for new security designs.",
        ),
        _ => (
            "hash_functions",
            "Unkeyed digest without authentication or password derivation.",
        ),
    };
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "hashing",
            description,
            input_type: "bytes",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family,
            aliases: &[],
            security: m::Security {
                classification,
                limitations,
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exact bytes.",
                key: "No key.",
                key_advancement: "Not applicable.",
                block_bits,
                output_bits: Some(width as u16),
            },
            resources: INPUT_LIMIT,
            resource_notes: "Input capped at 16 MiB. Work is linear in input length. Output is fixed-width lowercase hexadecimal.",
            rust_api,
            python_api,
            references,
        }),
    }
}
const INPUT_LIMIT: &[m::ResourceBound] = &[m::ResourceBound {
    name: "input_bytes",
    maximum: MAX_INPUT as u64,
    unit: "bytes",
    scope: m::Scope::Core,
}];
const PROFILED: [m::Entry; 25] = [
    row(
        "hash.sha224",
        "SHA-224",
        "SHA-224 with the initial values specified in FIPS 180-4.",
        m::Classification::UnkeyedDigest,
        224,
        Some(512),
        "hash sha224",
        "cube hash sha224 --text abcd",
        "cube_core::hashing::sha224",
        "sha224",
        &["https://csrc.nist.gov/pubs/fips/180-4/upd1/final"],
    ),
    row(
        "hash.sha384",
        "SHA-384",
        "SHA-384 with the initial values specified in FIPS 180-4.",
        m::Classification::UnkeyedDigest,
        384,
        Some(1024),
        "hash sha384",
        "cube hash sha384 --text abcd",
        "cube_core::hashing::sha384",
        "sha384",
        &["https://csrc.nist.gov/pubs/fips/180-4/upd1/final"],
    ),
    row(
        "hash.sha512_224",
        "SHA-512/224",
        "SHA-512/224 with the initial values specified in FIPS 180-4.",
        m::Classification::UnkeyedDigest,
        224,
        Some(1024),
        "hash sha512-224",
        "cube hash sha512-224 --text abcd",
        "cube_core::hashing::sha512_224",
        "sha512_224",
        &["https://csrc.nist.gov/pubs/fips/180-4/upd1/final"],
    ),
    row(
        "hash.sha512_256",
        "SHA-512/256",
        "SHA-512/256 with the initial values specified in FIPS 180-4.",
        m::Classification::UnkeyedDigest,
        256,
        Some(1024),
        "hash sha512-256",
        "cube hash sha512-256 --text abcd",
        "cube_core::hashing::sha512_256",
        "sha512_256",
        &["https://csrc.nist.gov/pubs/fips/180-4/upd1/final"],
    ),
    row(
        "hash.sha3_224",
        "SHA3-224",
        "FIPS 202 SHA3-224 with delimited suffix 0x06.",
        m::Classification::UnkeyedDigest,
        224,
        Some(1152),
        "hash sha3-224",
        "cube hash sha3-224 --text abcd",
        "cube_core::hashing::sha3_224",
        "sha3_224",
        &["https://csrc.nist.gov/pubs/fips/202/final"],
    ),
    row(
        "hash.sha3_256",
        "SHA3-256",
        "FIPS 202 SHA3-256 with delimited suffix 0x06.",
        m::Classification::UnkeyedDigest,
        256,
        Some(1088),
        "hash sha3-256",
        "cube hash sha3-256 --text abcd",
        "cube_core::hashing::sha3_256",
        "sha3_256",
        &["https://csrc.nist.gov/pubs/fips/202/final"],
    ),
    row(
        "hash.sha3_384",
        "SHA3-384",
        "FIPS 202 SHA3-384 with delimited suffix 0x06.",
        m::Classification::UnkeyedDigest,
        384,
        Some(832),
        "hash sha3-384",
        "cube hash sha3-384 --text abcd",
        "cube_core::hashing::sha3_384",
        "sha3_384",
        &["https://csrc.nist.gov/pubs/fips/202/final"],
    ),
    row(
        "hash.sha3_512",
        "SHA3-512",
        "FIPS 202 SHA3-512 with delimited suffix 0x06.",
        m::Classification::UnkeyedDigest,
        512,
        Some(576),
        "hash sha3-512",
        "cube hash sha3-512 --text abcd",
        "cube_core::hashing::sha3_512",
        "sha3_512",
        &["https://csrc.nist.gov/pubs/fips/202/final"],
    ),
    row(
        "hash.keccak224",
        "Keccak-224",
        "Original Keccak-224 with delimited suffix 0x01.",
        m::Classification::UnkeyedDigest,
        224,
        Some(1152),
        "hash keccak224",
        "cube hash keccak224 --text abcd",
        "cube_core::hashing::keccak224",
        "keccak224",
        &["https://keccak.team/keccak_specs_summary.html"],
    ),
    row(
        "hash.keccak384",
        "Keccak-384",
        "Original Keccak-384 with delimited suffix 0x01.",
        m::Classification::UnkeyedDigest,
        384,
        Some(832),
        "hash keccak384",
        "cube hash keccak384 --text abcd",
        "cube_core::hashing::keccak384",
        "keccak384",
        &["https://keccak.team/keccak_specs_summary.html"],
    ),
    row(
        "hash.keccak512",
        "Keccak-512",
        "Original Keccak-512 with delimited suffix 0x01.",
        m::Classification::UnkeyedDigest,
        512,
        Some(576),
        "hash keccak512",
        "cube hash keccak512 --text abcd",
        "cube_core::hashing::keccak512",
        "keccak512",
        &["https://keccak.team/keccak_specs_summary.html"],
    ),
    row(
        "hash.ripemd128",
        "RIPEMD-128",
        "RIPEMD-128 compatibility digest.",
        m::Classification::LegacyDigest,
        128,
        Some(512),
        "hash ripemd128",
        "cube hash ripemd128 --text abcd",
        "cube_core::hashing::ripemd128",
        "ripemd128",
        &["https://homes.esat.kuleuven.be/~bosselae/ripemd160.html"],
    ),
    row(
        "hash.ripemd256",
        "RIPEMD-256",
        "RIPEMD-256 compatibility digest.",
        m::Classification::LegacyDigest,
        256,
        Some(512),
        "hash ripemd256",
        "cube hash ripemd256 --text abcd",
        "cube_core::hashing::ripemd256",
        "ripemd256",
        &["https://homes.esat.kuleuven.be/~bosselae/ripemd160.html"],
    ),
    row(
        "hash.ripemd320",
        "RIPEMD-320",
        "RIPEMD-320 compatibility digest.",
        m::Classification::LegacyDigest,
        320,
        Some(512),
        "hash ripemd320",
        "cube hash ripemd320 --text abcd",
        "cube_core::hashing::ripemd320",
        "ripemd320",
        &["https://homes.esat.kuleuven.be/~bosselae/ripemd160.html"],
    ),
    row(
        "hash.blake2b512",
        "BLAKE2b-512",
        "Unkeyed BLAKE2b with a 64-byte digest and sequential default parameters.",
        m::Classification::UnkeyedDigest,
        512,
        Some(1024),
        "hash blake2b512",
        "cube hash blake2b512 --text abcd",
        "cube_core::hashing::blake2b512",
        "blake2b512",
        &["https://www.rfc-editor.org/rfc/rfc7693"],
    ),
    row(
        "hash.blake2s256",
        "BLAKE2s-256",
        "Unkeyed BLAKE2s with a 32-byte digest and sequential default parameters.",
        m::Classification::UnkeyedDigest,
        256,
        Some(512),
        "hash blake2s256",
        "cube hash blake2s256 --text abcd",
        "cube_core::hashing::blake2s256",
        "blake2s256",
        &["https://www.rfc-editor.org/rfc/rfc7693"],
    ),
    row(
        "hash.blake3",
        "BLAKE3",
        "Unkeyed BLAKE3 with the fixed 32-byte digest.",
        m::Classification::UnkeyedDigest,
        256,
        Some(512),
        "hash blake3",
        "cube hash blake3 --text abcd",
        "cube_core::hashing::blake3",
        "blake3",
        &["https://github.com/BLAKE3-team/BLAKE3-specs"],
    ),
    row(
        "hash.md4",
        "MD4",
        "RFC 1320 legacy MD4 compatibility digest.",
        m::Classification::LegacyDigest,
        128,
        Some(512),
        "hash md4",
        "cube hash md4 --text abcd",
        "cube_core::hashing::md4",
        "md4",
        &["https://www.rfc-editor.org/rfc/rfc1320"],
    ),
    row(
        "hash.whirlpool",
        "Whirlpool",
        "Final 2003 Whirlpool compatibility digest.",
        m::Classification::LegacyDigest,
        512,
        Some(512),
        "hash whirlpool",
        "cube hash whirlpool --text abcd",
        "cube_core::hashing::whirlpool",
        "whirlpool",
        &["https://github.com/RustCrypto/hashes/tree/whirlpool-v0.11.0/whirlpool"],
    ),
    row(
        "hash.crc8_smbus",
        "CRC-8/SMBUS",
        "Polynomial 0x07, init 0x00, no reflection, xorout 0x00.",
        m::Classification::Checksum,
        8,
        None,
        "hash crc8-smbus",
        "cube hash crc8-smbus --text abcd",
        "cube_core::hashing::crc8_smbus",
        "crc8_smbus",
        &["https://docs.rs/crc-catalog/latest/crc_catalog/algorithm/"],
    ),
    row(
        "hash.crc16_arc",
        "CRC-16/ARC",
        "Polynomial 0x8005, init 0x0000, reflected input and output, xorout 0x0000.",
        m::Classification::Checksum,
        16,
        None,
        "hash crc16-arc",
        "cube hash crc16-arc --text abcd",
        "cube_core::hashing::crc16_arc",
        "crc16_arc",
        &["https://docs.rs/crc-catalog/latest/crc_catalog/algorithm/"],
    ),
    row(
        "hash.crc64_ecma182",
        "CRC-64/ECMA-182",
        "Polynomial 0x42f0e1eba9ea3693, init 0, no reflection, xorout 0.",
        m::Classification::Checksum,
        64,
        None,
        "hash crc64-ecma182",
        "cube hash crc64-ecma182 --text abcd",
        "cube_core::hashing::crc64_ecma182",
        "crc64_ecma182",
        &["https://docs.rs/crc-catalog/latest/crc_catalog/algorithm/"],
    ),
    row(
        "hash.adler32",
        "Adler-32",
        "RFC 1950 Adler-32 with sums initialized to 1 and 0, modulo 65521.",
        m::Classification::Checksum,
        32,
        None,
        "hash adler32",
        "cube hash adler32 --text abcd",
        "cube_core::hashing::adler32",
        "adler32",
        &["https://www.rfc-editor.org/rfc/rfc1950"],
    ),
    row(
        "hash.fletcher16",
        "Fletcher-16",
        "Eight-bit words and zero-initialized sums modulo 255, second sum in the high byte.",
        m::Classification::Checksum,
        16,
        None,
        "hash fletcher16",
        "cube hash fletcher16 --text abcd",
        "cube_core::hashing::fletcher16",
        "fletcher16",
        &["https://doi.org/10.1109/TCOM.1982.1095369"],
    ),
    row(
        "hash.fletcher32",
        "Fletcher-32",
        "Big-endian 16-bit words and zero-initialized sums modulo 65535, second sum in the high half. Odd byte counts fail.",
        m::Classification::Checksum,
        32,
        None,
        "hash fletcher32",
        "cube hash fletcher32 --text abcd",
        "cube_core::hashing::fletcher32",
        "fletcher32",
        &["https://doi.org/10.1109/TCOM.1982.1095369"],
    ),
];
const OUTPUT_LENGTHS: [usize; 25] = [
    56, 96, 56, 64, 56, 64, 96, 128, 56, 96, 128, 32, 64, 80, 128, 64, 64, 32, 128, 2, 4, 16, 8, 4,
    8,
];

pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

fn output_schema<const LENGTH: usize>() -> schemars::Schema {
    let mut schema = c::schema::<String>().to_value();
    schema["minLength"] = json!(LENGTH);
    schema["maxLength"] = json!(LENGTH);
    schema["pattern"] = json!("^[0-9a-f]+$");
    schemars::Schema::try_from(schema).expect("exact lowercase hexadecimal output")
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: c::bytes_schema::<MAX_INPUT>,
    required: true,
};
const fn contract(
    profile: Profile,
    outputs: &'static [c::Port],
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: PROFILED[profile as usize].descriptor,
        inputs: &[INPUT],
        parameters: c::schema::<EmptyParameters>,
        outputs,
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(profile.hex_length() as u64),
            max_work: Some(MAX_INPUT as u64),
            max_items: None,
            max_memory_bytes: None,
        },
        fixture,
    }
}
#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParameters {}
macro_rules! digest_contract {
    ($profile:ident, $length:literal) => {
        contract(
            Profile::$profile,
            &[c::Port { name: "output", semantic_type: "text", schema: output_schema::<$length>, required: true }],
            || Ok(c::Fixture {
                inputs: json!({"input": {"base64": "YWJjZA=="}}),
                parameters: json!({}),
                outputs: json!({"output": Profile::$profile.digest(b"abcd")?}),
            }),
        )
    };
}
const CONTRACTS: &[c::Entry] = &[
    digest_contract!(Sha224, 56),
    digest_contract!(Sha384, 96),
    digest_contract!(Sha512224, 56),
    digest_contract!(Sha512256, 64),
    digest_contract!(Sha3224, 56),
    digest_contract!(Sha3256, 64),
    digest_contract!(Sha3384, 96),
    digest_contract!(Sha3512, 128),
    digest_contract!(Keccak224, 56),
    digest_contract!(Keccak384, 96),
    digest_contract!(Keccak512, 128),
    digest_contract!(Ripemd128, 32),
    digest_contract!(Ripemd256, 64),
    digest_contract!(Ripemd320, 80),
    digest_contract!(Blake2B512, 128),
    digest_contract!(Blake2S256, 64),
    digest_contract!(Blake3, 64),
    digest_contract!(Md4, 32),
    digest_contract!(Whirlpool, 128),
    digest_contract!(Crc8Smbus, 2),
    digest_contract!(Crc16Arc, 4),
    digest_contract!(Crc64Ecma182, 16),
    digest_contract!(Adler32, 8),
    digest_contract!(Fletcher16, 4),
    digest_contract!(Fletcher32, 8),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
