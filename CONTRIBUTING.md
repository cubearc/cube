# Contributing to Cube

## Setup

Complete [Python installation](docs/python.md#installation) to create and
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
cargo run --locked -p cube-core --example operation_contracts -- --check
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
when coverage fails; do not lower the threshold. External Codecov uploads do not
control this local threshold. The
[CI workflow](.github/workflows/ci.yml) attempts an optional upload using
`CODECOV_TOKEN`; upload failures do not fail the local coverage gate. Maintainers
must configure the repository in Codecov before the badge can show received
reports. See [Codecov token documentation](https://docs.codecov.com/docs/codecov-tokens)
for upload authentication; keep tokens in GitHub Actions secrets.

## Add an operation

Choose an existing core module by responsibility. Implement and document one
byte-oriented function, or a text function when text semantics are inherent.
Use typed errors and maintained crates for cryptographic primitives.

Add a static descriptor to the owning module's `OPERATIONS` slice. Use a unique
stable namespaced ID (for example, `encoding.hex.encode`), explicit semantic
input/output type names, parameter constraints, a concise example, determinism
and reversibility where meaningful. IDs are compatibility contracts: do not reuse
an ID for different behavior. Add a new module to `operations::catalog` only once;
consumers reuse its descriptors instead of maintaining parallel catalogs.
Descriptors describe direct functions; they do not execute them. For example, an
ASCII-only case operation would declare text input/output, document preservation
of other Unicode, and keep its descriptor beside the function.

In modules with optional [profile facts](docs/discovery.md), author the descriptor
and its annotation together in the module's `PROFILED` rows; project the original
public `OPERATIONS` slice from those rows. Register the module once with the
existing catalog. Keep variant text sourced from the descriptor and bounds from
the actual implementation constants. Do not add a second operation-ID map or
infer execution support from annotations. Preserve default discovery JSON and
verify both legacy and opted-in views within the shared discovery output budget.

Add known-answer and edge-case unit tests, public API tests when needed, then
expose the operation through the relevant CLI and Python APIs. Update command
discovery, Python exports and type stubs, interface tests, and affected user
examples. SDKs must call Rust rather than reimplement algorithms. The concise
[add-operation skill](.agents/skills/add-operation/SKILL.md) records this workflow.

### Operation contract Definition of Done

Every implemented operation is part of the complete versioned contract catalog,
including direct operations without a pipeline adapter. Existing Rust declarations
and `schemars::JsonSchema` on their real data types are the authoring source. Keep
custom serialization annotations accurate (bytes, hex and decimal integers).
Use module-owned `contracts::Entry` rows for named ports, richer parameter DTOs,
implementation-enforced ceilings and known-answer fixture callbacks. The callback
must call the actual operation and check an independent expected result.

For every new or changed operation:

- Keep its ID/version, input/parameter/output/error schemas, defaults, bounds,
  exact representations, sensitivity and explicit capability requirements accurate.
- Regenerate the versioned catalog and reference; do not hand-edit either file:
  `cargo run --locked -p cube-core --example operation_contracts`.
- Add positive and invalid/boundary contract/result fixtures; preserve applicable
  Rust, CLI and native Python behavior. Run the contract exporter with `--check`.
- Add/review the synthetic invocation and contract SHA-256 in
  `crates/cube-core/src/operations/contracts/conformance.json`. The complete
  `conformance_matrix` test executes every ID through its owning callback or a
  test-only direct API driver and rejects missing/duplicate examples, version or
  schema drift, unknown fields and incompatible actual results. Keep fixture
  drivers out of product execution paths; never lower the exact-ID gate.
- Provide/update the required safe typed Play adapter and usable output handling.
  Missing integration stays open; `pipeline=null` does not hide an operation.
- Keep sources read-only, processing bounded and secrets uncaptured by default.
  Schema metadata grants no filesystem/network permissions.

CI checks owning-module registration, complete concrete schemas, actual fixture
conformance and generated-file drift. A minimum catalog size is not a completeness
check. Human-written task guides remain concise; the generated reference owns the
complete metadata listing. See [contract architecture](docs/contracts.md).

### Add a bounded pipeline adapter

An operation descriptor does not automatically permit pipeline execution. Add a
small explicit entry/action in `pipeline/adapters.rs` only for a concrete need,
reuse the descriptor types and validators, and call the existing core function.
Declare its exact adapter-contract version and conservative output/work bounds
in `pipeline/execute.rs` before enabling it. Keep file/network capabilities out
of adapters. Test descriptor agreement, exact output, invalid parameters, adjacent
types and limit boundaries across Rust, CLI and native Python. A breaking adapter
or engine contract needs a new corresponding version; package versions alone do
not establish recipe compatibility. Community definitions and their compatibility
fixtures belong in `cubearc/recipes`, using the public recipe check interface.

## Documentation

Keep the [README](README.md) as a short entry point. Put the first-use tutorial in
[Getting Started](docs/getting-started.md), interface workflows in the
[CLI](docs/cli.md) and [Python](docs/python.md) guides, capability choices/limits in
[Operations](docs/operations.md), and implementation boundaries in
[Architecture](docs/architecture.md). Update the relevant guide when behavior changes;
add README content only when it helps someone choose or start using Cube.

The operation catalog owns operation metadata, Clap owns command arguments,
Rustdoc owns the Rust API reference, and the Python stubs own SDK signatures.
Do not copy full catalogs/signature lists into Markdown. Validate changed examples
against the CLI and real Python extension, use original local fixtures, and check
relative links/anchors. Documentation-only work should not change application APIs.

### Rust API reference

From the repository root:

```sh
cargo doc -p cube-core --no-deps --open
```

The source Rustdoc describes public functions, types and constraints. Keep examples
close to implementation and run doctests when changing them.

## Git and Commit Conventions

Follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).
Use `type(scope): description`, omitting `(scope)` when it adds no useful context.
Write concise, imperative descriptions in English, such as "add" or "fix".

| Type | Use for |
| --- | --- |
| `feat` | New functionality |
| `fix` | Bug fixes |
| `refactor` | Internal restructuring without changing behavior |
| `test` | Test additions or improvements |
| `docs` | Documentation changes |
| `ci` | CI workflow changes |
| `build` | Build configuration or dependency changes |
| `chore` | Repository maintenance |
| `perf` | Performance improvements |

```text
build: initialize Rust workspace
feat(core): add base64 encoding
feat(cli): implement decoding commands
feat(python): expose core operations
test(core): cover invalid hexadecimal input
ci: add code coverage checks
docs: improve contribution guidelines
```

Mark breaking changes with `!` before the colon or a `BREAKING CHANGE:` footer,
and explain the impact.

Keep commits small and atomic: each should represent one logical, working change.
Include its implementation, matching tests, and affected documentation together
when appropriate. Always document new or updated functionality in the relevant
API reference, examples, setup instructions, or architecture notes. For internal-only
changes, explain the purpose and validation in the commit or PR description.
Separate unrelated work into different commits. Run the relevant
[quality checks](#quality-checks) before committing, and exclude secrets, generated
artifacts, temporary files, and known broken code. When commits are authorized
and Git write access is available, commit completed logical units incrementally.

Maintain a clean, reviewable history by making commit boundaries and messages
clear. Preserve existing commits; force-pushing, amending, squashing, or otherwise
rewriting history requires explicit authorization. At merge time, maintainers may
choose a squash merge for a focused pull request when repository settings allow
it. That merge choice does not authorize rewriting existing branch history.

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
dependency additions justified and the lockfile updated.
Write comments, documentation, errors, and contributor templates in English.
Follow the [Code of Conduct](CODE_OF_CONDUCT.md).

### Complete and deliver the work

Before an authorized push, finish the scoped implementation, tests, documentation,
and relevant quality checks. After pushing, verify that the commits reached the
intended remote branch and check the CI results. Address failures before declaring
the work complete.

Update related GitHub Issues and CubeArc Roadmap with the delivery status and
links to the commit or PR and validation results. Close an issue and mark its
Project item Done only when its acceptance criteria are validated and the work
has reached the target branch. A push to a feature branch alone does not complete
an issue. Keep unfinished tasks open and describe any remaining work or blockers.
