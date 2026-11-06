# Raw block reference profiles

Encrypt or decrypt exact blocks for legacy interoperability and known-answer
tests. Select an explicit profile and supply the exact key bytes. These
unaudited backends do not promise constant-time execution or provide secure
message encryption. Raw blocks have no authentication: a wrong valid key still
produces bytes on decryption.

## Operations and profiles

`cipher.block.encrypt_blocks` and `cipher.block.decrypt_blocks` return exact
bytes under the selected full-round profile. Every profile below uses eight-byte
blocks. There is no default profile or key.

| Profile | Key bytes | Definition |
| --- | --- | --- |
| `des_fips46_v1` | 8 | DES, 16 rounds; parity bits ignored |
| `tdes_ede2_v1` | 16 | Two-key Triple DES, encrypt K1 / decrypt K2 / encrypt K1 |
| `tdes_ede3_v1` | 24 | Three-key Triple DES, encrypt K1 / decrypt K2 / encrypt K3 |
| `blowfish_be_v1` | 4..56 | Blowfish, 16 rounds, big-endian words |
| `idea_v1` | 16 | IDEA, eight rounds and output transformation, big-endian words |
| `rc2_rfc2268_v1` | 1..128 | RFC 2268 RC2; little-endian words, 16 mixing rounds |

RC2 requires `effective_key_bits` in 1..1024, independently of the key length.
Other profiles reject that setting. No effective-bit count is inferred from
the key. DES-family parity is not validated or repaired. Weak and degenerate keys
are accepted for compatibility with historical data and test vectors.

Equal input blocks produce equal output blocks. Input length must be a multiple
of the profile's block width; empty input returns empty bytes after settings
validation. No mode, padding, IV, nonce, counter, KDF, RNG, tag or serialized
message format is added. See [raw AES blocks](aes-blocks.md) for the separate
FIPS 197 interface.

## Inputs and results

Both version 1 operations name their byte input `input` and byte result `output`.
Canonical JSON uses `{"base64":"..."}` for each byte port and requires
`profile` and `key_hex` parameters. Keys use exact hexadecimal byte pairs,
without prefixes or separators; upper- and lowercase hex letters are accepted.
RC2 additionally requires `effective_key_bits`. Unknown or duplicate fields and
null effective-bit values are rejected.

The CLI reads a raw key file and a payload file or stdin. It writes binary stdout:

```sh
cube cipher block encrypt --profile des_fips46_v1 \
  --key-file key.bin --file blocks.bin > encrypted.bin
cube cipher block decrypt --profile des_fips46_v1 \
  --key-file key.bin --file encrypted.bin > restored.bin
```

With the public eight-byte key `0113b970fd34f2ce` and input
`059b5e0851cf143a`, encryption produces `86a560f10ec6d85b`. The hexadecimal
strings describe bytes; the files contain the corresponding raw bytes.

Rust calls `ciphers::block_profiles::Settings::new` or `Settings::parse`, then
`encrypt(data, &settings)` or `decrypt(data, &settings)`. Native Python calls
`block_encrypt_blocks(data, key, profile, *, effective_key_bits=None)` or
`block_decrypt_blocks` with the same arguments. JavaScript and TypeScript use
the operation IDs through `cube.execute`, with `Uint8Array` byte ports and the
generated profile-specific parameter types. All interfaces execute Core.

## Limits and privacy

| Resource or failure | Rule |
| --- | --- |
| Input and output | At most 1 MiB each; output has the input length |
| Work | At most 131,072 independent eight-byte blocks plus fixed key expansion |
| Private parameter JSON | At most 4096 UTF-8 bytes |
| Invalid profile, key or settings | `invalid_parameter`; no secret excerpts |
| Partial block | `invalid_input`; no output or padding |
| Exhausted byte/parameter budget | `resource_limit` before output allocation |

Host transport budgets apply separately and can be lower. See
[resource limits](../../concepts/resource-limits.md) and the
[canonical contracts](../operations.md) for the exact named schemas.

CLI keys come only from a raw file. Keys cannot be passed as text/hex arguments,
captured with `--lab-capture` or used in pipeline/recipe steps. Play requires its
explicit private-output retention policy; key parameters and literal inputs stay
out of saved configuration and provenance. Byte outputs use the existing viewer.
No network access or automatic capture is introduced.

Owned decoded keys and backend schedules are wiped on drop. Temporary backend
key-expansion arrays, caller buffers, parser/interpreter/compiler copies and OS
memory are not guaranteed erased. Returned bytes belong to the caller. Small
block sizes, obsolete algorithms and weak keys retain their original weaknesses.

Sources: [RustCrypto qualification](../../contributing/block-backends.md),
[RC2 specification and vectors](https://www.rfc-editor.org/rfc/rfc2268.txt),
[Blowfish author vectors](https://www.schneier.com/wp-content/uploads/2015/12/vectors-2.txt).
