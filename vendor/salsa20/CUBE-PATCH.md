# Salsa20 counter increment patch

Source: crates.io `salsa20` 0.11.0, RustCrypto stream-ciphers.
Upstream commit: `b516422ebb49ff0d9140b22ac8c2a627d10a452e`.
Crate archive SHA-256: `2f874456e72520ff1375a06c588eaf074b0f01f9e9e1aada45bd9b7954a6e42c`.
The source, licenses and upstream tests are retained. The only implementation
change is `get_block_pos().wrapping_add(1)` in `src/backends/soft.rs`.

The upstream unchecked block API generates the final valid block then increments
its local u64 counter with `+ 1`, which panics in debug builds. Explicit wrapping
matches release behavior. Cube preflights every requested block, discards the
one-call state and never generates a block after exhaustion. This does not change
the Salsa primitive, round schedule, serialization or earlier output.

Reproduction and regression: `cargo test -p cube-core --test stream`.
Upstream known-answer tests: `cargo test --manifest-path vendor/salsa20/Cargo.toml --features zeroize`.
Remove this vendor copy when a qualified upstream release fixes the increment.
