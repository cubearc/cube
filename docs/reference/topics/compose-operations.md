### Compose operations

Pipelines are implemented linear sequences over explicit bytes, UTF-8 text and an
inert structured byte range. Direct Rust/Python functions keep their normal types.
`cube ops list --json`, `cube ops describe` and the Python discovery functions
report `pipeline`: an adapter contract version object when supported, or null
(`None` in Python) otherwise. `cube pipeline adapters` /
`cube.pipeline_adapters()` lists the same executable subset. Current adapters cover
bounded encodings, classical ciphers, hashes/checksums, XOR and UTF-8 conversion.
Selected in-memory file inspections also have adapters: `file.signature.detect`,
`file.hex.inspect` and `file.strings.extract`. They accept bytes and return a
`forensic_report` with a concrete `signatures`, `hex` or `strings` result. Adapter
listings declare their pipeline input/output types separately from the direct
reader API's descriptor. Hex and strings reuse checked offsets/lengths; omitted
length selects through the end of the buffer (unlike the CLI file hex preview).
Other file operations and image/audio APIs require direct calls.

Reports preserve numeric offsets, signature evidence, completeness and exact
bytes. JSON hex rows encode their `bytes` field as canonical base64; native Python
uses `bytes`. They are inert observations, not file-validation proofs. No transform
currently consumes reports; compatible byte transformations can precede them and
captures can retain any selected step. Report input supports identity; JSON inputs
retain the existing document-size limit. Use JSON output for reports; raw output
is rejected. Conservative budgets include record overhead and may reject a result
that would be smaller for particular data.

The current catalog's operations are available through direct Rust functions or
methods, CLI commands, and native Python functions or methods. The descriptor's
`command` identifies its CLI route; Python stubs describe SDK entry points.
This includes orchestration such as recipe execution and investigation sessions,
which is available directly but cannot itself be nested as a pipeline adapter.
Read-only file APIs still require caller-selected I/O; discovery grants no file
permission. Signature detection, hex rows and printable strings can also inspect caller-supplied
buffers in a pipeline without acquiring file access.

Every step declares an operation identifier, exact contract `version`, and optional
parameters. The entire plan validates before transformation. Adjacent types must
match: a hash's text output is not silently converted to bytes for the next step.
Use explicit UTF-8 conversion operations when that is intended. Errors stop at the
first failing step and identify its zero-based index. An empty pipeline is identity.

JSON values use explicit tags: `{"type":"bytes","base64":"AP8="}`,
`{"type":"text","data":"Hello"}`, or
`{"type":"byte_range","offset":3,"length":4}`. Binary parameters use
`{"key":{"base64":"/w=="}}`. Range length may be omitted/null, checked offsets
cannot overflow, and a range grants no file access. Only identity currently accepts
range values. Parameter scalar types are not silently coerced.

List parameters are concrete tagged objects: `{"unsigned_list":[0,2]}` or
`{"text_list":["1=X"]}`. Tags distinguish even empty lists (`{"unsigned_list":[]}`
versus `{"text_list":[]}`). Bare arrays, mixed/nested lists, floats, null, negative
unsigned values, unknown/duplicate fields and integer overflow fail. Unsigned
list transport preserves all u64 values and checks conversion to platform positions;
scalar parameters retain signed i64 transport. Descriptor constraints validate
lengths and element values. Tags add parameter types without changing engine API 1,
recipe format 1, any existing adapter version, or legacy execution JSON.

`text.transform.masked` version 1 stays text-to-text and always returns its full
direct-operation report in `masked_audits[step]` alongside the composable output.
This includes original input/configuration, selected positions, deterministic text,
every explicit override and final text. Audits are mandatory even without captures;
`recipe check` returns them too. Python uses integer step keys; JSON uses object
keys. Existing executions/checks omit the field when no masked step is present.
CLI pipeline/recipe runs require `--format json` for masked steps so an ordinary
or raw output mode cannot silently discard their evidence. Investigation sessions
currently record only a final typed value, so they reject masked pipelines before
execution; use the direct pipeline/recipe interfaces to retain these reports.

Hard pipeline bounds are 64 steps, 1 MiB per value, 256 KiB parameter data,
16 MiB conservative work and 4 MiB shared captured-intermediate/masked-audit retention. Lower these with
`max_steps`, `max_value_bytes`, `max_work_bytes` and `max_capture_bytes` in the
limits object. Work counts initial input and each step's input/output upper bounds; ranges count
as 16 logical bytes. Conservative bounds can reject a data-specific smaller result.
Base58 adapters additionally cap step input at 4096 bytes. Captures are explicit,
unique zero-based step indexes. Cancellation is cooperative between bounded steps,
not a hard timeout. No shell, network, native loading or arbitrary file capability
is granted. See [CLI](../../guides/pipelines.md#run-a-pipeline-or-recipe) and
[Python](../../guides/pipelines.md#compose-a-pipeline-or-recipe) examples.

List storage counts toward the 256 KiB aggregate parameter budget: unsigned lists
charge 16 header bytes plus 8 per entry; text lists charge 16 plus 24 per entry and
exact UTF-8 bytes. Each list also has a hard 65,536-entry limit. The unchanged
256 KiB/depth-16/4096-node JSON bounds apply before decoding, so practical list
limits are smaller and depend on the entire document. The eight-bindings-per-step
limit remains; mutually exclusive selectors allow even a full masked request.

Masked output is bounded by input bytes plus three bytes per possible replaced
scalar. Each audit charges a 512-byte header, its three UTF-8 text fields, 8 bytes
per selected/configured index and 48 bytes per override. Captures and all audits
share `max_capture_bytes`. Conservative bounds may charge more than the actual
result, especially with repeated overrides or multibyte input. Masked work also
charges 32 times input bytes, 16 times selector entries times (floor(log2(max(1,
entries))) + 2), 64 per override and the audit bound, in addition to the normal
input/output work charges. All estimates precede transformation. Input-dependent
positions validate inside the step before its ciphers run; static later-step errors
reject the whole plan before earlier steps execute. These bounds can reject large
requests that a direct single-operation call accepts.

### Recipe files

Recipes make a validated pipeline reusable. Save this original round-trip example
as `roundtrip.json` in your working directory:

```json
{
  "format_version": 1,
  "engine_api": 1,
  "input": {"source": "slot", "name": "payload", "type": "bytes"},
  "output": {"type": "bytes"},
  "steps": [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1}
  ]
}
```

Format 1 uses JSON, one symbolic input slot or one tagged inline input, an explicit
output type and ordered steps. Slots use 1..64 ASCII letters/digits/underscores/
hyphens. Missing/extra bindings are errors; inline recipes accept none. An inline
input instead looks like `{"source":"inline","value":{"type":"bytes","base64":"AP8="}}`.
Bindings are caller-supplied values, never implicit paths, stdin or environment reads.

Format version, engine API and each adapter version are exact compatibility
contracts, independent of package versions. Matching operation identifiers alone
is insufficient. Unsupported versions are rejected; there is no automatic migration.
Optional `limits` uses the pipeline's lowered budgets.

Recipe/JSON transport is limited to 256 KiB, depth 16 and 4096 value nodes. Inline
bytes and parameters count toward the document limit. Duplicate keys, unknown
fields, noncanonical base64 and invalid types fail; there are no includes,
expressions, executable tags or privileged capabilities. Validation never executes
algorithms. `recipe check` / `check_recipe` compares exact typed output so community
recipes can test compatibility independently. A different expected value must fail.
The engine owns contracts; community definitions/examples live in
[cubearc/recipes](https://github.com/cubearc/recipes), without duplicated algorithms.
