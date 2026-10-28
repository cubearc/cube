# Cube repository instructions

- Apply the local [ub-mode](.agents/skills/ub-mode/SKILL.md) by default when
  implementing or changing code. Read it before the first edit; no separate user
  invocation is required. Explicit user scope and repository instructions take
  precedence. The copy matches `unclebob/ub-mode` revision
  `57a8bdd28e45ed41900ace0bafa2ff76dd9ed70f`.
- `crates/cube-core` owns all algorithms in modules grouped by responsibility.
  Keep byte transformations byte-oriented and errors typed.
- `crates/cube-cli` adapts core operations to Clap commands.
  `bindings/python` adapts them to PyO3 functions with Python stubs and pytest tests.
  Never duplicate an algorithm in an interface or SDK.
- Use stable Rust, edition 2024, idiomatic Rust and Python, and small cohesive
  modules. Do not add speculative frameworks or dependencies.
- Write documentation, comments, errors, templates, and agent instructions in English.
- Use challenge for generic activities and recipe for reusable workflows. Follow
  [CubeArc product scope](docs/architecture.md#cubearc-product-scope) when positioning products.
- Validate from the repository root with `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and
  `cargo test --workspace --all-features --locked`.
- Follow CONTRIBUTING.md for native Python installation, Ruff, pytest, and core
  coverage. Keep the 80% core line threshold; do not suppress linter warnings.
- Document new and updated functionality in the same logical change. Keep public
  Rust APIs, CLI/Python examples, setup instructions, and brief architecture notes
  accurate. Explain internal-only changes in the commit or PR description.
  Recipes belong in `cubearc/recipes`.
- Follow the [shared documentation guide](https://github.com/cubearc/docs/blob/main/content/contributing/documentation.md).
  Keep README content focused on first use; update the canonical task/reference
  page and affected examples instead of repeating a feature across guides.
  Maintain `docs/SUMMARY.md` and preserve old links when moving content.
- Every new or changed operation must have a complete canonical versioned contract:
  named input/output ports, typed parameter/result/error schemas, limits and privacy
  policy. Regenerate `contracts/v1/catalog.json` and `docs/reference/operations.md`,
  run real contract-conformance fixtures, and update the safe Play adapter/viewer
  handling. Direct-only operations remain in the catalog. Missing schema, docs or
  required Play integration is incomplete work, not an implicit scope exception.
  Keep exact catalog/fixture parity in `conformance_matrix`, including reviewed
  contract hashes and actual typed results; publish its per-operation CI report.
- Keep planning in GitHub Issues and CubeArc Roadmap. Use descriptive issue titles
  without prefixes and Project fields for status and priority.

## Git Workflow

- Follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):
  `type(scope): description`, with scope optional. Write concise, imperative
  descriptions in English.
- Create small, focused, atomic commits, each representing one logical change.
  Keep related implementation, tests, and documentation together when appropriate;
  separate unrelated changes.
- Run relevant formatting, linting, and tests before committing. Never commit
  secrets, generated artifacts, temporary files, or known broken code.
- Preserve existing history. Never force-push, amend, squash, or rewrite existing
  commits without explicit authorization.
- When Git write access is available and commits are authorized, commit completed
  logical units incrementally rather than accumulating unrelated changes.
- After an authorized push, verify the remote commits and CI results, and update
  related GitHub Issues and Project status with delivery and validation links.
  Close tasks only after their acceptance criteria pass and the work reaches the
  target branch. Keep incomplete tasks open and report what remains.
- Follow [CONTRIBUTING.md](CONTRIBUTING.md#git-and-commit-conventions) for details.
