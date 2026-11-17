# Raw ElGamal and Cramer–Shoup

These operations encrypt one element of the prime-order subgroup in RFC7919 ffdhe2048. A message is exactly 256 big-endian bytes representing that element, including leading zeroes. They do not embed text, pad integers or encrypt arbitrary file bytes. Use a complete [IES suite](ies.md) when the input is an arbitrary byte message.

## Complete profiles and encodings

| Operation family | Required profile | Public key | Private key | Ciphertext |
| --- | --- | --- | --- | --- |
| `cipher.elgamal.encrypt` / `.decrypt` | `elgamal_ffdhe2048_raw_v1` | y: 256 bytes | x: 256 bytes | u||e: 512 bytes |
| `cipher.cramer_shoup.encrypt` / `.decrypt` | `cramer_shoup_ffdhe2048_sha256_raw_v1` | c||d||h: 768 bytes | x1||x2||y1||y2||z: 1,280 bytes | u1||u2||e||v: 1,024 bytes |

Each component is a fixed-width unsigned big-endian integer of 256 bytes. Private scalars must be in 1..q−1, where p is the RFC7919 ffdhe2048 prime and q=(p−1)/2. Public keys and u/u1/u2 must be nonidentity elements of the prime-order subgroup. The message, e and v may equal the identity 1. Zero, p−1, p, wrong widths and nonmembers are refused.

The group and generators are fixed. There are no arbitrary parameter APIs, weak compatibility groups, key generation or deterministic production coins. Callers must supply properly generated private keys. Rust private settings provide `public_key()` to derive the corresponding public encoding without returning private material.

## ElGamal behavior

Encryption samples a fresh uniform nonzero r modulo q, computes u=2^r and e=y^r·message modulo p, then returns u||e. Decryption returns e·u^(q−x). Raw ElGamal is malleable and unauthenticated: altering a ciphertext within the valid group can change the returned message without an error. Successful output does not establish integrity or sender identity.

## Cramer–Shoup behavior

The profile uses g1=2 and a pinned second generator g2. Its public derivation reads 256 bytes from SHAKE256 over ASCII `CubeArc Cramer-Shoup ffdhe2048 generator v1` followed by a four-byte big-endian counter, rejects candidates outside 2..p−2, squares modulo p, and rejects identity/g1. The pinned counter is zero; the SHA256 of the resulting 256 generator bytes is `8a5e17bbd80f6f71d37a50e1c445ce2af79a46814c2486caff48de70bcc072da`.

`label_hex` is an optional binary transcript label, defaulting to empty and limited to 256 decoded bytes. The hash is SHA256 over ASCII `CubeArc Cramer-Shoup ffdhe2048 SHA256 v1`, a zero byte, fixed-width u1||u2||e, then the label. Encryption returns u1||u2||e||v. Decryption validates the complete private verification equation in constant time before returning the group message. A changed group element or label is rejected without plaintext.

This selects an explicit raw profile of the [Cramer–Shoup construction](https://www.shoup.net/papers/cs.pdf); it is not a universal standardized wire format or an audit/certification claim. In particular, Bouncy Castle's default Cramer–Shoup digest input uses a different integer encoding. Matching only the algorithm name does not establish ciphertext compatibility. Ciphertext verification does not identify the sender.

## CLI and SDKs

Save the required profile and, for Cramer–Shoup, the matching label in a settings file:

```sh
cube cipher elgamal encrypt --parameters-file settings.json --key-file public.bin --file group.bin > ciphertext.bin
cube cipher cramer-shoup decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin > group.bin
```

Native Python exposes `elgamal_encrypt`, `elgamal_decrypt`, `cramer_shoup_encrypt` and `cramer_shoup_decrypt`, each taking `(data, key, parameters_json)` and returning bytes. JavaScript/TypeScript uses the operation ID with separate byte ports `input` and `key` and typed profile parameters. Every interface calls Core. Play requires explicit private retention consent.

## Limits and failures

One call processes one group message or ciphertext. Encoded key and settings ceilings are 1,280 bytes and 1,024 UTF-8 bytes; exact per-profile widths still apply. Encryption makes at most eight 256-byte random draws within a 2,048-byte entropy budget. All secret exponentiation uses the public 2,048-bit width in the maintained backend. Validation and private-key derivation remain bounded by the fixed group; native host limits may be smaller.

Invalid profile, recipient key or label encoding returns `invalid_parameter`. Invalid message/ciphertext encoding, subgroup or Cramer–Shoup verification returns `group encryption failed` without output. Oversized input returns `resource_limit`. The CLI writes only after the complete Core operation succeeds.

Owned private scalars and intermediates are wiped on drop; erasure of every backend, caller, interpreter and operating-system copy is not guaranteed. [Backend qualification](../../contributing/asymmetric-backends.md) records the independent vectors, entropy tests and demonstrated difference in tampering behavior.
