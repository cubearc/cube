# Checked offline cryptographic vectors

Decision recorded2026-11-17 for milestone31 owner#249: adopt checked static
fixtures with source provenance and explicit validity. Test data is not a public
operation, arbitrary-vector executor, network client or certification service.
Core algorithms remain the only runtime implementations used by the SDKs.

## Recognized formats and selected use

| Source format | Reviewed semantics | Adoption |
| --- | --- | --- |
| NIST CAVP response files | Named algorithm/profile, lengths/counts and hex fields; decryption `FAIL` cases are adversarial results | Normalize selected AES/XTS/KW/KWP cases into checked owning-module fixtures, preserving source/archive hashes and case identifiers |
| NIST ACVP symmetric JSON | Version/vector-set metadata, algorithm/revision, test groups and case IDs; prompt/expected response are separate artifacts | Use the protocol schema as a reference for profile, length, group and validity distinctions. The selected offline evidence is static CAVP/KAT data; no ACVP registration/server client or certification claim is introduced |
| Wycheproof JSON | Algorithm/schema, group key/IV/tag sizes, `tcId`, flags and valid/invalid/acceptable result labels | Adopt every133 AES128/256-GCM case matching the required96-bit nonce and128-bit tag:79 valid and54 invalid. Other profiles are outside these operation IDs; no acceptable case is promoted to valid |
| CCTV age testkit | Text metadata plus binary or fixture-compressed file, expected success/failure and plaintext SHA256 | Normalize23 selected cases; preserve original/decompressed hashes and verify with official Go age1.3.2. Fixture decompression never enables product decompression |
| Backend KATs and independent captures | Exact profile-specific bytes, source hashes/case IDs, published examples or producing program/version | Preserve AEAD/SIV, FF1/FF3 variants and OpenSSL interoperability evidence alongside the recognized datasets |

Primary references: [NIST CAVP](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program),
[ACVP symmetric JSON specification](https://pages.nist.gov/ACVP/draft-celi-acvp-symmetric.html),
[Wycheproof](https://github.com/C2SP/wycheproof), and
[CCTV age format](https://github.com/C2SP/CCTV/tree/50a8ecf2a220f4c8bdc4f085789b8e85c26829e7/age).
The reviewed ACVP page is the2026-10-30 Internet-Draft, work in progress; it is
not described as a final standard.

## Immutable evidence and bounds

`crates/cube-core/tests/fixtures/crypto/provenance.json` pins all10 checked
corpora and585 cases by SHA256, origin format and exact count. Its typed manifest
rejects duplicate/unknown fields, unknown corpus/format names, repeated entries,
noncanonical digests and excessive counts. Corpus names map to fixed repository
paths; no caller path or network location is executed.

The offline checker reads at most16MiB plus one sentinel byte before parsing.
It verifies the reviewed corpus hash before interpreting JSON, so altered fields,
including duplicate fields or unknown algorithm names, cannot silently redefine
an oracle. The manifest itself changes only through normal reviewed source edits.
It then checks at most65536 total cases,1MiB per normalized case,512KiB per string
field,65536 array items,64 object fields and depth8. The string ceiling covers
multi-chunk ciphertext encoded as Base64. These are test-data bounds, separate
from each operation's runtime budgets. No untrusted KDF work is dispatched by
this checker; executable suites use the fixed reviewed cases and Core limits.

```sh
cargo test --locked -p cube-core --test crypto_fixture_provenance --test crypto_vectors
```

The provenance suite exercises oversized files/fields/nesting, changed hashes,
duplicate/unknown manifest fields, unknown algorithms/formats and count ceilings.
The cryptographic suites execute the oracle bytes and distinguish expected
adversarial failures. Wycheproof's133 selected cases also execute through installed
native Python and actual Wasm; the broader owning corpora exercise Core, CLI,
SDKs and eligible private native Play transport.

Wycheproof is pinned at revision
`65ca494727a52944c204441c860b039e6786b99b`, schema `aead_test_schema_v1.json`.
The source file SHA256 is retained on each normalized case. Its Apache-2.0 terms
are retained. CCTV source revision/file hashes and its0BSD licensing option are
recorded in [authenticated envelope qualification](authenticated-envelopes.md).
NIST test examples are informal verification evidence; they do not grant a
CAVP/FIPS certificate or prove comprehensive resistance to cryptanalysis.

## Updating a corpus

Fetch and review a source revision explicitly outside product execution. Record
the original archive/file hash, license, algorithm revision and selected case IDs.
Normalize exact bytes and validity without calling Cube to manufacture expected
answers. Keep invalid cases separate from successful round-trip oracles. Compare
existing fixture identities, add independent expected results, and update the
checked manifest only after reviewing the data changes. Run the owning Core and
installed SDK suites, then the provenance and complete conformance checks.

A recognized schema alone does not qualify a backend. The actual cryptographic
known answers and failures, installed SDK execution, portable builds and unchanged
coverage gates remain required delivery evidence.
