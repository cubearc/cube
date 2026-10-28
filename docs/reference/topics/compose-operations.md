# Compose operations

Pipelines run a linear sequence of operations over typed values. Each step names
an operation, its exact contract `version` and optional parameters. See
[Build a pipeline](../../guides/pipelines.md) for a walkthrough.

## Adapter discovery

An operation can be called directly without supporting pipelines. Check
`pipeline` in `cube ops list --json` or `cube ops describe`: an adapter version
object means supported; `null` means direct-only. `cube pipeline adapters` lists
the executable subset.

Adapters include bounded encodings, classical ciphers, hashes, checksums, XOR,
UTF-8 conversions and selected file inspections. Each adapter declares its own
input and output types. Image and audio operations require direct calls.

All catalog operations have direct Rust, CLI and native Python entry points.
The descriptor's `command` identifies the CLI route; Python stubs describe SDK
entry points. Orchestration such as recipe execution and investigation sessions
cannot itself be nested as an adapter.

## Values and type checking

Adjacent step types must match. Convert UTF-8 explicitly when a text result must
become bytes; the engine does not coerce values or scalar parameters.

| Value | JSON representation | Use |
| --- | --- | --- |
| Bytes | `{"type":"bytes","base64":"AP8="}` | Exact byte input |
| Text | `{"type":"text","data":"Hello"}` | UTF-8 text |
| Byte range | `{"type":"byte_range","offset":3,"length":4}` | Inert coordinates; identity only |
| Forensic report | Concrete signatures, hex or strings result | Inspection output; identity accepts report input |

Range length can be omitted or `null`. Offsets are checked for overflow; a range
grants no file access.

The entire plan validates before transformation. Static errors in later steps
prevent earlier steps from running. Runtime errors stop at the first failing
step and identify its zero-based index. An empty pipeline is identity.

### Parameters

| Parameter kind | Example | Rules |
| --- | --- | --- |
| Binary | `{"key":{"base64":"/w=="}}` | Canonical Base64 |
| Unsigned list | `{"unsigned_list":[0,2]}` | Full `u64` transport; checked conversion to platform positions |
| Text list | `{"text_list":["1=X"]}` | Exact UTF-8 strings |
| Scalar integer | A signed integer value | Signed `i64` transport |

List tags distinguish empty lists. Bare arrays, mixed or nested lists, floats,
`null`, negative unsigned values, unknown or duplicate fields and overflow fail.
Descriptor constraints check list lengths and element values.

## File inspection reports

These adapters inspect caller-supplied bytes without acquiring file access:

| Adapter | Report |
| --- | --- |
| `file.signature.detect` | Signature evidence and offsets |
| `file.hex.inspect` | Hex rows with exact bytes |
| `file.strings.extract` | Printable strings and completeness |

Hex and strings use checked offsets and lengths. Omitted length means through
the buffer's end, unlike the CLI file-hex preview.

Reports preserve numeric offsets, evidence, completeness and bytes. JSON hex rows
encode `bytes` as canonical Base64; Python uses byte objects. They are observations,
not proof of format validity.

Only identity accepts report input. Compatible byte transforms can precede an
inspection, and explicit captures can retain results. Reports require JSON output;
raw output is rejected. Record overhead counts toward conservative budgets.

## Limits

Limits can be lowered in the `limits` object; they cannot exceed the hard bounds.

| Resource | Hard bound | Setting |
| --- | --- | --- |
| Steps | 64 | `max_steps` |
| Each value | 1 MiB | `max_value_bytes` |
| Aggregate parameter data | 256 KiB | Fixed |
| Conservative work | 16 MiB | `max_work_bytes` |
| Shared captures and masked audits | 4 MiB | `max_capture_bytes` |
| Parameter bindings per step | 8 | Fixed |
| Entries per list | 65,536 | Fixed |
| Base58 adapter input per step | 4,096 bytes | Fixed |

Work includes initial input and each step's input/output upper bounds. A byte
range counts as 16 logical bytes. Captures are explicit, unique zero-based step
indexes. Conservative estimates can reject a request whose actual result would
be smaller.

### Parameter and transport accounting

| Resource | Accounting or limit |
| --- | --- |
| Unsigned-list storage | 16 header bytes + 8 per entry |
| Text-list storage | 16 header bytes + 24 per entry + exact UTF-8 bytes |
| JSON document | 256 KiB |
| JSON depth | 16 |
| JSON value nodes | 4,096 |

JSON bounds apply before decoding. Practical list sizes depend on the entire
document and can be smaller than the per-list hard limit.

Cancellation is cooperative between bounded steps, not a hard timeout. Pipelines
grant no shell, network, native loading or arbitrary file access.

## Masked transformation audits

`text.transform.masked` version 1 returns composable text plus a mandatory report
in `masked_audits[step]`. The audit contains original input/configuration, selected
positions, deterministic text, every explicit override and final text.

- Audits are retained even without captures, including in `recipe check`.
- Python uses integer step keys; JSON uses object keys.
- Executions without a masked step omit `masked_audits`.
- CLI pipeline and recipe runs with masked steps require `--format json`.
- Investigation sessions reject masked pipelines before execution because their
  final-value records cannot retain these audits.

### Masked budget calculations

| Component | Conservative charge |
| --- | --- |
| Output | Input bytes + 3 bytes per possible replaced scalar |
| Audit | 512-byte header + three UTF-8 text fields + 8 bytes per selected/configured index + 48 bytes per override |
| Additional work | `32 × input bytes` + `16 × selector entries × (floor(log2(max(1, entries))) + 2)` + `64 × overrides` + audit bound |

Captures and all audits share `max_capture_bytes`. Estimates precede transformation;
repeated overrides and multibyte text can increase charges. Input-dependent
positions validate inside the step before its ciphers run.

A direct call may accept a request that exceeds these combined pipeline budgets.
See [Position-aware transformations](position-aware-transformations.md).

## Recipe files

Recipes save a pipeline with an explicit input and output contract. This example
round-trips bytes through hex:

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

### Recipe contract

| Field or rule | Requirement |
| --- | --- |
| Format | JSON, `format_version: 1`, `engine_api: 1` |
| Input | One symbolic slot or one tagged inline value |
| Slot name | 1–64 ASCII letters, digits, underscores or hyphens |
| Bindings | Exact match; missing or extra bindings fail; inline recipes accept none |
| Output | Explicit type |
| Steps | Ordered operations with exact adapter versions |
| Limits | Optional lowered pipeline budgets |

An inline input can be
`{"source":"inline","value":{"type":"bytes","base64":"AP8="}}`.
Bindings are supplied values, never implicit paths, stdin or environment reads.

Format, engine and adapter versions are independent of package versions.
Unsupported versions fail without automatic migration.

### Validation and compatibility

The shared JSON limits apply to recipes, including inline bytes and parameters.
Duplicate keys, unknown fields, noncanonical Base64 and invalid types fail.
Recipes have no includes, expressions or executable tags.

Validation does not execute algorithms. `recipe check` / `check_recipe` does
execute the recipe and compares exact typed output; a different expected value
must fail. Community definitions and examples live in
[cubearc/recipes](https://github.com/cubearc/recipes). For embedding pipelines, see
the [Python SDK](../../interfaces/python.md#compose-operations).
