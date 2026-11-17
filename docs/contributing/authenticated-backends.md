# Authenticated backend qualification

`ciphers::modern_aead` owns exact parameter validation, resource preflight,
detached tag transport and whole-message plaintext release. CLI and SDKs adapt
that Core implementation. Backend selection is declared once beside each exact
profile; no host supplies a fallback algorithm or rewrites a failed tag.

## Maintained implementations

| Backend | Version | Accepted use |
| --- | --- | --- |
| RustCrypto `aes-gcm` | 0.11.1 | AES128/256, nonce12, tag16; zeroize enabled |
| RustCrypto `chacha20poly1305` | 0.11.0 | IETF and XChaCha variants; zeroize enabled |
| RustCrypto `aes-siv` | 0.8.0 | Low-level nonce-free SIV-CMAC, total key32/64 |
| RustCrypto `aes-gcm-siv` | 0.12.1 | RFC8452 AES128/256, nonce12, tag16 |
| RustCrypto `ascon-aead` | 0.6.0 | Final SP800-232 Ascon-AEAD128; zeroize enabled |
| RustCrypto `ccm` | 0.6.1 | Explicit nonce/tag sizes with length coupling |
| RustCrypto `eax` | 0.5.0 | Full EAX, nonce16 and tag16 |
| RustCrypto `ocb3` | 0.2.0 | RFC7253, nonce12 and tag16 |
| `aegis` | 0.9.20 | RFC10032 128L/256 with tag16/32; pure Rust |
| RustCrypto `deoxys` | 0.2.1 | Deoxys-II128/256 with nonce15 and tag16 |

All have MIT or MIT/Apache software licenses; the distributed notices retain
applicable text. EAX uses `cipher`0.4/`aead`0.5 and the pinned zeroizing AES0.8.4
compatibility backend. Other AES constructions reuse AES0.9.3. No EAX algorithm
is reimplemented in a binding. AEGIS enables `pure-rust` and the reviewed0.6
traits adapter, uses `softaes`0.1.8 without host AES target features, and does not
require C, relaxed Wasm SIMD, system randomness or a browser crypto service.

RustCrypto documents an NCC audit for AES-GCM and ChaCha20-Poly1305 and timing
assumptions for multiplication-based authenticators. That is not an audit of the
current Cube composition or every crate revision. Ascon, CCM, OCB3, Deoxys and
other backends do not share a blanket independent-audit or constant-time claim.
Core wipes its owned key and tentative plaintext buffers; this does not establish
complete erasure of every backend's expanded state or host copies.

Primary sources include [RustCrypto AEADs](https://github.com/RustCrypto/AEADs),
[AEGIS](https://github.com/jedisct1/rust-aegis),
[SP800-232](https://csrc.nist.gov/pubs/sp/800/232/final),
[RFC8439](https://www.rfc-editor.org/rfc/rfc8439),
[RFC5297](https://www.rfc-editor.org/rfc/rfc5297),
[RFC8452](https://www.rfc-editor.org/rfc/rfc8452),
[RFC3610](https://www.rfc-editor.org/rfc/rfc3610),
[RFC7253](https://www.rfc-editor.org/rfc/rfc7253) and
[informational RFC10032](https://www.rfc-editor.org/rfc/rfc10032).

## Independent verification

The owning module keeps checked static vectors with original source file,
SHA256 and case identity. The corpus includes RustCrypto-distributed NIST CAVP,
RFC, Wycheproof and designer vectors, final Ascon KATs and AEGIS RFC vectors.
AES256 EAX/OCB interoperability cases come from the separate PyCryptodome4.0.1
implementation. SIV-CMAC256/512 cases preserve Wycheproof's tag-prefix convention
when normalizing to detached ports. The RFC5297 multi-component example remains
an explicit test so component flattening cannot pass through single-AAD vectors.

Raw fixture formats are versioned: current RustCrypto blobby has a counted header
and five fields per AEAD case; the older EAX fixture has six fields, including a
validity flag. Never guess the arity from a filename or treat a partial parse as
an accepted corpus. Selected files and normalized cases are bounded and offline;
no product operation downloads or executes arbitrary test-vector code.

Known-answer tests execute the real Core API and typed executor, then CLI,
installed Python, actual Wasm and private native Play ports. Negative tests cover
changed ciphertext, AAD, tags, keys, nonces and SIV boundaries; limits include CCM
message coupling, the16MiB combined bound and126 SIV components. Keep the canonical
fixture for every operation and generated-schema drift check with these tests.
