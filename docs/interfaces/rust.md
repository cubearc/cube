# Rust interface

Core operations live in `crates/cube-core`. With a trusted Cube checkout, add a
path dependency from your application's Cargo manifest:

```toml
[dependencies]
cube-core = { path = "../cube/crates/cube-core" }
```

Adjust that path to your actual checkout. The shared workspace currently uses
edition 2024. No published crates.io package is assumed.

From Cube's repository root, build the full public API reference:

```sh
cargo doc -p cube-core --no-deps --open
```

Rustdoc owns public signatures, types and their examples. Use the
[operation reference](../reference/operations.md) for cross-interface contracts,
and [Contributing](../../CONTRIBUTING.md) for extension and verification.
