# AES byte modes

Encrypt or decrypt binary data with an explicit SP800-38A mode. These low-level
profiles provide confidentiality without authentication. A successful decryption,
including valid PKCS#7 padding, does not establish authenticity or a correct key.

`cipher.aes.mode.encrypt` and `cipher.aes.mode.decrypt` accept a named `input`
byte port and return `output` bytes. The Rust API is `ciphers::aes_modes`;
native Python exposes `aes_mode_encrypt(data, parameters_json)` and
`aes_mode_decrypt(data, parameters_json)`. JavaScript and TypeScript use
`cube.execute` with the same IDs, version 1, byte ports and parameter fields.

## Choose a mode

Every call requires a private JSON document containing `mode` and `key_hex`.
The key must contain exactly 16, 24 or 32 bytes, expressed as hexadecimal without
separators or a prefix. Unknown fields and unsupported variants fail.

| Mode | Additional required fields | Byte behavior |
| --- | --- | --- |
| `ecb` | `padding`: `none` or `pkcs7` | Independent blocks; equal blocks reveal repetition |
| `cbc` | `iv_hex`, `padding`: `none` or `pkcs7` | Chained 16-byte blocks |
| `cfb128` | `iv_hex` | Full-block feedback with a possible partial final block |
| `ofb` | `iv_hex` | Full-block output feedback, preserving input length |
| `ctr128_be` | `iv_hex` | Increment the entire 128-bit big-endian counter |

`iv_hex` contains exactly 16 bytes. ECB forbids an IV; feedback/counter profiles
forbid padding. CFB1/CFB8, partial-width counters and inferred padding are not
accepted. CBC and CFB need unpredictable fresh IVs; OFB/CTR need unique initial
states for each key. The caller supplies and manages these values.

## Padding and bounds

With `none`, ECB/CBC accept only whole blocks, including an empty identity.
PKCS#7 encryption always adds padding, including a full block for aligned or
empty input. Decryption requires nonempty whole blocks and validates every
padding byte before returning anything. Malformed ciphertext and padding share
one error. These checks do not make a padding oracle safe for an online service.

Core accepts bodies up to 16 MiB and private settings up to 2048 UTF-8 bytes.
PKCS#7 ciphertext may contain one additional block; decrypted output remains
at most 16 MiB. Feedback/counter input and output lengths are identical.
CTR permits the last counter value once and rejects wrap before allocation.
Host transport budgets can be lower; private Play execution requires explicit
retention consent. No linear recipe adapter or automatic capture is enabled.

## Transform a file

Store the caller-supplied key and IV in a private `settings.json` file:

```json
{"mode":"cbc","padding":"none","key_hex":"2b7e151628aed2a6abf7158809cf4f3c","iv_hex":"000102030405060708090a0b0c0d0e0f"}
```

For input bytes `6bc1bee22e409f96e93d7e117393172a`, this NIST example produces
`7649abac8119b246cee98e9b12e9197d` as raw bytes:

```sh
cube cipher aes mode encrypt --parameters-file settings.json --file input.bin > encrypted.bin
cube cipher aes mode decrypt --parameters-file settings.json --file encrypted.bin > recovered.bin
```

The CLI reads binary stdin when `--file` is omitted. It does not accept secret
key text on the command line. Owned decoded keys and temporary plaintext buffers
are wiped on drop; caller, parser, interpreter and operating-system copies remain
under their owners' control.

The implementation uses RustCrypto AES and its maintained CBC, ECB, CFB, OFB and
CTR adapters. The independent vectors are from
[NIST SP800-38A](https://csrc.nist.gov/pubs/sp/800/38/a/final), Appendix F.
