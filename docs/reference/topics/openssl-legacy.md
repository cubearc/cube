# Decrypt explicit OpenSSL legacy files

`cipher.openssl.decrypt` version1 accepts binary `Salted__` files under an exact
caller-selected profile. It returns bytes only after complete PKCS7 validation.
**The output is unauthenticated.** A wrong password or modified ciphertext can
pass padding and return plausible bytes. Successful decryption is not a password,
key, integrity or authenticity check.

Rust exposes `cube_core::ciphers::openssl::{Settings, decrypt}`; native Python
exposes `openssl_decrypt(data, parameters_json)`. JavaScript/TypeScript uses
`cube.execute` with a byte `input` and byte `output`. All adapters reuse Core.
There is no legacy encryption operation or automatic cipher/KDF detection.

## Select the exact compatibility profile

| Profile | Derivation | Salt |
| --- | --- | --- |
| `aes128_cbc_pkcs7_evp_md5` | EVP_BytesToKey MD5, count1 |8 bytes |
| `aes256_cbc_pkcs7_evp_md5` | EVP_BytesToKey MD5, count1 |8 bytes |
| `aes128_cbc_pkcs7_evp_sha256` | EVP_BytesToKey SHA256, count1 |8 bytes |
| `aes256_cbc_pkcs7_evp_sha256` | EVP_BytesToKey SHA256, count1 |8 bytes |
| `aes128_cbc_pkcs7_pbkdf2_sha256_salt8` | PBKDF2-HMAC-SHA256 |8 bytes |
| `aes256_cbc_pkcs7_pbkdf2_sha256_salt8` | PBKDF2-HMAC-SHA256 |8 bytes |
| `aes128_cbc_pkcs7_pbkdf2_sha256_salt16` | PBKDF2-HMAC-SHA256 |16 bytes |
| `aes256_cbc_pkcs7_pbkdf2_sha256_salt16` | PBKDF2-HMAC-SHA256 |16 bytes |

Required settings are `profile` and `password_hex`. The password is exact binary
bytes in even-length hexadecimal, at most1024 bytes, with no Unicode or newline
normalization. The empty password is an explicit empty string. PBKDF2 additionally
requires `iterations` in1..100000; EVP profiles forbid that field. There are no
implicit digest, salt-size or iteration defaults. Unknown fields are rejected.
Settings JSON is capped at4096 UTF-8 bytes.

The file is the eight-byte ASCII marker, the profile's salt bytes, then nonempty
whole AES-CBC blocks using PKCS7. Both input and output are capped at16MiB;
input includes header, salt and padding overhead. At most200000 HMAC invocations
are allowed. Incorrect header/length/padding uses one generic legacy-decryption
failure and releases no partial output. Invalid settings and resource limits
remain distinct. KDF and CBC temporary keys/plaintext are owned by Core; owned
secret buffers are wiped, without claiming erasure of caller/parser/OS copies.
Private Play execution requires explicit private-retention consent; automatic
capture and linear recipe adapters are disabled.

## Use a private settings file

For an existing AES256-CBC file produced with explicit PBKDF2/SHA256,10000
iterations and an eight-byte salt, create `settings.json` privately:

```json
{"profile":"aes256_cbc_pkcs7_pbkdf2_sha256_salt8","password_hex":"74657374","iterations":10000}
```

`74657374` is only the synthetic example password `test`.

```sh
cube cipher openssl decrypt --parameters-file settings.json --file encrypted.bin > plaintext.bin
```

```python
import cube
settings = '{"profile":"aes256_cbc_pkcs7_pbkdf2_sha256_salt8","password_hex":"74657374","iterations":10000}'
plaintext = cube.openssl_decrypt(encrypted_bytes, settings)
```

Keep actual passwords out of command-line arguments, logs, recipes and sessions.
The caller must independently verify recovered content. The CLI validates before
writing bytes to stdout; shell redirection can still create an empty destination
before the program runs.

## Compatibility evidence

As checked on2026-11-17, [OpenSSL enc documentation](https://docs.openssl.org/3.5/man1/openssl-enc/)
records the MD5-to-SHA256 default change in1.1.0 and the `-saltlen` option added
in3.2. Salt length must match decryption; it cannot be inferred from `Salted__`.
OpenSSL3 changed `-S` to omit an embedded salt header when encrypting. Such raw
ciphertext is outside this binary-header profile. Base64, unsalted files,
non-CBC modes, unpadded CBC and implicit fallback are not accepted by these IDs.

Core composes the documented [EVP_BytesToKey recurrence](https://docs.openssl.org/3.5/man3/EVP_BytesToKey/)
from maintained MD5/SHA256 primitives and uses RustCrypto PBKDF2 0.13.0/HMAC0.13.0
for the explicit PBKDF2 profiles. It reuses the existing Core AES-CBC adapter.
These are permissively licensed Rust dependencies; notices are retained.

The checked corpus contains48 complete files generated and decrypted by
OpenSSL3.6.3 and LibreSSL3.3.6, including empty, short and aligned messages.
A separate independently discovered wrong-password/valid-padding witness is
verified with real OpenSSL and tests the absence of authentication. Core, CLI,
installed native Python, actual Wasm and private native Play exercise the corpus.
No OpenSSL library is linked into Cube, and no FIPS/CAVP certification is claimed.

For artifacts that provide the salt and lengths separately, see the
[explicit EVP_BytesToKey profile](key-derivation.md#profiles-and-limits).
