# Raw AES blocks

Encrypt or decrypt whole AES blocks for low-level interoperability and known-answer
tests. These operations apply FIPS 197-upd1 AES independently to each block.
They do not provide a secure message-encryption scheme or authenticate data.

## Operations and keys

| Operation | Rust function | Python function |
| --- | --- | --- |
| `cipher.aes.encrypt_blocks` | `ciphers::aes_blocks::encrypt` | `aes_encrypt_blocks` |
| `cipher.aes.decrypt_blocks` | `ciphers::aes_blocks::decrypt` | `aes_decrypt_blocks` |

Both functions take exact input bytes followed by key bytes and return exact
bytes. A 16-, 24- or 32-byte key selects AES-128, AES-192 or AES-256 respectively.
Every profile uses a 16-byte block. There is no default key, passphrase conversion,
padding, IV, nonce, counter, KDF, tag or serialized encryption format. Generalized
Rijndael block sizes are unsupported.

Repeated equal blocks produce repeated equal output. A wrong valid-length key
still produces bytes on decryption: success is not evidence of key correctness,
integrity or authenticity. Empty input returns empty bytes with a valid key.

## Inputs and results

The version 1 contracts name their byte input `input` and byte result `output`.
Their JSON byte representation is `{"base64":"..."}` with canonical Base64;
`key_hex` is a required parameter containing exactly 32, 48 or 64 hexadecimal
digits, without a prefix or separators. Hexadecimal letter case is accepted.
Unknown and duplicate parameter fields are rejected. Rust and Python accept raw
key bytes instead of this JSON representation.

The public FIPS test input `00112233445566778899aabbccddeeff` under the public key
`000102030405060708090a0b0c0d0e0f` encrypts to
`69c4e0d86a7b0430d8cdb78070b4c55a`. Decrypting with that same key restores the exact
input bytes. No UTF-8 decoding or newline conversion occurs.

## Limits and errors

| Resource or rule | Limit or result |
| --- | --- |
| Input and output | At most 16 MiB each in Core, CLI and Python |
| Block alignment | Input length must be a multiple of 16 bytes |
| Work | At most 1,048,576 independent blocks; fixed AES rounds per key profile |
| Parameter JSON | At most 1024 UTF-8 bytes |
| Invalid key or parameter transport | `invalid_parameter`, without secret contents |
| Partial block | `invalid_input`; no implicit padding |
| Input or parameter budget exhausted | `resource_limit` before output allocation |

Output allocation is the input length plus fixed backend state. Play applies its
advertised JSON wire/input/output limits, which can be lower than Core's byte cap.
See the [canonical contracts](../operations.md) for the exact named schemas and
[CLI byte workflow](../../guides/binary.md#transform-raw-aes-blocks) for file input.

## Secret handling and backend

CLI keys come only from a raw key file; payload comes from a file or stdin.
Output is binary stdout. Keys cannot be passed as CLI text/hex arguments, used
in pipeline/recipe steps or exported through `--lab-capture`.

The Play editor treats the key and literal input as private run-time values.
They are omitted from saved configurations and input provenance; key parameters
are redacted in result provenance. Results are transient by default. An explicit
private-output retention choice stores the output locally, never the key.
There are no automatic captures or uploads.

The adapter pins RustCrypto `aes` 0.9.3 with `zeroize` enabled. It uses runtime CPU
dispatch or the upstream fixsliced software backend. CI also forces software and
runs the same independent vectors. Expanded keys and owned decoded key buffers
are wiped on drop; borrowed inputs/keys, returned bytes and parser, interpreter,
compiler or OS copies cannot be guaranteed erased. No certification or audit of
this Cube adapter is claimed.

Sources: [FIPS 197-upd1](https://csrc.nist.gov/pubs/fips/197/final),
[NIST validation vectors](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/block-ciphers)
and [the pinned RustCrypto backend](https://docs.rs/aes/0.9.3/aes/).
