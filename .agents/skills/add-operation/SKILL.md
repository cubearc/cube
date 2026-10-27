---
name: add-operation
description: Add a Cube operation to the Rust core and expose it through the existing CLI and Python SDK.
---

# Add an operation

1. Read root AGENTS.md and nearby operations. Choose `encoding`, `ciphers`,
   `hashing`, or `binary` in `crates/cube-core/src` according to responsibility.
2. Implement a small public function in the core with rustdoc. Use bytes for
   binary data and a typed error for invalid input. Reuse maintained crates for
   cryptographic primitives. Add dependencies only when justified.
3. Add known-answer, empty-input, invalid-input, and relevant Unicode tests near
   the implementation. Exercise round trips and determinism where applicable;
   use `crates/cube-core/tests` for public API behavior.
4. Expose the operation through the relevant existing Clap commands and PyO3
   functions. Update `ops list`, Python exports and type stubs, CLI integration
   tests, and native Python smoke tests. Keep algorithms solely in the core.
5. Update README examples or architecture notes only where the new behavior
   affects users. Do not describe future features as available.
6. Run Rust formatting, Clippy, workspace tests, native Python installation,
   Ruff, pytest, and the 80% core coverage check using CONTRIBUTING.md.

7. Supply/update the canonical versioned input, parameter, output and error
   schemas and named ports; derive schemas from actual Rust types. Add a real
   conformance fixture, regenerate the contract catalog/reference through the
   `operation_contracts` Cargo example, and run its `--check` mode. Complete the
   safe typed Play adapter and output handling; missing integration stays open.
