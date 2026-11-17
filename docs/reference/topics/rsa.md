# RSA encryption profiles

Encrypt a short message with an RSA public key, or recover bytes using the matching
private key. Keys are separate binary inputs: SPKI for public keys and unencrypted
PKCS8 for private keys. No key generation, certificate lookup or network access is
performed.

## Profiles

| Profile | Operations | Result |
| --- | --- | --- |
| `rsa_oaep_sha256_mgf1_sha256_cfrg10` | `cipher.rsa.oaep.encrypt`, `cipher.rsa.oaep.decrypt` | RSA-OAEP with SHA256 and MGF1-SHA256 |
| `rsaes_pkcs1_v1_5_implicit_rejection_cfrg10` | `cipher.rsa.pkcs1_v1_5.decrypt` | Unauthenticated legacy bytes, including deterministic synthetic rejection bytes |

The `cfrg10` suffix pins the depadding rules in the September 2026 CFRG
implementation-guidance draft. OAEP ignores the leading encoded-message octet,
as that guidance requires. This deliberately differs from strict RFC8017
rejection for that octet; the remaining OAEP checks and label comparison apply.
Encryption produces ordinary RFC8017-compatible OAEP ciphertexts. The guidance
is a draft, not a replacement RFC or certification.

OAEP does not identify the sender. Legacy PKCS1v1.5 success does not establish
valid padding or authenticity: malformed padding returns synthetic bytes without
a validity flag. Authenticate the enclosing format before using those bytes.
The legacy profile is for offline recovery and must not become an online
decryption endpoint. There is no legacy encryption API or OAEP fallback.

## Keys and parameters

The `key` input contains exact DER or PEM file bytes. Select `spki_der` or
`spki_pem` when encrypting; select `pkcs8_der` or `pkcs8_pem` when decrypting.
The `input` port contains the message or ciphertext. Parameters require `profile`
and `key_format`. OAEP additionally accepts `label_hex`, with an empty label by
default; the same label must be used when decrypting.

RSA moduli must contain 2,048–8,192 bits. The backend requires two balanced primes
and an odd public exponent between 3 and 2³³−1. Imports validate canonical ASN1,
the RSA encryption algorithm identifier, private arithmetic and CRT consistency.
Encrypted private keys, attributes, PSS-only keys, trailing material and
noncanonical encodings are refused. PEM accepts canonical LF or CRLF framing.

## CLI and SDKs

For a PEM public key, save this profile as `oaep.json`:

```json
{"profile":"rsa_oaep_sha256_mgf1_sha256_cfrg10","key_format":"spki_pem"}
```

Encrypt the message and write the raw ciphertext:

```sh
cube cipher rsa oaep encrypt --parameters-file oaep.json --key-file public.pem --file message.bin > ciphertext.bin
```

For decryption, select `pkcs8_pem`, use the private key file and run
`cube cipher rsa oaep decrypt` with the same label. The CLI writes no output on
an OAEP failure. Native Python exposes `rsa_oaep_encrypt(data, key, parameters_json)`,
`rsa_oaep_decrypt` and `rsa_pkcs1_v1_5_decrypt` with the same argument structure.
JavaScript/TypeScript uses the operation ID with byte inputs `input` and `key`.
All interfaces execute Core algorithms. Play requires explicit private retention
consent and applies its native transport limits.

## Limits and failures

| Resource | Maximum |
| --- | --- |
| Encoded key | 65,536 bytes |
| Parameters JSON | 10,240 UTF-8 bytes |
| OAEP label | 4,096 bytes |
| OAEP plaintext | k−66 bytes, where k is the modulus width in bytes |
| Ciphertext | Exactly k bytes, at most 1,024 |
| Legacy result | k−11 bytes |
| System entropy requests | 65,536 bytes per operation |

OAEP encryption uses fresh OS/browser randomness. Private operations always
receive fresh randomness for backend blinding. Entropy failure returns no result;
there is no caller-controlled seed parameter. Wrong labels, padding and
out-of-range ciphertext representatives share `RSA operation failed` without
partial output. Public length/size limits and invalid key/configuration errors
remain distinct.

Owned decoded private buffers and backend keys are wiped on drop. Caller,
interpreter, parser and OS copies are not guaranteed erased. See
[backend qualification](../../contributing/asymmetric-backends.md) for source,
interoperability and randomized-output verification evidence.
