# Key derivation

Derive exact raw key bytes from a password or input key material and explicit
parameters. The CLI reads secret bytes from a regular file and writes raw
bytes to standard output. Redirect to the intended private destination.
Python returns bytes from its `kdf_*` functions. The JavaScript SDK uses the
same typed named-port contract. The operations are direct-only and ineligible
for automatic capture.

## Profiles and limits

| Command after `cipher kdf` | Required settings | Output |
| --- | --- | --- |
| `pbkdf2` | `sha256` or `sha512`, iteration count | 1–1,024 bytes |
| `hkdf` | `sha256` or `sha512`, explicit salt and info | 1–1,024 bytes |
| `scrypt` | Power-of-two N, r, p | 16–1,024 bytes |
| `argon2id` | Memory in KiB, iteration count, lanes | 16–1,024 bytes |
| `evp-bytes-to-key` | `md5` or `sha256`, repetition count, key and IV lengths | Key followed by IV in the CLI, at most 1,024 bytes combined |

Password or input key material is capped at 1 MiB. PBKDF2 and HKDF use up
to 64 KiB per salt or info port. An empty HKDF salt or info is represented by
an explicit empty byte value. The scrypt and Argon2id salt must contain
8–1,024 bytes. EVP_BytesToKey requires exactly eight salt bytes.

PBKDF2 accepts 1–10,000,000 iterations, with the product of iterations and
digest blocks capped at 10,000,000. HKDF performs RFC 5869 extract followed
by expand. The 1,024-byte limit is below the 255-block HKDF limit for both
digest profiles.

The scrypt allocation estimate is `128 × r × (N + p + 1)` bytes, plus the
result and 4 KiB overhead. The maximum is 64 MiB. The
`4 × N × r × p` Salsa20/8 work count is capped at 16,777,216. The supported
individual ranges are N from 2 through 524,288 and r and p from 1 through
1,024. Their combined limits also apply.

Argon2id uses version 0x13. Memory may be set from 8 through 64,512 KiB, with
at least 8 KiB per lane. Passes range from 1 through 255 and lanes from 1
through 32. Memory times passes is capped at 16,777,216 KiB-pass units.
Cube wipes owned Argon2id working memory and its owned derived buffers.
Caller buffers and backend-managed scrypt scratch may retain copies.

EVP_BytesToKey repeats the selected digest over each chained block. Count
times the number of digest blocks is capped at 1,000,000. Derivation serves
legacy OpenSSL interoperability. See [OpenSSL legacy decryption](openssl-legacy.md).

```sh
cube cipher kdf pbkdf2 --profile sha256 --iterations 100000 --output-bytes 32 \
  --secret-file password.bin --salt-file salt.bin > derived.bin
```
