# Authenticate and decrypt an age file

`cipher.age.decrypt` version1 decrypts a complete binary agev1 file with one
caller-supplied X25519 identity. It returns plaintext only after the header MAC,
every payload tag, the final marker and end of input have been verified.
A failure returns no plaintext, including when a later chunk fails.
Authentication does not establish who sent a file.

Rust exposes `cube_core::ciphers::age::{Settings, decrypt}`; native Python exposes
`age_decrypt(data, parameters_json)`. JavaScript/TypeScript uses `cube.execute`
with byte `input` and `output` ports. Every interface uses the same Core adapter
and maintained age backend.

## Supply one private identity

Both settings are required:

| Field | Accepted value |
| --- | --- |
| `profile` | `age_v1_x25519_binary` |
| `identity` | One canonical uppercase74-character Bech32 X25519 identity |

The identity checksum is validated. Unknown fields, multiple identities and
identity-file comments are rejected. Settings JSON is capped at2048 UTF-8 bytes.
Keep real identities out of shell arguments, logs, recipes and sessions.

Create a private `identity.json` file. This example uses a **published CCTV test
identity**, which must never protect actual secrets:

```json
{"profile":"age_v1_x25519_binary","identity":"AGE-SECRET-KEY-1EGTZVFFV20835NWYV6270LXYVK2VKNX2MMDKWYKLMGR48UAWX40Q2P2LM0"}
```

```sh
cube cipher age decrypt --parameters-file identity.json --file encrypted.age > plaintext.bin
```

```python
import cube
settings = '{"profile":"age_v1_x25519_binary","identity":"AGE-SECRET-KEY-1EGTZVFFV20835NWYV6270LXYVK2VKNX2MMDKWYKLMGR48UAWX40Q2P2LM0"}'
plaintext = cube.age_decrypt(encrypted_bytes, settings)
```

The CLI writes stdout only after complete authentication. It does not create an
output destination itself; shell redirection can create an empty file before
execution even when decryption fails.

## Exact format and boundaries

The profile follows [C2SP agev1.1.0](https://c2sp.org/age@v1.1.0): X25519 and
HKDF-SHA256 unwrap the file key; HMAC-SHA256 authenticates the textual header;
ChaCha20-Poly1305 authenticates STREAM chunks using the payload nonce, chunk
counter and final marker. All Base64 fields use canonical unpadded encoding.
There is no cipher, KDF, identity or serialization fallback.

| Resource | Core ceiling |
| --- | --- |
| Complete binary ciphertext |16MiB, including header/nonce/tags |
| Plaintext |16MiB |
| Header through the MAC newline |65536 bytes |
| Recipient stanzas, including extensions |8 |
| Caller identities |1 X25519 identity |

Allocation-free header framing checks these ceilings and canonical grammar before
the maintained parser allocates header objects or attempts recipient cryptography.
Opaque canonical extension stanzas, including GREASE, count toward the same caps
and are authenticated with the header. They are ignored: no alternative identity,
KDF, plugin, process or network lookup is invoked. Scrypt stanzas are rejected
before any KDF work; armor and other versions are unsupported. SSH, PQ and plugin
identities, passwords, compression and encryption are not part of this operation.

Native Play retains its existing8KiB input and16KiB output transport limits and
requires explicit private-retention consent. Larger files can use Core, CLI or
the SDKs within their own transport budgets. No linear recipe adapter or automatic
capture is enabled for private identities and plaintext.

Malformed canonical headers, unsupported profiles/stanzas, resource limits and
an authentication-or-identity-match failure remain distinct safe outcomes.
Wrong identities, damaged MACs/tags, missing final chunks and trailing data never
return partial plaintext. Owned quarantine buffers are wiped on failure; caller,
OS, parser and interpreter copies are not guaranteed erased.

See [backend and envelope decisions](../../contributing/authenticated-envelopes.md)
for dependency, interoperability and verification evidence. No independent audit
or certification is claimed.
