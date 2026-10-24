# Contributing to Cube

## Setup

Complete the [README Python setup](README.md#python-quick-start) to create and
activate a virtual environment.
`rust-toolchain.toml` requests Rustfmt, Clippy, and LLVM tools. Run these commands
from the repository root:

```sh
python -m pip install --group pyproject.toml:dev
cargo build --workspace --locked
maturin develop --manifest-path bindings/python/Cargo.toml --locked
```

Use `maturin develop` again after changing Rust code used by the SDK. The Python
package only exports the native functions; it contains no algorithm fallbacks.

## Quality checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
ruff check bindings/python
ruff format --check bindings/python
python -m pytest bindings/python/tests
```

Apply formatting with `cargo fmt --all` and `ruff format bindings/python`.
Python tests must use the installed native extension. To test a distributable
wheel as CI does:

```sh
maturin build --manifest-path bindings/python/Cargo.toml --locked --out dist
python -m pip install --force-reinstall dist/*.whl
python -m pytest bindings/python/tests
```

The wheel glob is POSIX shell syntax; on PowerShell, pass the wheel path from
`Get-ChildItem dist/*.whl` to pip. This builds and installs locally without publishing.

## Core coverage

```sh
cargo install cargo-llvm-cov --version 0.9.1 --locked
cargo llvm-cov --package cube-core --locked --fail-under-lines 80 --ignore-filename-regex '(/tests/|/target/)'
cargo llvm-cov report --lcov --output-path lcov.info --ignore-filename-regex '(/tests/|/target/)'
```

The first command that runs coverage prints a human-readable summary and fails
below 80% core line coverage. The report command exports the same run as LCOV.
Only cube-core is instrumented; integration test files and build artifacts are
excluded from reports. Do not commit `lcov.info` or `target/` output. Improve tests
when coverage fails; do not lower the threshold. External Codecov setup is described
in the README and does not control this local threshold.

## Add an operation

Choose an existing core module by responsibility. Implement and document one
byte-oriented function, or a text function when text semantics are inherent.
Use typed errors and maintained crates for cryptographic primitives.

Add known-answer and edge-case unit tests, public API tests when needed, then
expose the operation through the relevant CLI and Python APIs. Update command
discovery, Python exports and type stubs, interface tests, and affected user
examples. SDKs must call Rust rather than reimplement algorithms. The concise
[add-operation skill](.agents/skills/add-operation/SKILL.md) records this workflow.

## Issues and pull requests

Search [existing issues](https://github.com/cubearc/cube/issues) before opening a
bug report or feature request. Use a concise English title without category,
priority, or milestone prefixes. Describe expected input, output, edge cases, and
tests for proposed operations. Labels and milestones organize scope;
[CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1) holds status and priority.
Keep planning in GitHub rather than repository backlog files.

For a contribution, fork the repository, create a focused branch, and open a pull
request against `main`. Reference the related issue; use `Closes #NUMBER` only
when the change fully resolves it.

Keep each pull request focused on one logical change and explain the problem and
observable behavior. Include the checks you ran and any limitations. Keep
dependency additions justified, the lockfile updated, and documentation accurate.
Write comments, documentation, errors, and contributor templates in English.
Follow the [Code of Conduct](CODE_OF_CONDUCT.md).
