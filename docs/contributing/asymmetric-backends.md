# Asymmetric backend qualification

Core owns key parsing, profile validation and cryptographic dispatch. The CLI,
native Python, JavaScript/TypeScript/WebAssembly and private Play calls adapt
Core operations. Agreement is distinct from encryption; metadata does not grant
permission to retain secrets.

## ML-KEM qualification

Reviewed on 2026-11-17. `libcrux-ml-kem` 0.0.11 (Apache-2.0, upstream
`42d68bd49b244b09bd02626c6ffa95e54dc64f91`) provides the portable implementation
and exact expanded-key format for the three parameter sets in
[FIPS 203](https://csrc.nist.gov/pubs/fips/203/final). The published potential
errata clarify the Appendix A table index and the comment in Algorithm 15. The
portable implementation includes upstream formal source verification for parts
of the algorithm. This qualification does not establish module certification
or machine-code timing guarantees. The Core-owned copies of coins and returned
private material are wiped on drop. Backend and interface copies may persist.

Test-only entropy injection exercises deterministic key generation and
encapsulation against independent ACVP data at
[`975de31eb83d87039ec88934fdc47d8c312b892d`](https://github.com/usnistgov/ACVP-Server/tree/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files).
Tests compare valid and implicitly rejected decapsulation outputs to exact ACVP
bytes. Fresh key pairs and encapsulations also qualify by agreement between two
calls. This relation verifies every output without substituting a random result
for a fixed known answer. Interoperability uses the named FIPS 203 profiles and
their exact byte lengths. See [ML-KEM reference](../reference/topics/ml-kem.md).

Kyber compatibility calls the distinct Libcrux Round 3 SHAKE decapsulation API
for all three historical parameter sets. Key-generation domain separation,
encapsulation preprocessing and the rejection-output KDF differ between
historical Kyber and the standardized ML-KEM algorithms. The independent
[original v3.0 reference](https://github.com/pq-crystals/kyber/tree/8e9308bd0f25fa698e4f37aba216249261f8b352/ref)
produced one valid and one modified ciphertext per parameter set. Core tests
check the exact historical outputs and the different result through the final
ML-KEM profile for the same bytes. The backend's historical Kyber code is
outside the upstream ML-KEM formal-verification claim.

Libcrux 0.0.11 matched the original valid Kyber outputs. The modified-ciphertext
vectors exposed a different rejection derivation. The pinned local source patch
uses the historical pre-key `z` for Kyber rejection and retains the FIPS 203
derivation and shared constant-time selection. The pinned ACVP tests also
exercise the FIPS 203 behavior after the patch.

## Alternate KEM qualification

The fixed decapsulation profiles use portable Rust backends. The exact
encoding and recognized reference bytes are part of each profile.

| Profile | Backend | Independent reference |
| --- | --- | --- |
| Salted FrodoKEM-640-SHAKE | `frodo-kem` 0.1.0 (`c1dfcd2ddf55b48218bfda359f3138b32d5ff5cd`) | Microsoft `PQCrypto-LWEKE` `a2f9dec8917ccc3464b3378d46b140fa7353320d`, salted SHAKE KAT count 0 |
| HQC-128, February 2026 submission | `hqc-kem` 0.1.0-rc.0 (`1a50023ab1dcebaaf500646589c76a3f0efade1f`) | HQC reference `161cd4f`, level 1 KAT count 0 |
| Classic McEliece 460896f, Round 4 | `pq-mceliece` 0.3.0 (`e34039fc546a70db3991b98ef1f7ec6ccf734f3c`) | liboqs 0.15.0 portable reference, deterministic NIST KAT seed stream |
| BIKE-L1, Round 4 | `oqs-native-bike` 0.16.0 (MIT and Apache-2.0) | liboqs 0.15.0 independent C answers for valid and modified ciphertexts |
| NTRU-HPS-2048-509, Round 3 | `oqs-native-ntru` 0.16.0 (MIT) | liboqs 0.15.0 independent C answers for valid and modified ciphertexts |

The two RustCrypto backends and `pq-mceliece` provide Apache-2.0 or MIT
licensing. Notices accompany the source and SDK packages. Only the selected
parameter-set features are enabled. The crate review and interoperability
tests do not establish a security audit or certify a protocol. The HQC profile
names the submission date to prevent confusion with later encodings.

The recognized valid cases are compared byte for byte. Core also tests
modified ciphertexts, short and oversized inputs, and wrong profiles. Core
wipes its owned shared-secret buffers and discards and wipes FrodoKEM's
internal decoded message. Backend and caller copies may persist. See
[alternate KEM profiles](../reference/topics/alternate-kem.md) for API bounds.

BIKE and NTRU use the independent portable Rust port of liboqs at
[`b4e41be8f4edba98ef1193db81c7a7d3f3c9ed96`](https://github.com/DeveloperBeau/liboqs-rust/tree/b4e41be8f4edba98ef1193db81c7a7d3f3c9ed96).
Both compiled for WebAssembly and matched the C reference including implicit
rejection. A single optimized x86-64 macOS probe measured about 5.4 ms for
BIKE and 0.32 ms for NTRU. These are compatibility profiles. Treat BIKE as
having the security claims documented by upstream, which does not claim
IND-CCA for this profile.

## Homomorphic ciphertext qualification

Reviewed on 2026-11-17. Microsoft SEAL 4.1.2 produced a distinct ciphertext
pair and evaluator result for BFV, BGV and CKKS. The SEAL oracle imported
the actual CLI output and recovered 3 modulo 17 for BFV and BGV, and
(4.75, 2.125) within 10⁻⁵ for CKKS. The tagged wire profiles use the
official modulus tables and the fixed domain and level described in the
[reference](../reference/topics/homomorphic-evaluation.md). The TFHE-rs 1.8.2
Boolean reference verified the linear NOT result, including decryption.
An AND probe serialized a 130,479,476-byte server key and exceeded the
16 MiB proposal. The default NOT gate needs no server key and is 3,228
bytes including the tag. SEAL and TFHE-rs were reference executables
only. Cube ships synthetic test vectors and public ciphertext arithmetic.

Paillier and Damgård–Jurik use the shared maintained `num-bigint` and
`num-integer` arithmetic on public operands only. Independent Python
integer calculations with OpenSSL-generated synthetic keys encrypted
inputs that cross the plaintext modulus, then decrypted the product to
5 in both profiles. The implementation covers deterministic public
evaluation of one pair with a 256 KiB direct memory reservation. Validated
public moduli and private operations remain with the source library.

## Educational lattice qualification

The original [NTRU paper](https://www.ntru.org/f/hps98.pdf) and
[Regev paper](https://cims.nyu.edu/~regev/papers/qcrypto.pdf) define the
construction. Independent small Python oracles generated synthetic keys
and exact ciphertexts for the documented teaching parameter sets.
Core uses bounded system randomness in normal calls. Conformance verifies
the known ciphertext as well as each fresh result with the matching inverse
operation. These are arithmetic demonstrations with no implementation-derived
security claim. Details are in the [educational lattice guide](../reference/topics/lattice-encryption-examples.md).

## ECDH profiles

Reviewed on 2026-11-17: `x25519-dalek` 2.0.1 (BSD-3-Clause) uses
`curve25519-dalek`; `p256` 0.14.0 (Apache-2.0/MIT) uses RustCrypto
`elliptic-curve`/`primeorder`. Both are portable Rust with fixed-curve secret
scalar multiplication and practical key zeroization. Explicit dependency
features omit key generation and unrelated signature APIs. This source review
is not an independent security audit or certification.

X25519 follows [RFC7748](https://www.rfc-editor.org/rfc/rfc7748), including
high-bit masking and input reduction, with mandatory zero-output rejection.
P-256 uses SEC1 points and the shared-x convention verified by
[RFC5903 section8.1](https://www.rfc-editor.org/rfc/rfc5903#section-8.1).
Checked `asymmetric/ecdh/vectors.json` stores independently specified inputs
and answers: the RFC7748 Alice/Bob exchange and both SEC1 encodings of the
RFC5903 responder. Expected values are not generated by Cube.

Core checks lengths before curve parsing and never infers a KDF or peer
authentication. Tests cover weak X25519 points, high-bit masking, invalid P-256
scalars, malformed/hybrid/off-curve points, unknown settings, redacted errors and
exact/one-over limits. Shared fixtures exercise the CLI, installed native
extension, actual Wasm and consent-gated native Play graphs and byte outputs.

## RSA and key serialization

Reviewed on 2026-11-17: `rsa` 0.10.0 (Apache-2.0/MIT) with
`crypto-bigint` 0.7.11 supplies the private arithmetic, OAEP and PKCS1v1.5
implicit rejection. The [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2023-0071.html)
was updated on 2026-11-08 to identify 0.10.0 as patched. The
[upstream tracker](https://github.com/RustCrypto/RSA/issues/711) records
fixed-width conversion, padding, base/exponent/CRT blinding and upstream
`cargo checkct` checks. Cube does not claim to have run that upstream tool or an
independent timing audit. Older releases and release candidates are not used.

The backend's default RNG feature is disabled. Every encryption/private operation
receives Cube's fallible OS/browser CSPRNG adapter, which caps requests at 65,536
bytes. There is no production deterministic-coin API; entropy failure returns no
result. Test-only injected failures exercise both encryption and private blinding.

Maintained `pkcs8`/`spki`/`pkcs1`/DER/PEM parsers own serialization. Cube bounds
transport and ASN1 components before bigint construction, accepts only the RSA
Encryption OID with NULL parameters, requires canonical re-encoding and validates
private arithmetic/CRT consistency. The selected subset is SPKI public and
unencrypted two-prime PKCS8v1 private keys, in canonical DER or PEM. Raw SEC1 and
RFC7748 keys remain curve-specific ECDH inputs. This is not PKI or a generic key
resolver. Separate binary key ports keep private key material out of graph
parameters without increasing native Play limits.

## Depadding and independent vectors

The profiles explicitly pin
[CFRG guidance draft 10](https://www.ietf.org/archive/id/draft-irtf-cfrg-rsa-guidance-10.html#section-7).
Its OAEP leading-octet rule deliberately accepts Wycheproof case 23, whose older
strict-RFC8017 expectation is invalid. The original corpus is unchanged;
`asymmetric/rsa/profile-decisions.json` records the exception and
[upstream implementation change](https://github.com/RustCrypto/RSA/pull/770).
All 37 OAEP cases are exercised with that explicit profile decision.

The 67 legacy cases also have independently computed expected outcomes from
Python's standard-library HMAC/SHA256 implementation of draft 10, including 19 exact
synthetic rejection messages. This oracle is a fixture-generation tool, not a
product algorithm. Publicly invalid ciphertext widths/representatives still fail;
private padding validity is never returned.

Additional ciphertexts come from cryptography 46.0.3/OpenSSL 3.5.4 and
PyCryptodome 4.0.1, covering DER, PEM LF/CRLF and actual 2048/2049/8192-bit moduli.
Exact 2049-bit generation was cross-checked by importing and decrypting with the
independent OpenSSL implementation. Known decryption answers and encryption
verified by native WebCrypto complement malformed-key, CRT, label, size,
redaction, entropy-failure and private graph tests. Licenses and source revisions
are retained with the checked corpora and dependency notices.

## Randomized conformance

Randomized encryption cannot compare fresh ciphertext bytes to one golden
ciphertext. A reviewed test fixture declares an explicit deterministic inverse,
fixed private inputs and the independently known plaintext. Test tooling validates
both the actual and stored reference ciphertext, requiring that each recovers the
original input. Corruption, wrong plaintext, missing assertions and attempts to
replace deterministic known answers are rejected.

Qualification reports retain actual/reference ciphertext hashes separately and
record the inverse-result comparison and hashes. Private outputs remain redacted.
These helpers are test tooling; no verifier endpoint, production RNG seed or
fixture dispatcher is added to the runtime API. Existing exact comparisons and
complete catalog coverage remain required.

## Finite-field DH qualification

The five RFC7919 Appendix A safe primes and generator 2 are pinned locally; their hexadecimal constants were independently checked against the RFC's formula using a 2,700-digit decimal calculation. Test-only Python integer exponentiation supplies independent public keys and shared secrets for every group. Production uses existing `crypto-bigint` 0.7.11 (Apache-2.0 OR MIT), including its fixed-window constant-time Montgomery exponentiation at fixed per-profile private widths. Variable-time Montgomery setup operates only on public named constants. No handwritten secret exponentiation, prime search, deterministic random generation or new arithmetic dependency is introduced.

The supported raw profile requires prime-order subgroup membership in addition to the RFC's range check. It exposes neither peer authentication nor a KDF. This qualification is implementation evidence, not a timing audit or certification; see the [key-agreement reference](../reference/topics/key-agreement.md) for exact encodings and bounds.

## SM2/SM3 qualification

Pinned `sm2` 0.14.0 (source revision `b48ce145dcab33b5140d89f8f5b183e9030717f6`, Apache-2.0 OR MIT) supplies SM2 field/point/scalar arithmetic, strict ciphertext codecs and encryption; `sm3` 0.5.0 (MIT OR Apache-2.0) supplies hashing with state zeroization enabled. Its default curve arithmetic uses `fiat-crypto` 0.3.0 (MIT OR Apache-2.0 OR BSD-1-Clause). Full notices accompany distributable SDKs. The upstream arithmetic explicitly lacks an independent audit; formal field arithmetic does not certify the whole scheme or its integration.

Source review found that upstream0.14's convenience decryption compares the tag with ordinary equality and does not reject an all-zero KDF. Core therefore owns a narrow strict decryption composition over the maintained point arithmetic, validated `Cipher` parser and SM3 primitives, using constant-time full-tag comparison, zero-KDF rejection and a wiped candidate plaintext. It does not call the unsuitable convenience decrypt helper or duplicate algorithms in an SDK. Unit injection exercises zero KDF blocks, partial blocks, counter ordering and failed entropy; public calls never accept deterministic coins or a substitute digest.

The [historical draft section7](https://datatracker.ietf.org/doc/html/draft-shen-sm2-ecdsa-02#section-7) is archived, not a final national or IETF standard. GB/T32918.4-2016 identifies SM2 encryption; the three supported profiles separately name their raw component order or OpenSSL-compatible DER representation. RFC8998 is not used as an encryption specification. Three independent OpenSSL4.0.3 vectors cover plaintext lengths9/256/27, with the same reviewed private key and external encryption/decryption. DER fields were independently serialized to both raw orders. See [SM2 reference](../reference/topics/sm2.md) for exact contract semantics.

## ECIES and DHIES qualification

Two complete BC1.83 ephemeral stream suites reuse Core's qualified P-256 or RFC7919 ffdhe2048 agreement, with KDF2-SHA256 and HMAC-SHA256 from maintained primitives. `hmac` 0.13.0 (MIT OR Apache-2.0) was already a resolved dependency; its direct use and SHA256 state zeroization are explicit. No new scheme arithmetic, group generation, static-static mode, IV framing or SDK algorithm is introduced.

The exact source is [BC1.83 IESEngine](https://github.com/bcgit/bc-java/blob/r1rv83/core/src/main/java/org/bouncycastle/crypto/engines/IESEngine.java), following IEEE1363a-style IES. A test-only Java17/Bouncy Castle1.83 oracle independently encrypts and decrypts twelve cases: both suites, plaintext lengths0/9/256, and empty/nonempty derivation and encoding vectors. Its JAR SHA256 is `82cf3a2af766c3bc874f6d36b9f20a8b99a8f09762dc776e8a227a45d8daaafb`. It is not included in the product or distributable SDKs.

Core verifies HMAC before allocating plaintext or applying the XOR key stream. Point/range/subgroup checks precede secret agreement. Negative tests cover ephemeral/body/tag/context changes, unknown suites, key encodings, transport bounds and entropy failure. The [IES reference](../reference/topics/ies.md) fixes every suite component and distinguishes these profiles from RFC9180 HPKE; matching a generic ECIES name alone does not establish interoperability.

## Raw ElGamal and Cramer–Shoup qualification

The exact raw group profiles reuse the RFC7919 prime and owning DH subgroup validation. `crypto-bigint`0.7.11 supplies fixed-width constant-time secret exponentiation and scalar arithmetic modulo q; no handwritten secret exponentiation or new arithmetic dependency is introduced. Cramer–Shoup's second generator is publicly reproducible from the pinned SHAKE256 procedure; the complete fixed-width/domain-bound transcript and label are part of the profile, not an implied universal encoding.

Nine independent vectors cover identity, small and full-width group messages and empty/binary labels. Python stdlib integer arithmetic and SHA256/SHAKE256 generated and checked their keys and equations. Test-only BC1.83 `GroupOracle` independently decrypts all nine; eight ciphertexts were generated by BC's engines, while one full-width ElGamal message uses Python encryption and BC decryption because BC's legacy byte encryption adapter accepts at most255 input bytes. The selected Core group profile supports the full256-byte group representation.

The BC Cramer–Shoup oracle uses a test-only digest serialization adapter: its first three integer updates are normalized to256 bytes, with this profile's domain prefix and binary label. It independently exercises BC's maintained Cramer–Shoup equations; it does not claim compatibility with BC's default variable signed-integer transcript. The oracle/JAR are not shipped. Core tests check exact scalar order bounds, subgroup/identity policy, key redaction, failed/degenerate entropy and bounded attempts. A valid-group ciphertext alteration changes ElGamal's plaintext but is rejected by Cramer–Shoup before output. See the [raw group reference](../reference/topics/group-encryption.md) for exact semantics and limitations.


## Historical educational references

Reviewed on 2026-11-17: the exact educational profiles are implemented in Core using `num-bigint` 0.5.1 (MIT/Apache-2.0), `num-integer` 0.1.47 (MIT/Apache-2.0) and the maintained `crypto-primes` 0.7.2 Miller–Rabin component (Apache-2.0/MIT). These dependencies are portable and already present in the resolved dependency inventory. Ordinary bigint arithmetic is variable-time and provides no private-heap zeroization guarantee. These profiles are reference implementations, not production secret primitives.

The construction and root/bit/knapsack semantics follow [Handbook of Applied Cryptography chapter 8](https://cacr.uwaterloo.ca/hac/about/chap8.pdf). Benaloh follows [Dense Probabilistic Encryption](https://www.microsoft.com/en-us/research/publication/dense-probabilistic-encryption/) with the explicit prime-r order conditions motivated by the [authors' ambiguity correction](https://arxiv.org/abs/1008.2991). The profile limits r to an odd prime at most 4093 and builds a complete, resource-charged lookup table. Rabin returns all verified roots; it does not claim a CCA-safe disambiguation envelope. Merkle–Hellman is explicitly broken.

Checked `asymmetric/historical_public_key/vectors.json` stores independently calculated synthetic answers: two Rabin moduli with exhaustively enumerated root sets, GM bit residues, Benaloh r=3/5/17/257 and a permuted eight-weight knapsack. The fixture generator uses Python standard-library integer arithmetic, rather than calling Cube. Negative tests cover root ambiguity, nonresidues/nonunits, invalid private order, malformed permutations, congruent but invalid knapsack ciphertexts, exact tokens, combined work/memory/report bounds and entropy failure. Each interface executes Core; randomized GM/Benaloh conformance uses independently known plaintext and deterministic private inversion.

See the [historical public-key reference](../reference/topics/historical-public-key.md) for every exact key field, transport, profile and limitation. Qualification is implementation evidence, not a security audit or primality certificate.
