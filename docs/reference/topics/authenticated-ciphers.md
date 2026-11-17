# Authenticated byte encryption

Encrypt binary data with associated data and a detached authentication tag, or
verify all of them before receiving any plaintext. The operations return exact
bytes without choosing a file format, generating keys or supplying a nonce.

| Operation | Inputs | Outputs |
| --- | --- | --- |
| `cipher.aead.encrypt` | `input`, `aad` bytes | `output` ciphertext and `tag` bytes |
| `cipher.aead.decrypt` | `input` ciphertext, `aad`, `tag` bytes | Authenticated `output` bytes |
| `cipher.aes.siv.encrypt` | `input` bytes, `aad_components` | `output` ciphertext and `tag` bytes |
| `cipher.aes.siv.decrypt` | `input` ciphertext, `aad_components`, `tag` | Authenticated `output` bytes |

Use version 1 with the JavaScript/TypeScript `cube.execute` API. Rust functions
live in `ciphers::modern_aead`. Native Python exposes `aead_encrypt`,
`aead_decrypt`, `aes_siv_encrypt` and `aes_siv_decrypt`.

## Choose an exact profile

For nonce-based encryption, the private settings require `profile`, `key_hex`
and `nonce_hex`. Hexadecimal fields contain exact bytes without separators or a
prefix. Associated data is a separate required byte input; empty AAD must still
be supplied. Tags have their full profile length; truncation is not accepted.

| Profile | Key bytes | Nonce bytes | Tag bytes | Message ceiling |
| --- | ---: | ---: | ---: | ---: |
| `aes128_gcm` | 16 | 12 | 16 | 16 MiB |
| `aes256_gcm` | 32 | 12 | 16 | 16 MiB |
| `chacha20poly1305_ietf` | 32 | 12 | 16 | 16 MiB |
| `xchacha20poly1305` | 32 | 24 | 16 | 16 MiB |
| `aes128_gcm_siv` | 16 | 12 | 16 | 16 MiB |
| `aes256_gcm_siv` | 32 | 12 | 16 | 16 MiB |
| `ascon_aead128_nist_sp800_232` | 16 | 16 | 16 | 16 MiB |
| `aes128_ccm_nonce13_tag8` | 16 | 13 | 8 | 65,535 bytes |
| `aes128_ccm_nonce12_tag16` | 16 | 12 | 16 | 16,777,215 bytes |
| `aes256_ccm_nonce12_tag16` | 32 | 12 | 16 | 16,777,215 bytes |
| `aes128_eax_nonce16_tag16` | 16 | 16 | 16 | 16 MiB |
| `aes256_eax_nonce16_tag16` | 32 | 16 | 16 | 16 MiB |
| `aes128_ocb3_nonce12_tag16` | 16 | 12 | 16 | 16 MiB |
| `aes256_ocb3_nonce12_tag16` | 32 | 12 | 16 | 16 MiB |
| `aegis128l_rfc10032_tag128` | 16 | 16 | 16 | 16 MiB |
| `aegis128l_rfc10032_tag256` | 16 | 16 | 32 | 16 MiB |
| `aegis256_rfc10032_tag128` | 32 | 32 | 16 | 16 MiB |
| `aegis256_rfc10032_tag256` | 32 | 32 | 32 | 16 MiB |
| `deoxys_ii_128_128` | 16 | 15 | 16 | 16 MiB |
| `deoxys_ii_256_128` | 32 | 15 | 16 | 16 MiB |

The Ascon profile implements final NIST SP800-232 byte order. Prestandard
Ascon-128/128a names are rejected. OCB is OCB3 from RFC7253. The AEGIS profiles
follow informational RFC10032, published in September 2026. Deoxys-II key sizes
are explicit; these names do not select Deoxys-I.

XChaCha20-Poly1305 derives a subkey with HChaCha20 from the first 16 nonce
bytes, then uses the IETF construction with four zero bytes followed by the
remaining eight nonce bytes. This is separate from ordinary ChaCha20-Poly1305.

Every call also limits the **combined message and AAD** to 16 MiB. CCM's nonce
length determines its smaller message bound. Private settings are at most 2048
UTF-8 bytes. Encryption preserves message length and returns the tag separately.
Hosts, SDK transports and private Play graphs may impose lower budgets.

## Manage authentication and nonce use

Supply a unique nonce for each key and encryption call, and follow the selected
algorithm's per-key usage limits. Cube is stateless and does not track key/nonce
reuse across calls. GCM's message and counter ceilings are tighter here because
of the 16 MiB body bound and fixed 96-bit nonce. GCM-SIV's misuse resistance does
not justify unlimited reuse; deterministic SIV has different equality leakage.

Decryption checks the complete tag before returning any plaintext. A modified
key of valid length, nonce, ciphertext, AAD or tag fails authentication uniformly.
Wrong tag lengths also fail. Invalid settings and resource limits remain distinct
errors. Private results are excluded from automatic capture and linear recipes;
Play execution requires explicit private-retention consent.

Owned decoded keys and tentative plaintext buffers are wiped on drop. Caller,
parser, interpreter, operating-system and some backend state copies may remain.
Backend audit and timing qualifications differ; see the
[backend reference](../../contributing/authenticated-backends.md). No profile
implies FIPS validation or a security audit of this integration.

## Preserve SIV component boundaries

Nonce-free SIV settings require only `profile` and `key_hex`:

| Profile | Total key bytes | Detached tag bytes |
| --- | ---: | ---: |
| `aes128_siv_cmac_nonce_free` | 32 | 16 |
| `aes256_siv_cmac_nonce_free` | 64 | 16 |

The total key contains the two RFC5297 component keys. A `nonce_hex` field is
forbidden. `aad_components` is an ordered JSON array of byte arrays for versioned
execution; native Python accepts `list[bytes]` and Rust accepts `&[Vec<u8>]`.
At most 126 components are accepted, with the same 16 MiB combined byte bound.

Component count, order and boundaries are authenticated. `[]` differs from
`[[]]`, and `[[1, 2], [3]]` differs from `[[1, 2, 3]]`. Encryption is deterministic:
identical plaintext, key and components produce identical ciphertext and tag.
The detached tag is the synthetic IV; combined RFC5297 serialization places it
before the ciphertext, but these operations return separate byte outputs.

## Verify a binary message

For a NIST AES-GCM example with empty plaintext and AAD, put these public test
settings in a private `settings.json` file:

```json
{"profile":"aes128_gcm","key_hex":"00000000000000000000000000000000","nonce_hex":"000000000000000000000000"}
```

Create empty `input.bin` and `context.bin`, then run:

```sh
cube cipher aead encrypt --parameters-file settings.json --aad-file context.bin --file input.bin
```

The result is JSON with empty `ciphertext_hex` and
`tag_hex` equal to `58e2fccefa7e3061367f1d57a4e7455a`. These are fixed test values;
use caller-managed keys and fresh nonces for actual messages.

To verify, supply raw ciphertext, AAD and tag files. Plaintext is written only
after successful authentication:

```sh
cube cipher aead decrypt --parameters-file settings.json --aad-file context.bin --tag-file tag.bin --file encrypted.bin
```

SIV uses `cube cipher aead siv encrypt` or `decrypt` and replaces `--aad-file`
with `--aad-components-file components.json`. Its component document is a JSON
array of byte arrays and has an additional 16 MiB CLI transport bound.
