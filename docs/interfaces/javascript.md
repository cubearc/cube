# JavaScript SDK qualification

The JavaScript SDK is being developed in `bindings/javascript`. It is not yet a
published or release-qualified package. Milestone [#48](https://github.com/cubearc/cube/milestone/48)
tracks the remaining API, worker, packaging and compatibility work. The browser
portal is separate application work in `cubearc/docs`; it consumes this SDK.

## Build and test the actual runtime

Use a working stable Rust toolchain with `wasm32-unknown-unknown`, Node.js and the
`wasm-bindgen` CLI **0.2.129**, matching `Cargo.lock`:

```sh
rustup target add wasm32-unknown-unknown
cd bindings/javascript
npm ci --ignore-scripts
npm run build -- --dev
npm run test:qualification
npm run test:types
```

The default build uses the release profile; `--dev` is for local qualification.
`CARGO_TARGET_DIR`, `CARGO` and `WASM_BINDGEN` may select existing local tooling.
Generated `dist/` output is untracked. The build records the source revision,
dirty-source flag, source fingerprint, package version, canonical catalog hash
and actual Wasm SHA-256. A dirty development build does not identify a delivered
product revision and must not be adopted into the documentation source lock.

Tests execute real Core functions through the generated Wasm artifact. They check
all 53 currently exposed linear pipeline adapters against reviewed known answers,
strict recipe preflight, Unicode, exact bytes, captures and errors. Direct tests cover all 19 registered pure executors and
their exact named outputs. Serialization checks reject exhausted budgets before
visiting later data or encoding oversized byte buffers. The full
canonical catalog contains 329 operations at this baseline; this count is not a
promise of 329 JavaScript bindings. Native Core conformance remains a separate
required check.

## Exercise an installed package and its examples

From the SDK directory, build and create an inspectable tarball:

```sh
npm run build
npm pack --ignore-scripts
node scripts/verify-package.mjs ./cubearc-cube-0.18.0.tgz
```

The verifier installs that actual artifact into a separate temporary consumer,
with lifecycle scripts disabled and no Rust source/runtime fallback. It checks
public exports, exact bytes, recipes and declarations, then runs the packaged
`examples/node.mjs`. It also compiles the packaged `examples/node.mts` with the
locked TypeScript compiler and executes its emitted ESM JavaScript. The latter
expects `Café`, exact bytes `[0, 255]` and a captured UTF-8 byte intermediate.
This is execution of compiled TypeScript, not just a typecheck. Run the verifier
with each declared Node.js runtime. The package carries the unchanged project
license and third-party notices alongside its Wasm assets.

## Use the local ESM API

```javascript
import { createCube } from './dist/node.mjs';

const cube = await createCube();
const result = cube.execute({
  operation: 'encoding.base64.decode',
  version: 1,
  inputs: { input: 'AP8=' },
});
// result.output is Uint8Array [0, 255], never an implicit UTF-8 string.
```

The direct API uses Core's actual module-owned pure executor registry, with exact
canonical named inputs, outputs, parameters and operation versions. Inspect
`cube.executors` for the installed surface; the current baseline exposes 19 direct
executors, including Base64, structured composition and multi-port pixel export.
Operations without a registered pure executor fail with `unsupported_operation`;
having a pipeline adapter does not grant direct API support. Declarations are
generated from the actual Wasm executor metadata and canonical parameter schemas.
Node `Buffer` is accepted wherever `Uint8Array` is required. Wasm copies
input bytes and returned buffers are caller-owned copies; neither passing nor
receiving a buffer transfers or detaches the caller's original.

The browser entry is `dist/browser.mjs`. Its default Wasm URL is relative to that
module, preserving non-root static deployment paths. Pass `{ wasm: new URL(...) }`
or `{ wasm: bytes }` to configure it. Fetches require successful responses and a
bounded asset body; SHA-256 is checked before instantiation. A wrong MIME type
does not require streaming instantiation. Missing, interrupted and mismatched
assets produce typed initialization failures. Browser use requires Web Crypto in
a secure context, such as HTTPS or localhost.

Initialization is shared within one module realm. Repeated or concurrent calls
with the same source share one promise and API object. A conflicting source is
rejected; use a separate worker/module realm for another runtime. Source bytes are
copied before asynchronous work, including Node Buffer inputs. A failed load can
be retried. Unexpected runtime traps require a fresh worker; they are not empty
successes.

## Discover contracts and pipeline support separately

`cube.catalogJson()` returns the complete canonical contract wire after checking
its runtime hash against the package identity. This includes unbound operations.
Preserve the raw wire when hashing or exporting it. JavaScript's JSON parser
cannot represent every wide numeric schema annotation; use the canonical decimal
annotations for exact bounds rather than rounding a Number.

`cube.pipelineAdapters` lists actual Core adapter IDs, versions and input/output
kinds. Core prevalidation still determines parameter and adjacent-type
compatibility. Generate a per-operation local inventory with:

```sh
node scripts/qualification-report.mjs /tmp/cube-qualification.json
```

The version-2 report verifies the complete raw catalog and Core-computed
per-contract hashes against reviewed fixtures, then executes every advertised
direct and pipeline path through real Wasm. It records runtime/build identities,
operation and adapter versions, exact actual/expected result digests and capture
verification. Synthetic output values are retained only when Core permits it;
private outputs are compared before hashing and their data is omitted. Inputs
and parameters are not retained. A missing fixture, capability/contract drift or
known-answer mismatch fails the command and invalidates any stale success report.

The report distinguishes actual qualified paths, missing direct bindings and
host-adapter requirements for every catalog ID. It qualifies only the recorded
runtime. Real-browser qualification can write the same evidence with
`CUBE_QUALIFICATION_REPORT=/path/to/report.json`; the JavaScript CI workflow
retains separate Chromium, Firefox and WebKit artifacts. It is not another operation
registry or a release compatibility declaration. Reader/seek APIs can often use
owned in-memory cursors and do not inherently require filesystem access. In
contrast, `file.info.inspect` requires a real regular-file handle and metadata,
`file.carve` currently has a filesystem destination, and
`investigation.profile.run` calls `std::time::Instant`. None is exposed through
the portable direct boundary. Browser File metadata must not fabricate native
filesystem facts or silently disable elapsed-time limits.

## Validate and run a linear recipe

```javascript
const recipe = {
  format_version: 1,
  engine_api: 1,
  input: { source: 'slot', name: 'message', type: 'text' },
  output: { type: 'text' },
  steps: [
    { operation: 'encoding.base64.decode', version: 1 },
    { operation: 'text.utf8.decode', version: 1 },
  ],
};
const plan = cube.validateRecipe(recipe);
const run = cube.runRecipe(recipe,
  { message: { type: 'text', data: 'Q2Fmw6k=' } }, { captures: [0] });
// run.output: { type: 'text', data: 'Café' }
// run.intermediates['0']: { type: 'bytes', data: Uint8Array [67,97,102,195,169] }
```

These methods call Core's strict Recipe and Plan APIs. Whole-plan validation
precedes execution; bytes/text conversion is explicit, and invalid UTF-8 fails.
`recipeJson()` serializes a definition with the same exact integer/byte transport
for export; it does not imply validation or execution. The model is linear and
does not import or flatten Play graphs. Validation info
is redacted and contains no parameter secrets or input contents.

Pass objects with safe integer Numbers or explicit bigint values, or pass an
exact JSON document string. Unsafe integer Numbers and malformed Unicode scalar
strings are rejected before serialization. Core output uses decimal strings for
integers outside the safe Number range. Do not turn those strings back into
Number. Pipeline parameter types can be narrower than their direct operation
counterparts: rotation count, for example, uses signed i64 in the linear adapter,
while the direct Rust API accepts u64. A larger direct value is not silently
truncated to fit a pipeline.

Core pipelines preserve their existing budgets: 64 steps, one MiB logical value, 16 MiB work,
four MiB aggregate captures and 256 KiB parameters. JSON documents and bindings
each additionally have the existing 256 KiB, depth-16 and 4096-node transport
ceilings; Base64 overhead can therefore make the effective input limit smaller
than the logical value ceiling. Plans may lower budgets. Direct invocations have
one MiB aggregate logical input and output ceilings, a separate one-MiB image
sample ceiling, the interpreter's existing eight-MiB memory ceiling and one
million interpreter instructions. Their JSON transport is separately bounded to
four MiB, depth 32 and 65536 nodes. Parameters/defaults and operation-specific
conservative limits still apply. Wasm asset downloads are capped at 64 MiB. These are distinct
bounds, not an assertion of a process-wide memory ceiling.

`CubeError` preserves typed Core codes and safe operation/parameter/step context.
Caller text and keys must not be added to diagnostic messages. Core executor
failure reasons are retained as safe `reason` values; an `operation_failed`
reason does not establish a specific input, budget or partial-completion outcome.
Cancellation of
synchronous Wasm is not provided by these methods; an AbortSignal cannot
interrupt a running call. Dedicated-worker lifecycle and cancellation outcomes
use the dedicated worker API instead.

## Use a dedicated browser worker

```javascript
import { createWorkerCube } from './dist/worker-client.mjs';
const worker = await createWorkerCube();
const result = await worker.execute({
  operation: 'encoding.base64.decode', version: 1, inputs: { input: 'AP8=' },
}, { context: { inputRevision: 7 } });
// result.value.output is exact Uint8Array data.
// result.requestId, generation and context identify this frozen invocation.
worker.terminate();
```

Worker and Wasm asset URLs are configurable explicit URLs. Each client permits
one pending invocation; it copies caller inputs/context and never transfers or
detaches original buffers. Older generations and unmatched reply IDs are ignored.
Invocation and initialization deadlines default to 30 seconds and may be lowered
or raised through 120 seconds. Requests also have a conservative two-MiB copy
budget, 8192-node and depth-20 ceilings before cloning.

An already-aborted signal reports `not_started`. Aborting after dispatch
terminates the dedicated worker and reports `worker_terminated`; a deadline
reports `worker_timeout`. Neither confirms that Core observed a cancellation
flag or that an unobserved computation did or did not finish. A reported unexpected runtime failure also discards the worker realm. Subsequent
calls initialize a new generation. Preserve immutable input/configuration revision
identities in application state and reject stale results there as well.

## Observed qualification and remaining gates

Local qualification exercised Node.js 22.14.0 and 24.12.0, headless Chrome
156.0.8078.74, Firefox 153.0 and Playwright WebKit 26.5. Development and optimized
Wasm artifacts passed the complete discovered 19-executor and 53-adapter
known-answer matrices in Node. All three tested browser engines passed the
canonical wire/operation matrix, non-root asset loading, missing-asset retry,
worker termination/restart and caller-byte ownership checks. Installed tarball
consumers passed in Node 22 and 24, including package exports, declarations,
Unicode recipes and exact binary data. TypeScript 7.0.2 checks positive and
deliberately invalid usage fixtures. The WebKit engine check does not qualify
the native Safari application. Untested runtime versions, other operating
systems and a delivered production release remain unqualified.

The proposed JavaScript workflow builds a release artifact, tests the same
tarball on Linux/macOS/Windows with Node 22/24, and runs Chromium/Firefox/WebKit
qualification. It contains no npm publication command. These remote jobs have
not run for the local changes. Required remaining delivery gates include those
CI matrices, accessibility/real-device review, a reviewed clean source revision
and validated documentation adoption. The complete responsive portal remains in the canonical
`cubearc/docs` backlog. Publication and deployment require separate authorization.
