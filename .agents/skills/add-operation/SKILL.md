---
name: add-operation
description: Add a Cube operation to the Rust core and expose it through the existing CLI and all supported SDKs.
---

# Add an operation

1. Read root AGENTS.md and nearby operations. Choose the owning module in
   `crates/cube-core/src` by responsibility; keep interface adaptation separate.
2. Implement a small public function in the core with rustdoc. Use bytes for
   binary data and a typed error for invalid input. Reuse maintained crates for
   cryptographic primitives. Add dependencies only when justified.
3. Add known-answer, empty-input, invalid-input, and relevant Unicode tests near
   the implementation. Exercise round trips and determinism where applicable;
   use `crates/cube-core/tests` for public API behavior.
4. Expose the operation through the relevant existing Clap commands, PyO3
   functions and JavaScript/TypeScript/WebAssembly bindings. Update `ops list`,
   SDK exports/type declarations and executable discovery/availability. Test the
   installed native Python extension and actual packaged Node.js/browser SDK,
   including known answers, errors and limits. Catalog presence does not prove
   runtime support. Apply this parity rule to changed operations too. Keep
   algorithms solely in the core.
5. Follow the [Cube documentation workflow](../cube-docs/SKILL.md). Update the
   canonical reference and affected task examples. Add a guide only for a new
   reader task; change the README only for positioning or first use. Keep
   architecture details in contributor docs and do not claim future features.
6. Run Rust formatting, Clippy, workspace tests, native Python installation,
   Ruff, pytest, and the 80% core coverage check using CONTRIBUTING.md.

7. Supply/update the canonical versioned input, parameter, output and error
   schemas and named ports; derive schemas from actual Rust types. Add a real
   conformance fixture, regenerate the contract catalog/reference through the
   `operation_contracts` Cargo example, and run its `--check` mode. Complete the
   safe typed Play adapter and output handling; missing integration stays open.
