# Cube JavaScript SDK

Source-built adapter over the shared Rust/Wasm runtime. Algorithms and canonical
contracts remain in `cube-core`. This package has not been published to npm.
See [the interface guide](../../docs/interfaces/javascript.md) for supported
paths, build/test commands, limits and revision-specific qualification evidence.

Install a locally produced artifact with `npm install /path/to/cubearc-cube-0.18.0.tgz`.
Use `createCube` from `@cubearc/cube` in Node.js, `@cubearc/cube/browser` in browser
modules and `createWorkerCube` from `@cubearc/cube/worker` for dedicated workers.
Discover complete canonical contracts separately from actual direct/pipeline
execution support. See `examples/node.mjs`, the compiled TypeScript example `examples/node.mts`, and
`examples/static/index.html`. The package retains `dist/THIRD_PARTY_NOTICES`.

Build and qualification commands run from the Cube source checkout, as explained
in the interface guide. Package installation requires no Rust compiler.
