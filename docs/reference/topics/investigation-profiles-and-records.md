# Investigation profiles and records

Investigations suggest inspections and record reproducible checks. Suggestions
cite evidence and available operations; they are not automatically executed and
do not establish concealed content.

## Triage and profile depth

Triage reads up to 12 header bytes and reports stable rules, evidence and offsets.

| Header | Suggested inspection |
| --- | --- |
| PNG | Chunks |
| JPEG or TIFF | EXIF |
| WAVE | Stream metadata |
| Unknown | Hex and strings |

Profiles select how much to inspect:

| Setting | Behavior |
| --- | --- |
| `depth: "metadata"` (default) | Triage only |
| `depth: "preview"` | Triage, 256-byte hex view, and strings from the first 4,096 bytes; minimum length 4, at most 32 results |
| `identify: true` | Hash the complete input; otherwise only an admitted prefix is buffered |

Save this example as `preview.json` for a reusable profile:

```json
{
  "format_version": 1,
  "depth": "preview",
  "identify": true,
  "budgets": {"max_operations": 4, "max_elapsed_ms": 5000}
}
```

Only `format_version` is required. Identity hashing is off by default.

## Profile budgets

| Field | Default | Hard ceiling |
| --- | --- | --- |
| `max_input_bytes` | 1 MiB source buffering | Default; can only be lowered |
| `max_inspected_bytes` | 4 MiB charged ranges | Default; can only be lowered |
| `max_operations` | 16 units | 64 units |
| `max_retained_bytes` | 64 KiB results | Default; can only be lowered |
| `max_elapsed_ms` | 5,000 ms | 60,000 ms |

Repeated ranges are charged again. A pipeline wrapper and each step consume
operation units; pipeline value and work limits also apply.

`plan` shows the allowlist, ranges, budgets, capture bounds and skipped reasons
without reading content. Deadlines and cancellation are checked around reads
and operations, without preemption. Elapsed time is diagnostic and excluded
from deterministic records.

Check partial, skipped and failed outcomes. Budget exhaustion does not mean an
investigation is complete.

### Optional pipeline and retention

| Field | Purpose |
| --- | --- |
| `pipeline` | Steps and lowered limits from the [composition contract](compose-operations.md) |
| `retain_results:[2]` | Retain selected in-memory results by session-local run ID |
| `record_parameters:true` | Opt into pipeline parameter retention |

Captures use conservative bounds; a small actual result can still be skipped.

## Session records

Records use `schema_version:1` with `status`, `replay_gaps`, `uncertainty` and
`session`. The session includes format/engine versions, input identity, limits,
runs, findings, separately labeled hypotheses and optional profile context/counters.

Findings trace to an input, run and offset. Raw input, paths and timestamps are
not saved. Default export also omits result contents, nonempty parameters and
hypotheses. Retention and export are separate explicit choices.

### Record limits

| Resource | Maximum |
| --- | --- |
| Record | 128 KiB |
| Retained result JSON | 64 KiB |
| Runs | 64 |
| Findings | 128 |
| Hypotheses | 32, each at most 1,024 UTF-8 bytes |
| Direct session/replay input | 1 MiB |

Loading validates IDs, references, states and retained-result digests. Human
reports escape untrusted strings.

Saving requires a caller-selected new file. There is no hidden session database;
delete records with normal filesystem tools. Omitted or redacted data cannot be
recovered from a saved record.

## Replay

Replay performs these checks in order:

1. Validate contracts and required parameters.
2. Verify caller-supplied input size and SHA-256.
3. Reproduce runs and compare typed result digests and states.

Size alone cannot establish identity. Missing digests, redactions, incompatible
versions and changed input produce explicit failures. Skipped runs remain skipped;
hypotheses are not replayed as machine evidence.

Digests identify content, not trust. Reports are not signed attestations.
See [Record and replay an investigation](../../guides/investigations.md) for a
walkthrough, or [InvestigationSession in the SDK](../../interfaces/python.md#discover-operations).
