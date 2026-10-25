# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source toolkit for transforming data and investigating files.
Use it to decode unfamiliar representations, work with classical ciphers,
inspect binary content, or reproduce a sequence of analysis steps.

The Rust library, `cube` command-line interface (CLI), and Python SDK share the
same Rust implementations. Binary operations preserve raw bytes; text conversion
is explicit.

## What you can do

- Encode/decode data, compare exact hashes/checksums and transform binary units.
- Work with classical/book ciphers, Unicode selectors/statistics, A1Z26, Morse,
  Braille dot masks and integer base conversion.
- Inspect file signatures, EXIF, hex, strings and binary differences; validate
  supported embedded ZIP structures and recover selected bytes safely.
- Analyze bounded PNG8/JPEG/GIF/BMP/WebP images, reconstruct raw rasters, and extract
  exact colors, ordered pixel/bit streams or binary QR payloads.
- Analyze bounded PCM spectra/spectrograms, extract sample bits, reverse WAV frames,
  inspect FLAC metadata and review DTMF/Morse candidates with explicit assumptions.
- Compose typed linear pipelines and versioned JSON recipes.
- Run bounded local investigations with evidence, reports and verified replay.

Classical ciphers and repeating-key XOR are puzzle tools, not secure encryption.
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

Decode Base64 into exact bytes without adding a newline:

```sh
cube decode base64 --text 'SGVsbG8=' --output raw
```

This writes `Hello`. Without `--output raw`, the decoder prints `48656c6c6f`.

Apply a Caesar shift:

```sh
cube cipher caesar --shift 3 --text 'Attack at dawn!'
```

Output: `Dwwdfn dw gdzq!`.

Create and inspect a small local file (POSIX shell):

```sh
printf 'Cube' > sample.bin
cube inspect sample.bin
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
- [Architecture](docs/architecture.md): core ownership, interfaces and extension contracts.
- [Contributing](CONTRIBUTING.md): development setup, checks and contribution workflow.

For exact arguments, use `cube --help` and `cube <command> --help`.
Use `cube ops list --json` for operation metadata. Rustdoc and the included Python
stubs are the API references; see [Contributing](CONTRIBUTING.md#rust-api-reference).

## Community

Report bugs and request features in [GitHub Issues](https://github.com/cubearc/cube/issues).
Development is tracked in [CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1)
and repository [milestones](https://github.com/cubearc/cube/milestones).
Community recipe definitions belong in [cubearc/recipes](https://github.com/cubearc/recipes).

Contributions are welcome; read [Contributing](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities through the
[security policy](SECURITY.md), rather than posting sensitive details publicly.

Cube uses the [Apache License 2.0](LICENSE). Dependency notices are in
[THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES) and included in Python distributions.
