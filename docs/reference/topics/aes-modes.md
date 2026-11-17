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
For ordinary and CTS modes, the key contains 16, 24 or 32 bytes; storage modes use
two combined keys as described below. Keys are hexadecimal without
separators or a prefix. Unknown fields and unsupported variants fail.

| Mode | Additional required fields | Byte behavior |
| --- | --- | --- |
| `ecb` | `padding`: `none` or `pkcs7` | Independent blocks; equal blocks reveal repetition |
| `cbc` | `iv_hex`, `padding`: `none` or `pkcs7` | Chained 16-byte blocks |
| `cfb128` | `iv_hex` | Full-block feedback with a possible partial final block |
| `ofb` | `iv_hex` | Full-block output feedback, preserving input length |
| `ctr128_be` | `iv_hex` | Increment the entire 128-bit big-endian counter |

`iv_hex` contains exactly16 bytes. ECB forbids an IV; storage modes require
`tweak_hex` instead. Only ordinary ECB/CBC accept a padding field. CFB1/CFB8, partial-width counters and inferred padding are not
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

## Ciphertext stealing and storage units

The same operations accept these exact modes, preserving input length:

| Mode | Key bytes | Initial state | Length |
| --- | ---: | --- | --- |
| `cbc_cs1`, `cbc_cs2`, `cbc_cs3` | 16, 24, 32 | `iv_hex`:16 bytes | 16 bytes to16 MiB |
| `xts_aes128` | 32 total | `tweak_hex`:16 bytes | 16 bytes to16 MiB |
| `xts_aes256` | 64 total | `tweak_hex`:16 bytes | 16 bytes to16 MiB |
| `xex_aes128_ieee1619_full_blocks` | 32 total | `tweak_hex`:16 bytes | Whole blocks,16 bytes to16 MiB |
| `xex_aes256_ieee1619_full_blocks` | 64 total | `tweak_hex`:16 bytes | Whole blocks,16 bytes to16 MiB |

CTS modes forbid a `padding` field. CS1 places the truncated penultimate segment
before the final full ciphertext block; CS2/CS3 put it after that block. For aligned
messages CS1/CS2 are CBC; CS3 swaps the final two blocks when at least two blocks
are present. At the exact 16-byte minimum all variants equal single-block CBC.

Storage keys concatenate two independent AES keys of the indicated size; equal
halves are rejected. `tweak_hex` is the exact 16-byte **input** to the second AES
key, before tweak encryption. To represent an integer data-unit number, encode it
as a 128-bit little-endian byte sequence. Tweak advancement uses the IEEE1619
little-endian field convention. No implicit sector split, unit-number increment
or disk access occurs; each call transforms one complete caller-supplied unit.

XTS uses the IEEE1619-2007/SP800-38E byte-oriented construction, including
ciphertext stealing for a partial final block. Its ceiling is 2^20 AES blocks per
data unit. The explicitly named XEX modes use that construction's full-block
component: `AES_K1(P xor T) xor T`, with initial `T=AES_K2(tweak)` and field
advancement between blocks. They reject partial blocks rather than selecting a
stealing convention. These are precise two-key IEEE profiles, not an unspecified
single-key XEX interface. Tweak reuse and malleability remain caller concerns;
none of these profiles authenticates data.

Reference sources are the [NIST CBC addendum](https://csrc.nist.gov/pubs/sp/800/38/a/sup/final),
[SP800-38E](https://csrc.nist.gov/pubs/sp/800/38/e/final) and the
[NIST CAVP vectors](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/cavp-testing-block-cipher-modes).
The profile pins the 2010 final recommendation; the 2026 revision draft is not
represented as a final publication. RustCrypto CTS0.7.0 and `xts-mode`0.6.0
provide the maintained mode implementations over the shared AES backend.
