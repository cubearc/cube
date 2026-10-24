# Architecture

Cube is a Cargo workspace with three packages:

| Package | Responsibility |
| --- | --- |
| `crates/cube-core` | Reusable operations and typed input errors |
| `crates/cube-cli` | Clap arguments, stdin/stdout, and exit codes |
| `bindings/python` | PyO3 boundary, Python exports and type stubs, Maturin packaging |

The root `pyproject.toml` configures Maturin against `bindings/python/Cargo.toml`.
Keeping Python packaging at the workspace root lets source distributions preserve
the shared Cargo.lock and use the repository README and license directly.

## One implementation

Rust is the source of truth so the CLI and SDK agree on validation, byte handling,
and results. Each operation is an ordinary function in the core. Base32 uses
`data-encoding`, with a core check that rejects concatenated padded blocks.
Base58 uses `bs58` with the Bitcoin alphabet and no checksum extension.
Base64, hex, and SHA-256 also delegate to established crates; no cryptographic
primitive is implemented from scratch. Repeating-key XOR and Caesar are small puzzle
operations; ROT13 reuses Caesar with shift 13.
CRC-32/ISO-HDLC uses `crc32fast` and returns eight lowercase hexadecimal digits;
it is a checksum rather than a cryptographic hash.

Core modules follow responsibility: `encoding`, `ciphers`, `hashing`, `binary`, `forensics`, `image`, `text`, `symbols`, `numbers`, and `media`.
Binary transformations accept byte slices and return bytes. Encodings and digests
return strings with specified alphabets. Classical ciphers accept UTF-8 text, rotate
only ASCII letters, and preserve other characters without normalization. Caesar
normalizes signed 32-bit shifts before inversion to avoid overflow. Invalid
inputs return a core `Error` that retains the underlying decoder cause.

## Interface boundaries

The CLI reads exact stdin bytes or UTF-8 `--text` arguments. Decoding and XOR
print hex by default and offer explicit raw output. Only text operations require
UTF-8 conversion. Argument errors and operation failures have documented exit
codes and write errors to stderr.

The Python package exports functions from `cube._native`. PyO3 converts arguments
and return values; the core performs the work. Decoders and XOR return Python
`bytes`; input errors become `ValueError`. Stubs describe the small public API.
Maturin controls extension linking, so no Cargo feature enables extension linking
globally. Rust workspace checks can include all features without Python test
harness linker conflicts. Python tests load the compiled extension directly.

SDKs must not duplicate algorithms: a second implementation would create drift
in validation and semantics. Tests combine core known-answer vectors, byte round
trips, CLI process tests, and native Python smoke tests. Coverage targets core
source independently of bindings and enforces an 80% line minimum.

## Future integration

Future Java, JavaScript, Go, Kotlin, or Swift bindings should call the Rust core
and translate their own language's types and errors. Introduce a shared FFI only
when another implemented binding demonstrates a concrete need.

Static operation descriptors live beside their Rust implementations and are
assembled by `operations::catalog`. Stable IDs, semantic type names, parameter
constraints, determinism and reversibility describe operations without dispatching
them. Core lookup/filter errors are shared by CLI and Python. JSON discovery is
versioned; direct functions do not adopt a universal runtime value type.

The `forensics` module separates range-safe I/O, basic facts, signature evidence,
byte inspection, PNG framing and bounded EXIF parsing. Reader/writer APIs support
streaming without tying algorithms to paths or CLI processes. File paths must be
regular files; outputs use exclusive creation. Limits and unsupported formats are
explicit. `OperationError` adds stable categories/context and retains causes;
legacy decoder errors and public signatures remain compatible. Structured Python
results use a JSON transport boundary; algorithm bytes use native Python bytes.

The `image` module owns an immutable, validated PNG8 sample representation and
cohesive analysis, geometry/color, and bit-extraction modules. The maintained
pure-Rust `png` decoder/encoder uses explicit limits and identity transformations;
framing/CRC inspection reuses `forensics::png`. Indexed samples/palette/tRNS stay
separate until explicit expansion. No graphical library or native image framework
is required. Core methods return typed samples, structured facts or exact bytes;
CLI/Python only translate arguments/results. Derived image exports preserve
samples and relevant transparency but drop unrelated metadata. Histograms,
coordinate listings and extraction use documented bounds, without approximations
or implicit UTF-8. Static descriptors remain beside their owning implementations.

The `ciphers::classical` functions preserve ASCII case and other Unicode; keys
and modular arithmetic rules are explicit. `text` owns deliberate normalization
and Unicode-scalar transformations, with shared input/output bounds. Normalization
uses pinned `unicode-normalization` data (published as `text::UNICODE_VERSION` and
`cube.UNICODE_VERSION`); data changes require compatibility review. `symbols`
separates canonicalized Morse text from reversible Braille cell-mask bytes.
`numbers` converts integer representations with checked 128-bit magnitude
arithmetic, independently of byte encodings. CLI/Python adapters share these
functions and typed errors; new CLI stdin paths enforce bounds before buffering.
Legacy public APIs retain their original behavior and discovery command order.

The `media` module separates WAVE framing (`wave`), immutable selected PCM blocks
(`pcm`) and incremental exact waveform aggregation (`envelope`). Metadata skips
payloads with `Read + Seek`; extraction/envelopes share a concrete frame-aligned
streaming callback with fixed-size I/O buffers. Private PCM block fields preserve
format/channel/range invariants. Explicit signed amplitude interpretation is
separate from original bytes. Whole-request and per-block budget validation
precedes allocation/consumption, and result caps fail instead of truncating.
The small RIFF subset needs only existing serde and Rust standard-library I/O;
no new dependency, native library, ffmpeg subprocess or graphical/media runtime
is introduced. Linux/macOS/Windows use the same implementation and packaging.
Video, other codecs and spectral/detection policy remain outside current support.

The `pipeline` module composes an explicit allowlist of bounded operations through
immutable prevalidated plans. Static operation descriptors remain the metadata
source of truth, while small adapters reuse direct Rust algorithms and validators.
Bytes, text and an inert structured byte range cross this boundary; direct APIs
keep their ordinary types. Whole-plan type, version, parameter and conservative
size/work/capture checks precede execution. Captures are opt-in; cancellation is
cooperative between bounded steps. CLI and Python share this engine and grant
local I/O only through caller-selected paths. No plugin runtime, shell, network,
arbitrary dispatcher or persistence layer is involved. Recipes belong in
`cubearc/recipes`; the engine owns strict JSON parsing, validation and execution.

Recipe format 1 has exactly one symbolic input slot or a tagged inline value,
explicit output type, engine API and operation-contract versions. The same
bounded unique-key JSON parser serves all transports. Unknown fields, versions,
capability requests and adjacent type mismatches fail before algorithms execute.
The validated recipe owns only an immutable plan and optional inline memory.
Caller-provided bindings never resolve paths, environment values or expressions.
Validation reports redact data and parameters. Exact typed output checks expose
an independent compatibility-test boundary for community definitions without
copying algorithms or recipe collections into this repository.

The `investigation` module uses explicit rules/contracts rather than a second
algorithm engine. Triage reuses signature evidence and the available descriptor
catalog. Private session fields preserve session-local run/finding IDs, typed
provenance, input/result digests, separately linked user hypotheses and bounded
opt-in retention. A small run enum reuses existing hex/string functions and
validated pipeline plans. Direct operation APIs stay unchanged. JSON loading
shares the bounded unique-key parser and validates references/versions/states;
replay rejects identity/redaction gaps before executing. Human reports escape
untrusted strings, and JSON export independently selects results and parameters.
Only interface code opens caller-selected files or explicitly saves a new record;
there is no storage backend, implicit network capability or automatic persistence.
