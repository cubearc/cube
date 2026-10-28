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
