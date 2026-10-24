# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source Rust toolkit for encoding, classical ciphers, hashing,
and binary transformations. Use the Rust library, `cube` CLI, or Rust-backed
Python package. All interfaces share the same implementations.

## Capabilities

| Operation | Behavior |
| --- | --- |
| Base32 | Uppercase, padded RFC 4648 standard encoding and strict decoding |
| Base64 | Padded RFC 4648 standard encoding and strict decoding |
| Hex | Lowercase encoding; decoding accepts either case |
| ROT13 | Rotates ASCII letters; preserves case and non-ASCII characters |
| SHA-256 | Lowercase hexadecimal digest |
| XOR | Byte-wise XOR with a nonempty repeating key |

ROT13 and repeating-key XOR are classical puzzle tools, not secure encryption.
Decoders reject whitespace. Base32 and Base64 reject invalid padding and trailing
bits; Base32 also rejects lowercase letters.

## Build and install

Install stable Rust through [rustup](https://rustup.rs/) and a platform C linker.
Then build from source:

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo build --workspace --locked
cargo install --path crates/cube-cli --locked
cube --version
```

For an uninstalled development build, use `cargo run -p cube-cli --` in place of
`cube`. No published registry packages are required.

## CLI quick start

```sh
cube --help
cube ops list
cube encode base32 --text "foo"                   # MZXW6===
cube decode base32 --text "MZXW6==="               # 666f6f
cube decode base32 --text "MZXW6===" --output raw  # foo (no added newline)
cube encode base64 --text "Hello"                 # SGVsbG8=
cube decode base64 --text "SGVsbG8="               # 48656c6c6f
cube decode base64 --text "SGVsbG8=" --output raw   # Hello (no added newline)
cube encode hex --text "Hello"                    # 48656c6c6f
cube decode hex --text "48656c6c6f" --output raw   # Hello
cube cipher rot13 --text "Hello"                  # Uryyb
cube hash sha256 --text "Hello"
cube binary xor --text "Hello" --key "key"        # 230015070a
cube binary xor --text "Hello" --key-hex 6b6579    # 230015070a
```

Omit `--text` to read stdin until EOF. Input bytes, including newlines, are
preserved. Use `printf`, rather than `echo`, when a trailing newline is unwanted:

```sh
printf 'Hello' | cube encode base64
printf 'AP8=' | cube decode base64 --output raw > bytes.bin
cube binary xor --key-hex ff --output raw < bytes.bin > transformed.bin
```

Decoding and XOR default to **hexadecimal plus a newline** so arbitrary bytes
have a predictable representation. `--output raw` writes exact bytes without a
newline. Other operations write text followed by a newline. ROT13 requires UTF-8;
encoding, hashing, and XOR accept arbitrary stdin bytes. `--text` and `--key`
use UTF-8. Specify exactly one of `--key` or `--key-hex` for XOR.

Results go to stdout and errors to stderr. Exit codes are `0` for success
(including help/version), `1` for invalid operation input or I/O failure, and
`2` for invalid command usage. A downstream closed pipe exits successfully.

## Python quick start

Use Python 3.10 or newer with Rust installed. From the repository root:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install .
```

On Windows, create the environment with `py -m venv .venv` and activate it with
`.venv\Scripts\Activate.ps1` in PowerShell.

```python
import cube

assert cube.base32_encode(b"foo") == "MZXW6==="
assert cube.base32_decode("MZXW6===") == b"foo"
assert cube.base64_encode(b"Hello") == "SGVsbG8="
assert cube.base64_decode("SGVsbG8=") == b"Hello"
assert cube.hex_encode(b"\x00\xff") == "00ff"
assert cube.hex_decode("00FF") == b"\x00\xff"
assert cube.rot13("Hello, 世界") == "Uryyb, 世界"
assert cube.sha256(b"Hello") == (
    "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969"
)
assert cube.xor(b"Hello", b"key") == b"\x23\x00\x15\x07\x0a"
```

Binary inputs and outputs use `bytes`; encoded text and ROT13 use `str`.
Encode Unicode explicitly, for example `cube.sha256("Olá".encode("utf-8"))`.
Invalid encoded input and empty XOR keys raise `ValueError`; incorrect argument
types raise `TypeError`. Type stubs are included. The local distribution name is
`cubearc-cube`; the import name is `cube`. Package registry availability has not
been established.

## Tests and development

```sh
cargo test --workspace --all-features --locked
python -m pytest bindings/python/tests
```

Complete the [development setup](CONTRIBUTING.md#setup) before Python tests.
That guide also covers formatting, linting, and core coverage commands.
CI enforces at least 80% core line coverage locally before attempting
an upload to Codecov. Badges display the external services' current status.

Maintainers: activate `cubearc/cube` in Codecov. Private repositories require a
repository upload token stored as the GitHub Actions secret `CODECOV_TOKEN`.
For a public repository, either use that token or disable public-repository token
authentication in the CubeArc organization's **Global Upload Token** settings.
Public fork pull requests can upload without the upstream secret. If using a token,
add it separately to Dependabot secrets for Dependabot uploads. See
[Codecov's token documentation](https://docs.codecov.com/docs/codecov-tokens).
Unconfigured or failed external uploads do not fail CI; local coverage failures do.
The coverage badge is not evidence of an upload until Codecov has been connected
and received a report.

## Architecture and direction

`crates/cube-core` owns the algorithms, grouped by encoding, ciphers, hashing, and
binary transformations. `crates/cube-cli` and `bindings/python` adapt those APIs
to their respective interfaces. See [the architecture overview](docs/architecture.md).
Open the Rust API reference with `cargo doc -p cube-core --no-deps --open`.

Future work may add puzzle-solving operations and composable pipelines. Those
features are not implemented here. Recipes will live separately in
[`cubearc/recipes`](https://github.com/cubearc/recipes).

Track bugs and feature requests in [GitHub Issues](https://github.com/cubearc/cube/issues).
Planning lives in [CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1),
with release scope defined by [milestones](https://github.com/cubearc/cube/milestones).
The [operations catalog issue](https://github.com/cubearc/cube/issues/1) links the
next planned operations; it does not describe additional implemented features.

Contributions are welcome through [CONTRIBUTING.md](CONTRIBUTING.md). Community
participation follows the [Code of Conduct](CODE_OF_CONDUCT.md). Report security
issues using [SECURITY.md](SECURITY.md).

## License

Cube is licensed under the [Apache License 2.0](LICENSE).
