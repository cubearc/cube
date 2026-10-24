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
primitive is implemented from scratch. Repeating-key XOR and ROT13 are small puzzle operations.
CRC-32/ISO-HDLC uses `crc32fast` and returns eight lowercase hexadecimal digits;
it is a checksum rather than a cryptographic hash.

Core modules follow responsibility: `encoding`, `ciphers`, `hashing`, and `binary`.
Binary transformations accept byte slices and return bytes. Encodings and digests
return strings with specified alphabets. ROT13 accepts UTF-8 text, rotates only
ASCII letters, and preserves other characters without normalization. Invalid
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

Future pipelines can compose these functions and handle transport or orchestration
outside the operations. There is no execution engine, operation registry, plugin
system, or persistence layer. Add an abstraction only when an implemented feature
needs it. Recipes will live in `cubearc/recipes` and may later describe compositions
against a versioned Cube interface; this repository does not parse recipes yet.
