# ECIES and DHIES recipient encryption

Encrypt bytes to a recipient using `cipher.ies.encrypt`, or validate and decrypt them with `cipher.ies.decrypt`. Each explicit suite combines a fresh ephemeral agreement, KDF2-SHA256 streaming encryption and HMAC-SHA256. These profiles match Bouncy Castle 1.83 `IESEngine` stream mode. They provide ciphertext integrity and confidentiality without establishing the sender's identity.

## Complete suites

| Profile | Recipient public key and ephemeral V | Recipient private key | Shared Z |
| --- | --- | --- | --- |
| `ecies_p256_bc183_sha256_kdf2_hmac256` | 65 uncompressed SEC1 P-256 bytes | 32 big-endian scalar bytes in 1..n−1 | 32-byte shared x coordinate |
| `dhies_ffdhe2048_bc183_sha256_kdf2_hmac256` | 256 fixed-width big-endian RFC7919 ffdhe2048 bytes | 32 big-endian exponent bytes greater than one | 256 fixed-width big-endian bytes |

P-256 points must be canonical, on the curve and not infinity. DH public values must be in 2..p−2 and the prime-order subgroup. There are no arbitrary groups, static sender keys, IVs, curve substitutions or format guessing. Key bytes are a separate input; these suites do not accept PEM keys.

Ciphertext is exactly V||C||T. V uses the public encoding above, C has the plaintext length, and T is 32 bytes. Integers and coordinates retain leading zeroes. Every encryption obtains a fresh private ephemeral key from the system CSPRNG; there is no production deterministic-coin interface.

## KDF and authenticated context

`derivation_hex` and `encoding_hex` are optional hexadecimal byte strings, each defaulting to empty. The suite computes KDF2-SHA256 over V||Z, a four-byte big-endian counter starting at one, then the derivation bytes. The first 32 derived bytes form the MAC key; the remaining message-length stream is XORed with the plaintext.

T is HMAC-SHA256 over C||encoding||L2, where L2 is the encoding length in bits as an eight-byte big-endian integer. Decryption validates the full tag before deriving or returning plaintext. Both contexts must match exactly. Empty plaintext is supported and still produces V and an authentication tag.

The [BC1.83 implementation](https://github.com/bcgit/bc-java/blob/r1rv83/core/src/main/java/org/bouncycastle/crypto/engines/IESEngine.java) follows an IEEE1363a-style integrated encryption construction. These complete profiles select its ephemeral stream variant, KDF, hash, MAC, lengths and public encodings explicitly. They do not denote every scheme called ECIES or DHIES. [HPKE RFC9180](https://www.rfc-editor.org/rfc/rfc9180) uses distinct KEM/KDF/AEAD suites and labelled derivations; its ciphertexts are not compatible with these IES profiles. No independent audit or certification is claimed.

## CLI and SDKs

Save the suite and matching contexts in `settings.json`:

```json
{"profile":"ecies_p256_bc183_sha256_kdf2_hmac256","derivation_hex":"","encoding_hex":""}
```

Supply the raw public or private key separately:

```sh
cube cipher ies encrypt --parameters-file settings.json --key-file public.bin --file message.bin > ciphertext.bin
cube cipher ies decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin > message.bin
```

Native Python exposes `ies_encrypt(data, key, parameters_json)` and `ies_decrypt(data, key, parameters_json)`. JavaScript/TypeScript uses `cube.execute` with byte ports `input` and `key`, the operation ID and typed parameters. All interfaces reuse Core. Play requires explicit private retention consent.

## Limits and failures

Plaintext is at most 16 MiB; ciphertext adds 97 bytes for ECIES or 288 bytes for DHIES. Raw keys are at most 256 bytes, settings at most 2,048 UTF-8 bytes, and each context at most 256 decoded bytes. Ephemeral generation makes at most eight 32-byte draws within a 256-byte entropy budget. A host's SDK, transport and native worker limits may be smaller.

Invalid suite, recipient key or settings return `invalid_parameter`. Invalid ephemeral material, an incorrect context, a truncated or altered ciphertext and an incorrect tag return the same `IES operation failed` error without plaintext. Oversized input returns `resource_limit`. There is no algorithm fallback or partial recovery.

Owned private/shared/key-stream/plaintext buffers and supported SHA256/HMAC state are erased on drop; erasure of every caller, interpreter and operating-system copy is not guaranteed. See [backend qualification](../../contributing/asymmetric-backends.md) for independent interoperability evidence and [raw agreement](key-agreement.md) for the underlying key encodings.
