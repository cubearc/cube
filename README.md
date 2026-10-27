# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source Rust toolkit for data transformation, cryptography,
digital forensics and investigative workflows. Its CLI, Python SDK and Rust API
share the same implementations.

- Decode data, transform text and bytes, and work with classical ciphers.
- Inspect local files, images, audio, video and saved web artifacts.
- Compose typed pipelines and recipes, and retain investigation evidence.

## Start

Install from source with Git, stable Rust and a C linker:

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo install --path crates/cube-cli --locked
cube decode base64 'SGVsbG8=' --output raw
```

The result is `Hello` without a trailing newline. See
[Installation](docs/installation.md) for prerequisites, Python setup and Rust use,
then follow [Getting Started](docs/getting-started.md).

## Documentation

- [Task guides](docs/guides/index.md): choose an investigation or transformation.
- [CLI](docs/interfaces/cli.md), [Python](docs/interfaces/python.md) and
  [Rust](docs/interfaces/rust.md): use the interface you need.
- [Reference](docs/reference/index.md): exact formats, profiles and operation contracts.
- [Cube Play](https://github.com/cubearc/play): use the engine in a local visual workspace.
- [Recipes](https://github.com/cubearc/recipes): reusable workflows and challenges.

## Contribute

Read [Contributing](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).
Report bugs in [Issues](https://github.com/cubearc/cube/issues); planning lives in
[CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1).
Report vulnerabilities through [Security](SECURITY.md).

Licensed under [Apache-2.0](LICENSE). Dependency notices are in
[THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).
