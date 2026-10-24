# Cube repository instructions

- `crates/cube-core` owns all algorithms, grouped into `encoding`, `ciphers`,
  `hashing`, and `binary`. Keep byte transformations byte-oriented and errors typed.
- `crates/cube-cli` adapts core operations to Clap commands.
  `bindings/python` adapts them to PyO3 functions with Python stubs and pytest tests.
  Never duplicate an algorithm in an interface or SDK.
- Use stable Rust, edition 2024, idiomatic Rust and Python, and small cohesive
  modules. Do not add speculative frameworks or dependencies.
- Write documentation, comments, errors, templates, and agent instructions in English.
- Validate from the repository root with `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and
  `cargo test --workspace --all-features --locked`.
- Follow CONTRIBUTING.md for native Python installation, Ruff, pytest, and core
  coverage. Keep the 80% core line threshold; do not suppress linter warnings.
- Document new and updated functionality in the same logical change. Keep public
  Rust APIs, CLI/Python examples, setup instructions, and brief architecture notes
  accurate. Explain internal-only changes in the commit or PR description.
  Recipes belong in `cubearc/recipes`.
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
