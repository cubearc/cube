# Bounded Brainfuck

The direct Rust, CLI and native Python interfaces share one strict interpreter.
It executes only `> < + - . , [ ]` and accepts ASCII space, tab, CR and LF as
separators. Other characters, including prose and other Unicode whitespace, fail
at their original UTF-8 byte offset. Every bracket is validated before execution,
including unreachable code. Parsing is linear, iterative and stores command
positions and paired jumps without retaining another source copy.

Cells are unsigned wrapping integers: 8 bits by default, explicitly 16 or 32 bits.
The tape always has 30,000 initially zero cells; the pointer starts at zero.
Movement outside 0..29999 fails without wrapping, growth or changing the pointer.
`,` consumes one explicit caller byte and zero-extends it. EOF defaults to an
input-exhaustion error; `zero` and `unchanged` require explicit selection. `.` emits
exactly the low eight bits, so `-.` emits ff at every width while diagnostics show
255, 65535 or 4294967295. No wider word I/O is inferred.

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

MiB means 1,048,576 bytes. Limits never grow implicitly. The Core preflight
conservatively charges source/input, actual instruction-vector capacity, parser
stack, the fixed 120,000-byte u32 tape, output capacity, trace capacity, selected
tape records and compact JSON serialization. The report bound is 4,096 bytes of
framing/headroom plus four bytes per permitted output byte, 48 per selected cell
and 320 per permitted trace record. A small actual output does not override a
failed conservative preflight; lower the output budget to reduce the bound.
Host CLI/CPython object and allocator overhead is outside the Core allocation
model. Both transports stay bounded by the same cardinality/report ceilings.

Trace capacity is the smaller of the record limit and byte budget divided by
Rust's `size_of::<TraceRecord>()`; ABI padding is therefore charged explicitly.
Both trace limits must be zero or both positive. A byte budget smaller than one
record is valid and produces an explicitly incomplete trace on the first dispatch.
Trace exhaustion continues execution and sets `trace_complete=false`; it never
silently discards evidence. When enabled, records are taken **after** each attempted
dispatch, including a failing command, with a one-based instruction number,
original source offset/command, before/after pointer and cell, emitted byte and
input-consumption flag. `cell_after` belongs to `pointer_after`. Disabled traces
allocate no records. `completed` qualifies the execution separately from trace
completeness.

Reports retain exact partial bytes, effective options/limits, stop reason, safe
error context, instruction/input counts, final pointer and explicitly selected
tape indices/values. Syntax/parameter/preflight errors occur before a report
exists. Successful byte APIs never return partial output. CLI JSON byte arrays and
Python `bytes` preserve all values. CLI report/comparison commands exit nonzero
when any run is incomplete; Python report/comparison callers inspect the explicit
completion flags. Budget stops identify `per_run`, `aggregate` or `both` scope.

Comparisons share source/parser and use independent tapes with the same requested
policies and per-run budgets. `per_run_limits` records those requests; each run
shows effective budgets lowered by aggregate dispatch/output remainders. Retained
outputs are reduced to their exact length. Conservative shared memory includes
one live tape, retained outputs/reports and their compact JSON bound. Exhausted
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

The strict profile and dispatch/EOF defaults follow the recorded design in
[#121](https://github.com/cubearc/cube/issues/121); implementation is separately
scoped by [#139](https://github.com/cubearc/cube/issues/139) and
[#140](https://github.com/cubearc/cube/issues/140). Existing research history is
preserved. Wrapping 8-bit execution remains the default when diagnostics or wider
cells are selected. Nonwrapping/checked overflow is deferred: the accepted use case
requires exact modular widths and supplies no nonwrapping compatibility fixture;
adding another policy without such a consumer would create an unverified language
profile. Unknown overflow/options fields fail. Ook notation, arbitrary widths,
arbitrary precision, JIT, host filesystem/network/process effects, implicit stdin,
interactive debugging and interpreter pipeline adapters are outside this profile.
No fetched or historical SHA-256 program is used as a known-answer fixture.
