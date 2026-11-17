# SM2 encryption

Use `cipher.sm2.encrypt` or `cipher.sm2.decrypt` with the standard SM2 curve and SM3. The selected profile fixes the complete ciphertext representation; it never guesses a curve, digest or component order. Decryption validates ciphertext integrity before returning recovered bytes. It does not establish the sender's identity.

## Profiles and key encodings

| Profile | Ciphertext representation |
| --- | --- |
| `sm2_sm3_c1c3c2` | Uncompressed SEC1 C1 (65 bytes), SM3 C3 (32 bytes), then C2 |
| `sm2_sm3_c1c2c3` | Uncompressed SEC1 C1 (65 bytes), C2, then SM3 C3 (32 bytes) |
| `sm2_sm3_der` | Canonical ASN.1 DER SEQUENCE of unsigned INTEGER x, INTEGER y, OCTET STRING C3, OCTET STRING C2 |

Encryption requires exactly 65 uncompressed SEC1 public bytes on SM2. Decryption requires exactly 32 big-endian private scalar bytes in 1..n−1. Both operations take the key as a separate byte input. P-256, secp256k1, infinity, invalid coordinates, compressed or hybrid encodings and key-format guessing are refused. DER ciphertext is separate from key encoding; these profiles do not accept PEM keys.

The algorithm uses the SM2 prime curve, SM3 counter KDF over x2||y2 with counters starting at one, and C3=SM3(x2||message||y2). Encryption obtains fresh system randomness and retries an all-zero KDF within its entropy budget. Decryption rejects an all-zero KDF and compares the complete C3 in constant time. Empty plaintexts and ciphertexts with an empty C2 are refused.

SM2 public-key encryption is specified in GB/T 32918.4-2016. The [archived IETF draft](https://datatracker.ietf.org/doc/html/draft-shen-sm2-ecdsa-02#section-7) describes the algorithm but is not an IETF standard. These profiles select their representations explicitly, including the historical C1C2C3 order. RFC8998 specifies a TLS profile and does not define this encryption interface. No national/ISO certification or independent audit is claimed.

## CLI and SDKs

Save `{"profile":"sm2_sm3_c1c3c2"}` as `settings.json` and supply the raw key separately:

```sh
cube cipher sm2 encrypt --parameters-file settings.json --key-file public.bin --file message.bin > ciphertext.bin
cube cipher sm2 decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin > message.bin
```

The CLI writes bytes only after Core succeeds. Native Python exposes `sm2_encrypt(data, key, parameters_json)` and `sm2_decrypt(data, key, parameters_json)`. JavaScript/TypeScript uses `cube.execute` with separate byte ports `input` and `key` and the required `profile`. Every interface reuses Core; Play requires explicit private retention consent.

## Limits and failures

The nonempty plaintext body is at most 16 MiB. Ciphertext is at most 16 MiB plus 128 bytes, covering the three representations. Keys and UTF-8 JSON settings are bounded by 65 and 256 bytes. Encryption consumes at most 256 bytes of system entropy, allowing at most eight 32-byte scalar draws; exhaustion fails without ciphertext. Host SDK and Play transport limits may impose smaller limits.

Invalid keys or settings return `invalid_parameter`. Malformed ciphertext, invalid C1, an incorrect component order, a wrong key and C3 failure return the same `SM2 operation failed` error with no plaintext. Oversized input returns `resource_limit`. DER rejects noncanonical integers, trailing data and other ASN.1 layouts.

Owned plaintext and shared-coordinate buffers are wiped on rejection/drop, and SM3 state erasure is enabled. Erasure of every backend, caller, interpreter and operating-system copy is not guaranteed. See [backend qualification](../../contributing/asymmetric-backends.md) for dependency and interoperability evidence.
