# Stream backend qualification

Core owns stream initialization and arithmetic. CLI, native Python and
JavaScript/TypeScript/WebAssembly adapt the same implementations; catalog
presence alone does not establish execution support. The
[stream reference](../reference/topics/stream-ciphers.md) owns public parameters,
representations and byte limits.

## Maintained Rust backends

| Package | Exact version | Source and license | Rust minimum |
| --- | --- | --- | --- |
| `rc4` | 0.2.0 | [RustCrypto](https://docs.rs/crate/rc4/0.2.0/source/), MIT OR Apache-2.0 | 1.85 |
| `hc-256` | 0.6.0 | [RustCrypto](https://docs.rs/crate/hc-256/0.6.0/source/), MIT OR Apache-2.0 | 1.85 |
| `rabbit` | 0.5.0 | [RustCrypto](https://docs.rs/crate/rabbit/0.5.0/source/), MIT OR Apache-2.0 | 1.85 |

Versions and checksums are pinned in `Cargo.lock`. The enabled `zeroize` feature
wipes supported owned backend state on drop. Caller buffers, parser and
interpreter copies, temporary stacks and OS memory are outside this guarantee.
These portable Rust packages require no external cipher library. Their MIT
notices are retained in `THIRD_PARTY_NOTICES` and included in SDK packages.

RC4 is broken. An explicit drop count preserves interoperability and does not
repair RC4. HC-256 and Rabbit qualification here does not establish an independent
security audit or a constant-time guarantee. These raw profiles provide no
authentication or key/nonce reuse tracking.

The 2026-11-12 package-path review used RustSec revision
[`af7b3ec`](https://github.com/RustSec/advisory-db/tree/af7b3ec33460b8e2de8d422e16d628518891a519).
Its complete tree had no `crates/rc4`, `crates/hc-256` or `crates/rabbit` advisory
paths. This is narrower than a full dependency audit and gives no absence-of-
vulnerabilities guarantee.

## Independent answers

The owning `stream/reference/vectors.json` records literal expected outputs and
their sources. RFC 6229 supplies RC4 keys and explicit stream offsets. RFC 4503
Appendix A supplies Rabbit integer test vectors, converted explicitly to the
little-endian key, IV and output-block representation selected by the profile.
HC-256 literals are cross-checked against the independently maintained
[Go implementation's fixed corpus](https://github.com/deatil/go-cryptobin/blob/e938a8032604741a6b5ac21127f648648b86b084/cipher/hc/hc_test.go).

Core public-executor, actual CLI, installed native Python, compiled Wasm and
private native graph suites use those same independent expected values in both
directions. They also check exact profile coverage, empty and prefix handling,
key/IV lengths, resource ceilings, private errors and explicit output retention.
Round trips supplement the fixed answers; they cannot replace them.


## Direct specification implementations

VMPC, VMPC-KSA3, Spritz, HC-128 and Trivium use Core-owned Rust state machines.
VMPC follows the original [FSE 2004 specification](https://www.vmpcfunction.com/vmpc.pdf)
and its key/IV size interval. Spritz follows the October 2014
[Rivest/Schuldt specification](https://people.csail.mit.edu/rivest/pubs/RS14.pdf),
including addition/subtraction and the separate stop-delimited IV construction.
HC-128 follows [Wu's specification](https://www3.ntu.edu.sg/home/wuhj/research/hc/hc128.pdf).
Trivium uses the eSTREAM 80-bit key/IV byte convention and all 1152 initialization
clocks. Arrays and owned permutation/FSM state use `Zeroizing` or explicit drop
wiping; this does not establish erasure of compiler temporaries.

RC4A uses the PRGA from [Paul/Preneel FSE 2004 section 4.2 and Figure 3](https://www.esat.kuleuven.be/cosic/publications/article-40.pdf).
The paper leaves the K2 PRG unspecified. The exact named profile fixes it to the
first K1-length RC4 bytes without discard, then independently schedules both
permutations. Expected values come from an independent Go transcription of that
specified construction; no published end-to-end literal for this precise
initialization was located. This is a narrower oracle than an external maintained
RC4A implementation and is not an independent security audit.

VMPC and HC-128 answers are generated independently by Bouncy Castle 1.83 Java
engines; original VMPC Table 6 also checks output positions 0, 252 and 1020. Spritz
and Trivium answers use the independently maintained Go backend at the frozen
revision above, with Spritz's three published eight-byte answers checked as well.
Corpora include nonzero input, variable key/IV boundaries, odd output lengths and
HC-128 table transitions. Oracles remain validation tools and are never SDK
runtime dependencies.


## Additional portable reference profiles

SOSEMANUK follows the [designers' specification](https://www.rocq.inria.fr/secret/Anne.Canteaut/Publications/sosemanuk.pdf),
with full Serpent24 initialization, snapshots after rounds 12/18/24, Serpent S2
output filtering and the specified field arithmetic. Core computes those steps
in portable Rust. The independent C++ reference is frozen at CryptoStreams
`b92d96ad16679c24a9402b0d33378d1f318db23a`. Its benchmark setting 25 includes
all 24 Serpent rounds and all five output groups; setting 24 omits the final group.
Seven published eSTREAM literals were checked before generating nonzero input
and intermediate key-length answers. The C++ code is a validation oracle only.

Grain v1 and ISAAC32 fixed answers use independently compiled Bouncy Castle 1.83.
The [Grain v1 upstream engine](https://github.com/bcgit/bc-java/blob/e40316b6e25816c1e302c98b5fc1ba6e2dfdfc0c/core/src/main/java/org/bouncycastle/crypto/engines/Grainv1Engine.java)
identifies the exact 80-bit revision. ISAAC follows
[Jenkins' state generator](https://www.burtleburtle.net/bob/rand/isaacafa.html)
and the explicitly named BC seed/output convention; it does not claim an
interchangeable seed API or a system random source. Core implementations own
and wipe their state; Java is never a runtime dependency.

Strumok-256/512 follows DSTU 8845's reference state transitions and explicit
little-endian words. Lookup tables retain the MIT notice from
[`dstu8845` 1.0.0](https://github.com/alexyer/dstu8845/tree/9c1099c469fd03edfcabcf33d9e8ced58b94e41b)
and the BSD-2-Clause attribution of the
[C reference](https://github.com/outspace/dstu8845/tree/752b1e0c5f69f95e128738cf3c450ac8b49675a2).
Both notices ship in `THIRD_PARTY_NOTICES`. The upstream crate's native-endian
transmutation and big-endian panic are replaced with safe explicit LE loads.
Its marker-only `ZeroizeOnDrop` implementation is not used as an erasure claim:
Core owns `Zeroizing` state arrays. Fourteen C-reference answers include eight
external fixed literals and cross-block odd-length data. This establishes
interoperability with the pinned sources, not independent certification to the
regional standard or a security audit.

PANAMA follows the [Daemen/Clapp 1998 reset/push/pull construction](https://jacobfilipp.com/DrDobbs/articles/DDJ/1998/9812/9812c/9812c.htm).
The Go reference supplies a fixed key-only literal. Its existing IV initializer
is reached through a validation-only adapter, without modifying arithmetic.
Little-endian oracle values explicitly convert each key/IV/output word from the
reference's big-endian representation. Both key-only and IV profiles use the
full 32 discarded pull rounds. The broken PANAMA hashing construction is separate
from these raw stream profiles.


## SNOW, ZUC and SOBER qualification

SNOW 2.0 follows the designers' reference state transitions, using the shared
SOSEMANUK field arithmetic and Core's existing Rijndael S-box. Independent answers
use the original C reference frozen in SUPERCOP at
`a351f2c29235512a042d4b6989d241a67e86ad23`; its eSTREAM wrapper's IV/output
word conversion is explicit in the validation oracle. SNOW 3G uses the
[SAGE 1.1 Annex 4 reference mirror](https://github.com/P1sec/CryptoMobile/blob/e53b92924ca8f43e8caf45b31aff5785414e3d93/C_alg/SNOW_3G.c),
with the ascending key/IV word-array convention independently checked against all
four fixed raw-generator test sets, including the 10,000-byte case. Neither C
implementation is linked into Core or an SDK.

ZUC-128's 256-byte substitution tables retain the Bouncy Castle MIT notice in
`THIRD_PARTY_NOTICES`. Core owns the portable modulo-2^31-minus-1 recurrence,
bit reorganization and nonlinear function. Independent Bouncy Castle 1.83 answers
include all four fixed GSMA raw-generator cases and odd-length nonzero data.
The profile's 64 KiB raw-generator bound does not imply support for the separate
EEA3 framing protocol or its bounds. SNOW and ZUC are historical interoperability
profiles without a certification or constant-time guarantee.

SOBER-128 follows the
[LibTomCrypt reference](https://github.com/libtom/libtomcrypt/blob/6c6d5104de66f3ca0dfd7b68540ef86869982b07/src/stream/sober128/sober128_stream.c)
under its public-domain dedication. Four independent C-reference answers include
the published `test key 128bits` case and nonzero odd-length input. Core owns and
wipes the register state and bounds constant generation to 65,536 iterations;
a private budget-exhaustion test verifies explicit failure rather than a partial
stream. The C reference is a validation tool only.


## SEAL, F-FCSR and LEX qualification

SEAL 3.0 follows the public-domain Wei Dai/Leonard Janke
[Crypto++ reference](https://github.com/weidai11/cryptopp/blob/5873a73a231c95771f5bbf3466a484952fb22780/seal.cpp),
with fixed L=32768 bits per outside position. Core reuses RustCrypto SHA1's
compression function with the exact key-derived initial state and unpadded index
block; using a complete SHA1 hash would be a different construction. A separate
C++ oracle mechanically adapts the upstream SEAL body and uses an independently
written FIPS180 compression. Its complete 4096-byte published Crypto++ literal
was checked before generating eight answers, including two position increments,
odd lengths, both output word orders and the final position. Core rejects
position exhaustion before returning any output; it never wraps the counter.
The oracle adaptation and published anchor are interoperability evidence, not
an independent security audit. SHA1 is used only as specified inside historical
SEAL, without a collision-resistance claim.

F-FCSR-H follows the permissively licensed FCSR project 2005 reference, frozen
in [CryptoStreams](https://github.com/crocs-muni/CryptoStreams/blob/b92d96ad16679c24a9402b0d33378d1f318db23a/streams/stream_ciphers/estream/f-fcsr/f-fcsr-h.cpp).
All five state words and all 160 initialization clocks are used. Its wrapper's
128-bit size macros disagree with the actual ten-byte key/IV loads; Core accepts
exactly those ten-byte inputs and names this 2005 revision explicitly. Six C++
reference answers include nonzero data and the all-zero degenerate state. This
broken historical construction is distinct from the later H-v2/16 setup.

LEX follows the public-domain February 2008 version 2.0
[designer reference](https://github.com/crocs-muni/CryptoStreams/blob/b92d96ad16679c24a9402b0d33378d1f318db23a/streams/stream_ciphers/estream/lex/lex.cpp).
Core's existing Rijndael module owns the key schedule, substitution, row shifts
and column mixing used by LEX. Ordinary AES initialization remains unchanged;
only the explicit LEX helper exposes its different leaky-round construction.
Five independently compiled C++ reference answers cover both zero and nonzero
IVs, multiple 40-byte blocks and odd tails. Only the complete AES-128 profile is
exposed: the reference's fixed 44-word key schedule must not be mistaken for
safe support of its comments' 192/256-bit key suggestions. Full notices for all
three reference sources ship with the SDK package.
