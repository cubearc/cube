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

Core modules follow responsibility: `encoding`, `ciphers`, `hashing`, `binary`, `forensics`, and `image`.
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

Future pipelines may compose these functions with typed intermediate values.
There is no execution engine, plugin runtime, generic dispatcher or persistence
layer. Introduce abstractions only for a concrete implemented need. Recipes belong
in `cubearc/recipes`; this repository does not parse or execute recipes yet.
