# Hashing and checksums

Hashes and checksums consume exact bytes and return lowercase hexadecimal,
including leading zeros. Include or exclude newlines deliberately.

## Content digests

| Digest | Hex digits | Use |
| --- | --- | --- |
| SHA-256 | 64 | One-way content digest |
| SHA-512 | 128 | One-way content digest |
| SHA-1 | 40 | Legacy compatibility; unsuitable for collision-resistant security |
| MD5 | 32 | Legacy compatibility; unsuitable for collision-resistant security |

These functions do not provide password hashing or message authentication.
RIPEMD-160 and Keccak-256 are described in [Offline key formats](offline-key-formats.md).

## CRC variants

CRCs detect accidental corruption. They are not cryptographic hashes.

| CLI name | Exact variant | Polynomial | Initial value | Reflected input/output | Final XOR | Hex digits |
| --- | --- | --- | --- | --- | --- | --- |
| `crc32` | CRC-32/ISO-HDLC (IEEE) | `04c11db7` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc32c` | CRC-32/ISCSI (Castagnoli) | `1edc6f41` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc16-ibm3740` | CRC-16/IBM-3740 | `1021` | `ffff` | no/no | `0000` | 4 |

For example, calculate two CRCs for the same bytes:

```sh
cube hash crc32c --text '123456789'
# e3069283
cube hash crc16-ibm3740 --text '123456789'
# 29b1
```

## Input limits and pipelines

| Interface | Input handling |
| --- | --- |
| SHA-512, SHA-1, MD5, CRC-32C and CRC-16 CLI commands | At most 16 MiB input |
| SHA-256 and CRC-32 CLI commands | No fixed input-size cap; input is read into memory |
| Direct Rust/Python functions for the digests and CRCs above | Caller-owned buffers; constant digest state and linear work |
| Version-1 pipeline adapters | Bytes to text; subject to shared [pipeline limits](compose-operations.md#limits) |

## Implementations and vectors

Digests use RustCrypto `sha2`, `sha1` and `md-5`. CRC-32 uses `crc32fast`;
CRC-32C and CRC-16 use `crc`.

Vectors follow [RFC 6234](https://www.rfc-editor.org/rfc/rfc6234),
[RFC 1321](https://www.rfc-editor.org/rfc/rfc1321) and the
[CRC catalog](https://github.com/akhilles/crc-catalog).

## Additional digest profiles

All of these profiles accept at most 16 MiB of exact bytes. Results are
fixed-width lowercase hexadecimal, including leading zeros.

| Family | CLI names after `hash` | Output |
| --- | --- | --- |
| SHA-2 | `sha224`, `sha384`, `sha512-224`, `sha512-256` | 56, 96, 56 and 64 digits |
| SHA-3 | `sha3-224`, `sha3-256`, `sha3-384`, `sha3-512` | 56, 64, 96 and 128 digits |
| Keccak | `keccak224`, `keccak384`, `keccak512` | 56, 96 and 128 digits |
| RIPEMD | `ripemd128`, `ripemd256`, `ripemd320` | 32, 64 and 80 digits |
| BLAKE2 | `blake2b512`, `blake2s256` | 128 and 64 digits |
| BLAKE3 | `blake3` | 64 digits |
| Legacy | `md4`, `whirlpool` | 32 and 128 digits |

SHA-512/224 and SHA-512/256 use their FIPS 180-4 initial values. SHA-3 uses
delimited suffix `0x06` and original Keccak uses `0x01`. BLAKE2 and BLAKE3
are unkeyed fixed-output profiles. Whirlpool uses the final 2003 variant.
MD4, Whirlpool and the additional RIPEMD profiles are for compatibility.

```sh
cube hash sha3-256 --text abc
# 3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532
```

## Additional checksum profiles

| CLI name | Calculation | Check value for `123456789` |
| --- | --- | --- |
| `crc8-smbus` | Width 8, polynomial `07`, zero init and XOR, no reflection | `f4` |
| `crc16-arc` | Width 16, polynomial `8005`, zero init and XOR, reflected input and output | `bb3d` |
| `crc64-ecma182` | Width 64, polynomial `42f0e1eba9ea3693`, zero init and XOR, no reflection | `6c40df5f0b497347` |
| `adler32` | RFC 1950. Initial sums 1 and 0 modulo 65,521 | `091e01de` |
| `fletcher16` | Byte words, initial sums 0 and 0 modulo 255 | `1ede` |
| `fletcher32` | Big-endian 16-bit words, initial sums 0 and 0 modulo 65,535 | Requires an even byte count |

Adler and Fletcher place the second sum above the first in the hexadecimal
result. Fletcher-32 refuses an incomplete final word. These checksums detect
corruption within the 16 MiB input limit. They provide no authentication.
