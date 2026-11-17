# Authenticated envelope selection and backend qualification

Decision recorded2026-11-17 for milestone31 owners#247 and#293: implement the
bounded binary agev1/X25519 decryption profile in [age files](../reference/topics/age-files.md).
This resolves the parser-cap gate from completed#78 and delivers a complete
recognized authenticated-file use case. It introduces no Cube envelope format.

## Candidate comparison

| Candidate | Concrete profile and inputs | Decision |
| --- | --- | --- |
| agev1/X25519 | Binary file, one caller identity; wrapped file key, authenticated header and STREAM payload | Implement the specified bounded decrypt-only operation |
| RFC7516 JWE | Compact five-part serialization with protected `alg`/`enc`, encrypted key, IV, ciphertext and tag; a narrow profile would have to fix algorithms and external-AAD policy | Compare as an alternative serialization; the selected file-decryption operation is age |
| RFC9180 HPKE | Explicit KEM/KDF/AEAD suite, mode and context inputs | A cryptographic construction, not a universal encrypted-file serialization; age supplies the selected complete file format |

Primary specifications: [agev1.1.0](https://c2sp.org/age@v1.1.0),
[JWE RFC7516](https://www.rfc-editor.org/rfc/rfc7516), and
[HPKE RFC9180](https://www.rfc-editor.org/rfc/rfc9180). These alternatives do not
create placeholder JWE/HPKE catalog entries. The implemented envelope is exactly
`cipher.age.decrypt`, not a generic packet or algorithm-negotiation framework.
It is deterministic for fixed file/identity; no reverse/encryption operation or
linear pipeline adapter is advertised.

## Parser gate and authentication boundary

Maintained [age0.12.1](https://docs.rs/age/0.12.1/age/) supplies X25519 wrapping,
header authentication and STREAM. Core preflights borrowed ciphertext before
calling `Decryptor::new_buffered`: at most16MiB input,65536 header bytes and8
stanzas. Fixed stack buffers validate canonical Base64; no unbounded header/body
allocation or KDF occurs during preflight. Unknown canonical recipient stanzas
are opaque authenticated data, bounded and ignored. Scrypt is rejected before
backend construction. One X25519 identity is the only dispatched identity.

The backend returns a streaming reader, so Core consumes it to EOF into an owned
zeroizing quarantine. Core returns bytes only after every chunk and final marker
pass. No streaming plaintext or partial success escapes. Identity/no-match and
cryptographic failures share a sanitized authentication outcome. Header grammar,
unsupported format and resource-limit errors remain distinguishable without
including identity material.

Initial tests exposed the upstream encoder's randomized GREASE extensions.
The preflight accepts those canonical opaque stanzas under the same caps, as the
format permits, and the exact16MiB test measures the header of the same writer
before choosing plaintext length. It does not assume a fixed generated header.

## Dependency, licensing and portability evidence

Pinned age0.12.1 source revision:
`7a854c9c8c02dab2ebea19e02e6693a5730a8b2a`. It offers MIT/Apache-2.0 licensing;
notices are retained with all newly resolved dependencies. Default features are
disabled: optional armor, SSH, plugin, CLI and asynchronous interfaces are not
enabled. The upstream base crate still includes its Rust HPKE/PQ and localization
dependencies; their presence grants no extra Cube execution profile.

Decryption performs no random generation. Its transitive getrandom0.2 JavaScript
backend is explicitly enabled for WebAssembly compilation; this does not cause
an RNG call in this operation. Native Linux/macOS/Windows and actual Node/browser
WebAssembly qualification remain the delivery gates. Beta/pre-1.0 library status
and absence of an independent Cube audit are not hidden by catalog metadata.

## Reproducible interoperability evidence

The checked normalized corpus records23 source-file hashes from the pinned
[upstream testkit](https://github.com/str4d/rage/tree/7a854c9c8c02dab2ebea19e02e6693a5730a8b2a/age/tests/testdata/testkit),
which incorporates [CCTV age vectors](https://github.com/C2SP/CCTV/tree/50a8ecf2a220f4c8bdc4f085789b8e85c26829e7/age).
CCTV offers0BSD/CC0/Unlicense; the0BSD text is retained. Offline normalization
only decompresses reviewed test fixtures, never product input.

All success payload hashes and failure outcomes were independently verified with
Go age1.3.2. Its official darwin/amd64 release archive SHA256 is
`1d1e4bc66e1427edad7739ae7616157de0e79db8b6d2a1497d7d9925fb06a539`, matched to
the published release asset digest before execution. The suite covers X25519,
multiple recipients, GREASE, no-match, noncanonical fields, bad header MAC,
missing final markers, later-chunk tags and trailing data. Core additionally
checks hostile scrypt, oversized headers, nine stanzas, exact16MiB valid input,
last-tag corruption at that ceiling and one byte over.

Native Play tests execute eligible cases with private consent and verify refusal
at its unchanged8KiB transport ceiling; full multi-chunk cases execute through
Core, CLI, installed native Python and actual Wasm. Validation logs distinguish
these scopes. The shared offline corpus establishes interoperability evidence,
not CAVP/FIPS validation or sender authentication.
