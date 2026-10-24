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
- Document public Rust APIs and update CLI/Python examples when behavior changes.
  Keep architecture notes brief. Recipes belong in `cubearc/recipes`.
- Keep planning in GitHub Issues and CubeArc Roadmap. Use descriptive issue titles
  without prefixes and Project fields for status and priority.
