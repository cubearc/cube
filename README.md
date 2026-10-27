# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source Rust toolkit for data transformation, cryptography,
digital forensics, and investigative workflows.
Use it to decode unfamiliar representations, work with classical ciphers,
inspect binary content, or reproduce a sequence of analysis steps.

The Rust library, `cube` command-line interface (CLI), and Python SDK share the
same Rust implementations. Binary operations preserve raw bytes; text conversion
is explicit.

## What you can do

- Encode/decode data, compare exact hashes/checksums and transform binary units.
- Run bounded Brainfuck programs with byte I/O, wrapping 8/16/32-bit cells,
  execution reports and explicit width comparisons.
- Work with classical/book ciphers, Unicode selectors/statistics, A1Z26, Morse,
  Braille dot masks, flag Semaphore, directional clocks and integer base conversion.
- Apply Bacon/Polybius/Tap alphabets, explicit keyboard/keypad mappings and
  character-class patterns with source positions.
- Arrange Unicode scalars in exact grids and iterate bounded string combinations.
- Traverse scalar/word positions, inspect Unicode differences and extract exact
  numeric text evidence; convert caller-defined positional digit alphabets.
- Compare exact scalar inventories and evaluate bounded Boolean models; analyze
  explicit graph paths/cycles/edge-once trails and exact combinatorial counts.
- Compute exact bounded GCD/Bezout coefficients, modular inverses/powers and
  decimal digital roots through shared Rust, CLI and native Python APIs.
- Inspect file signatures, EXIF, hex, strings and binary differences; validate
  supported embedded ZIP structures and recover selected bytes safely.
- Analyze bounded PNG8/JPEG/GIF/BMP/WebP images, reconstruct raw rasters, and extract
  exact color layers, ordered pixel/bit streams or binary QR payloads. Combine
  compatible image pairs and exchange small exact sample grids through CSV.
- Analyze bounded PCM spectra/spectrograms, extract sample bits, reverse WAV frames,
  inspect FLAC metadata and review DTMF/Morse candidates with explicit assumptions.
- Inspect MP4/MOV/WebM metadata; extract MOV/JPEG frames and raw PCM8 tracks,
  and compare exact decoded frame samples with source provenance.
- Inspect saved HTML/HAR and pass explicitly selected embedded assets into
  existing transformations without executing content or following references.
- Validate supplied Base58Check/WIF/scalar formats and derive Bitcoin/Ethereum
  public addresses offline, with explicit private export and bounded candidate checks.
- Compose typed linear pipelines and versioned JSON recipes.
- Run bounded local investigations with evidence, reports and verified replay.

Classical ciphers and repeating-key XOR are useful for challenges and education;
they do not provide secure encryption.
See the [Operations Guide](docs/operations.md) for supported formats and limits.

## Install from source

You need Git, stable Rust and a C linker. The Python SDK also needs Python 3.10+.
These instructions build Cube from the checked-out source.

### CLI

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo install --path crates/cube-cli --locked
```

Ensure Cargo's binary directory is on `PATH`. To try Cube without installing it,
use `cargo run -p cube-cli --` in place of `cube`.

### Python

From the repository root, in a POSIX shell:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install .
```

The distribution is named `cubearc-cube`; import it as `cube`.
For Windows activation and setup details, see [Python installation](docs/python.md#installation).

## Quick start

Text/byte transformation commands accept literal text directly, `--file`/`-f`
for file contents, or stdin when no source is given. `--text`/`-t` remains available.

Decode Base64 into exact bytes without adding a newline:

```sh
cube decode base64 'SGVsbG8=' --output raw
```

This writes `Hello`. Without `--output raw`, the decoder prints `48656c6c6f`.

Apply a Caesar shift:

```sh
cube cipher caesar --shift 3 'Attack at dawn!'
```

Output: `Dwwdfn dw gdzq!`.

Create and inspect a small local file (POSIX shell):

```sh
printf 'Cube' > sample.bin
cube inspect sample.bin
cube encode base64 -f sample.bin
```

Or use Cube as a Python library:

```python
import cube

assert cube.base64_decode("SGVsbG8=") == b"Hello"
assert cube.caesar_encode("Attack at dawn!", 3) == "Dwwdfn dw gdzq!"
```

## Documentation

- [Getting Started](docs/getting-started.md): install, discover and run an operation.
- [CLI Guide](docs/cli.md): input, output, errors and practical workflows.
- [Python Guide](docs/python.md): installation, data types and library examples.
- [Operations Guide](docs/operations.md): choose a capability and understand its limits.
- [Discrete Foundations](docs/discrete-foundations.md): inventories, logic, graphs and counts.
- [Brainfuck](docs/brainfuck.md): strict execution, diagnostics and resource limits.
- [Architecture](docs/architecture.md): core ownership, interfaces and extension contracts.
- [Contributing](CONTRIBUTING.md): development setup, checks and contribution workflow.

For exact arguments, use `cube --help` and `cube <command> --help`.
Use `cube ops list --json` for operation metadata. Rustdoc and the included Python
stubs are the API references; see [Contributing](CONTRIBUTING.md#rust-api-reference).

## Community

Report bugs and request features in [GitHub Issues](https://github.com/cubearc/cube/issues).
Development is tracked in [CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1)
and repository [milestones](https://github.com/cubearc/cube/milestones).
Reusable investigation workflows, examples and community challenges belong in
[Cube Recipes](https://github.com/cubearc/recipes). See the
[CubeArc product scope](docs/architecture.md#cubearc-product-scope) for the intended
roles of Cube Play and Cube Play Platform.

Contributions are welcome; read [Contributing](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities through the
[security policy](SECURITY.md), rather than posting sensitive details publicly.

Cube uses the [Apache License 2.0](LICENSE). Dependency notices are in
[THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES) and included in Python distributions.

[Bounded puzzle analysis](docs/puzzle-analysis.md) covers caller-dictionary anagrams and
word ladders, exact finite probabilities/dice and explicit spatial lag measurements.
