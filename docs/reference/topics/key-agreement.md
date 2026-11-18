# Raw key agreement

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

## Finite-field DH

`crypto.dh.agree` uses the five safe-prime groups in [RFC7919 Appendix A](https://www.rfc-editor.org/rfc/rfc7919#appendix-A), with generator 2 and prime-order subgroup q=(p−1)/2. It refuses arbitrary moduli, weak compatibility groups and implicit parameter generation.

| Profile | Public input and shared output | Private exponent |
| --- | --- | --- |
| `ffdhe2048_rfc7919_raw` | 256 bytes | 32 bytes |
| `ffdhe3072_rfc7919_raw` | 384 bytes | 40 bytes |
| `ffdhe4096_rfc7919_raw` | 512 bytes | 48 bytes |
| `ffdhe6144_rfc7919_raw` | 768 bytes | 56 bytes |
| `ffdhe8192_rfc7919_raw` | 1,024 bytes | 64 bytes |

All integers use fixed-width big-endian bytes, retaining leading zeroes. Private exponents must exceed one; callers must supply fresh uniformly random secret material at the exact profile width. These widths exceed RFC7919's short-exponent minima. A peer must be in 2..p−2 and satisfy peer^q mod p = 1 before any secret exponentiation. The constant-time backend processes the full public profile exponent width, including leading zeroes. There is no implicit KDF or authentication.

Use a separate binary key file, keeping JSON settings to `{"profile":"ffdhe2048_rfc7919_raw"}`:

```sh
cube cipher dh agree --parameters-file settings.json --key-file secret.bin --file peer.bin > shared.bin
```

Native Python exposes `dh_agree(peer_bytes, private_key_bytes, parameters_json)`. JavaScript/TypeScript uses `cube.execute` with separate byte ports `input` and `key` and the required `profile`. Rust also provides `Settings::public_key()` for the matching generator exponentiation. The executable operation returns only the raw shared secret.

JSON settings are at most 256 bytes; peer and private key transport are at most 1,024 and 64 bytes respectively. A call performs one public subgroup check and one secret exponentiation. Wrong lengths, range or subgroup yield no shared secret. Unknown groups or private exponents 0/1 fail with redacted settings errors. Play requires explicit private retention consent and retains its existing transport and worker limits.

Use [key derivation](key-derivation.md) to bind raw shared material to an
explicit context.
