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
