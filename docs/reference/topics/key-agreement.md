# Elliptic-curve key agreement

Compute a raw shared secret from your private key and a peer's public key with
`crypto.ecdh.agree`. Agreement does not authenticate the peer or encrypt a message.
Apply an explicit KDF that binds the protocol context and both participants before
using the result as key material.

## Key encodings

| Profile | Private key | Peer public key | Output |
| --- | --- | --- | --- |
| `x25519_rfc7748` | 32 bytes, RFC7748 clamping | 32 raw little-endian bytes | 32 raw bytes |
| `p256_sec1_raw` | 32-byte big-endian scalar in 1..n−1 | 33-byte compressed or 65-byte uncompressed SEC1 point | 32-byte big-endian shared x coordinate |

Private settings require `profile` and `private_key_hex` (exactly 64 hexadecimal
digits). There is no default curve, RNG, KDF or key lookup. X25519 accepts the
masked and reduced encodings required by RFC7748 and rejects an all-zero result.
P-256 validates the scalar and point, rejecting infinity, hybrid encodings and
noncanonical coordinates. A secp256k1 point is not a P-256 key.

## CLI and SDKs

Save the private settings in a local file, and supply the peer's raw public key:

```sh
cube cipher ecdh agree --parameters-file private.json --file peer.bin > shared.bin
```

The CLI writes raw bytes after successful validation. Native Python exposes
`ecdh_agree(peer_bytes, parameters_json)`; JavaScript/TypeScript uses
`cube.execute` with the operation ID, a byte `input` and typed parameters. These
interfaces call Core. Shared secrets require private handling and explicit
retention consent in Play.

## Limits and failures

Private settings are limited to 1,024 UTF-8 bytes. The peer key is at most 65
bytes and must match the selected encoding. One call performs one agreement and
returns exactly 32 bytes. Invalid peer material yields `key agreement failed`
without output; invalid private settings are redacted. Oversized inputs or
settings return `resource_limit`.

Owned private buffers and backend key containers are wiped on drop. Erasure of
caller, parser, interpreter and operating-system copies is not guaranteed. See
[backend qualification](../../contributing/asymmetric-backends.md) for evidence.
