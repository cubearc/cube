# Bounded Brainfuck

Run a strict Brainfuck program with explicit input bytes and resource budgets.
The direct Rust, CLI and native Python interfaces share the same behavior.

## Program syntax

The interpreter executes only `> < + - . , [ ]`. ASCII space, tab, CR and LF
are accepted separators. Other characters, including prose and other Unicode whitespace, fail
at their original UTF-8 byte offset. Every bracket is validated before execution,
including unreachable code. Unknown options and overflow-policy fields fail.

## Cells and byte I/O

Cells are unsigned wrapping integers: 8 bits by default, explicitly 16 or 32 bits.
The tape always has 30,000 initially zero cells; the pointer starts at zero.
Movement outside 0..29999 fails without wrapping, growth or changing the pointer.
`,` consumes one explicit caller byte and zero-extends it. EOF defaults to an
input-exhaustion error; `zero` and `unchanged` require explicit selection. `.` emits
exactly the low eight bits, so `-.` emits ff at every width while diagnostics show
255, 65535 or 4294967295. No wider word I/O is inferred.

## Loops and instruction counts

`[` on zero jumps after its paired `]`; otherwise it advances. `]` on nonzero
jumps after its paired `[`; otherwise it advances. Every dispatched command costs
one instruction, including loop tests, repeated work and failing pointer/I/O
operations. The budget is checked before dispatch. Finishing exactly at the limit
succeeds; needing another dispatch fails. Thus `[+]` costs one instruction,
`+[+]` costs 512 with 8-bit cells, and `+[]` cannot run without a finite limit.
No optimizer changes these counts. Output-limit failures do not append the failed
byte. Input failures do not consume a byte.

## Limits

| Resource | Default / hard ceiling |
| --- | --- |
| UTF-8 source / explicit input | 1 MiB each |
| Parsed commands / bracket depth | 65,536 / 256 |
| Dispatched instructions | 1,000,000; lower per-call values, including zero |
| Output | 1 MiB; lower per-call values, including zero |
| Combined Core memory bound | 8 MiB; lower per-call values |
| Final tape selection | Exact half-open range within 30,000 cells; empty by default |
| Trace | Disabled; at most 4,096 records and 1 MiB of in-memory records |
| Compact JSON report bound | 8 MiB; lower per-call values |
| Comparison | 1..3 unique supported widths; ascending order |
| Aggregate comparison dispatch / output / memory | 1,000,000 / 1 MiB / 8 MiB |

Caller budgets can lower the hard ceilings, including to zero where shown.
The [conservative memory and report reservation](resource-accounting.md#brainfuck)
can reject a request before execution. A smaller requested output allowance can
reduce that reservation. See [resource limits](concepts/resource-limits.md) for
units, scope and the difference between Core memory and process memory.

## Traces

Both trace limits must be zero or both positive. A byte budget smaller than one
record is valid and produces an explicitly incomplete trace on the first dispatch.
Trace exhaustion continues execution and sets `trace_complete=false`; it never
silently discards evidence. When enabled, records are taken **after** each attempted
dispatch, including a failing command, with a one-based instruction number,
original source offset/command, before/after pointer and cell, emitted byte and
input-consumption flag. `cell_after` belongs to `pointer_after`. Disabled traces
allocate no records. `completed` qualifies the execution separately from trace
completeness.

## Reports and completion

Reports retain exact partial bytes, effective options/limits, stop reason, safe
error context, instruction/input counts, final pointer and explicitly selected
tape indices/values. Syntax/parameter/preflight errors occur before a report
exists. Successful byte APIs never return partial output. CLI JSON byte arrays and
Python `bytes` preserve all values. CLI report/comparison commands exit nonzero
when any run is incomplete; Python report/comparison callers inspect the explicit
completion flags. Budget stops identify `per_run`, `aggregate` or `both` scope.

## Comparing cell widths

Comparisons use independent tapes with the same requested policies and per-run
budgets. `per_run_limits` records those requests; each run
shows effective budgets lowered by aggregate dispatch/output remainders. Exhausted
shared budgets may leave later runs with zero effective instruction/output budget.
A zero-budget empty program still completes. `equal_completed_outputs` is null
when any run is incomplete. Equal completed output bytes do not prove semantic
equivalence across cell models.

## Rust example

```rust
use cube_core::esolang::brainfuck::{self, Limits};
assert_eq!(brainfuck::execute("++++++++[>++++++++<-]>+.", &[], Limits::default())?, b"A");
# Ok::<(), cube_core::errors::OperationError>(())
```

`Program::parse` allows reuse across `execute` and `report`; `compare` parses once
for all selected widths. See the [CLI](guides/brainfuck.md#run-bounded-brainfuck) and
[Python](guides/brainfuck.md#bounded-brainfuck) guides for transport examples.

## Compatibility decisions

The supported profile has 8/16/32-bit wrapping cells, explicit input bytes and no
pipeline adapter or host effects. Other widths, nonwrapping overflow and implicit
stdin are unsupported. See the [profile and verification decisions](resource-accounting.md#profile-and-verification-decisions)
for contributor rationale and design history.
