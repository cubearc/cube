# AES key wrapping

Wrap binary key data deterministically, or verify the complete wrapping before
receiving any unwrapped key bytes. These operations implement fixed RFC profiles
without inventing a file format, deriving a password key or accepting a custom IV.

| Operation | Rust API in `ciphers::key_wrap` | Native Python |
| --- | --- | --- |
| `cipher.aes.key_wrap.wrap` | `wrap(data, settings)` | `aes_key_wrap(data, parameters_json)` |
| `cipher.aes.key_wrap.unwrap` | `unwrap(data, settings)` | `aes_key_unwrap(data, parameters_json)` |

JavaScript/TypeScript uses `cube.execute` with version 1, required private
`parameters`, an `input` byte port and an `output` byte port.

## Select the wrapping profile

Private settings require `profile` and `key_hex`. The key-encryption key (KEK)
contains exactly 16, 24 or 32 bytes, encoded as hexadecimal without separators
or a prefix. This selects AES-128, AES-192 or AES-256. Unknown fields, including
an IV or KDF choice, fail rather than changing the recognized construction.

| Profile | Key data | Wrapped result |
| --- | --- | --- |
| `aes_kw_rfc3394` | 16 to 65,536 bytes, multiple of 8 | Input length plus 8 bytes |
| `aes_kwp_rfc5649` | 1 to 65,536 bytes | Round up to 8 bytes, then add 8 bytes |

KW uses the fixed RFC3394 eight-byte initial value. KWP uses RFC5649's alternative
initial value, message-length indicator and defined zero padding. A one-byte KWP
input produces 16 bytes; aligned input does not add another padding block.
Wrapped input/output is at most 65,544 bytes. Private settings are at most 1024
UTF-8 bytes. Processing uses at most 49,152 AES wrapping steps with fixed backend
state and bounded input/output buffers. Hosts may impose lower transport budgets.

## Verify before key release

Unwrap verifies the IV/AIV, encoded length and padding before returning any key
material. A wrong KEK of valid length, malformed wrapping or failed integrity
check yields the same `invalid_input` failure, with no partial result. Invalid
settings and resource limits have distinct categories. KW's integrity field is
64 bits; key wrapping does not replace a general-purpose authenticated message
envelope or key-management system.

Owned decoded KEKs and temporary key-data buffers are wiped on drop; caller,
parser, interpreter and OS copies remain their owners' responsibility. Both byte
ports and the KEK are private. Automatic capture and linear recipe adapters are
disabled; native Play requires explicit private-retention consent.

## Wrap a binary key

For the public RFC3394 example, store these settings in `settings.json`:

```json
{"profile":"aes_kw_rfc3394","key_hex":"000102030405060708090a0b0c0d0e0f"}
```

Input bytes `00112233445566778899aabbccddeeff` produce
`1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5` as raw bytes:

```sh
cube cipher aes key-wrap wrap --parameters-file settings.json --file key.bin > wrapped.bin
cube cipher aes key-wrap unwrap --parameters-file settings.json --file wrapped.bin > recovered-key.bin
```

Omit `--file` to read binary stdin. The CLI accepts private settings through a
file and writes unwrapped bytes only after verification. Fixed values above are
test data, not generated keys.

The implementation pins the maintained zeroizing RustCrypto `aes-kw`0.3.1
backend over the shared AES primitive. Independent evidence comes from
[RFC3394](https://www.rfc-editor.org/rfc/rfc3394),
[RFC5649](https://www.rfc-editor.org/rfc/rfc5649) and
[NIST SP800-38F CAVP fixtures](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/cavp-testing-block-cipher-modes).
Known successful and adversarial source cases retain file hashes and case IDs.
These informal vector checks do not imply CAVP/FIPS certification.
