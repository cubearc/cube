# Hashing and checksums

Use SHA-256 or SHA-512 for one-way content digests. Neither is password hashing
or message authentication. MD5 and SHA-1 are legacy compatibility tools, unsuitable
for collision-resistant security. They help compare historical artifacts without
providing a security guarantee. All consume exact bytes and return lowercase hex:
SHA-256 has 64 digits, SHA-512 128, SHA-1 40 and MD5 32, including leading zeroes.

Checksums detect accidental corruption and are not cryptographic hashes:

| CLI name | Exact variant | Polynomial | Init | Reflected input/output | Final XOR | Hex digits |
| --- | --- | --- | --- | --- | --- | --- |
| `crc32` | CRC-32/ISO-HDLC (IEEE) | `04c11db7` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc32c` | CRC-32/ISCSI (Castagnoli) | `1edc6f41` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc16-ibm3740` | CRC-16/IBM-3740 | `1021` | `ffff` | no/no | `0000` | 4 |

```sh
cube hash sha512 --text abc
cube hash crc32c --text '123456789'
# e3069283
cube hash crc16-ibm3740 --text '123456789'
# 29b1
```

The maintained RustCrypto `sha2`, `sha1` and `md-5` crates supply digests;
`crc` supplies the two new exact CRC variants, while existing CRC32 retains
`crc32fast`. Vectors follow [RFC 6234](https://www.rfc-editor.org/rfc/rfc6234),
[RFC 1321](https://www.rfc-editor.org/rfc/rfc1321) and the
[CRC crate catalog](https://github.com/akhilles/crc-catalog).

New hash CLI commands read at most 16 MiB from stdin; existing SHA-256/CRC32 behavior
is unchanged. Direct Rust/Python functions hash caller-owned byte buffers with
constant digest state and linear work; callers control their buffer size. All
have explicit version-1 byte-to-text adapters under the existing pipeline limits.
Discover these operations under `hashing`; include/exclude newlines deliberately.
