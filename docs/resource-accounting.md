# Resource accounting for contributors

These notes describe storage reservations and implementation-specific accounting.
Readers choosing inputs or interpreting budget failures should start with
[resource limits](concepts/resource-limits.md). Exact operation ceilings remain
in each family reference and its canonical Core implementation.

## Scalar multisets

Core checks memory and compact-report bounds before preparation. The combined
bound includes original input bytes, prepared scalar vectors with capacity growth,
both counting trees, retained inventory/residual/deficit records and a JSON buffer.
At most one temporary normalized text buffer, capped at 8 MiB, is live at a time.
The report buffer is included in both the combined-memory and report constraints.

Work includes raw bytes, all normalized scalar visits before filtering, retained
scalar counts and conservative counting-tree/merge visits. The shared preparation
and counting functions belong to `text::statistics`; multiset consumption remains
in `text::multiset`.

## Boolean formulas

Report reservation uses actual ID escaping and retained-field sizes. A full
assignment listing with long IDs can fail its memory/report bound before reaching
its row limit. Raw JSON parsing applies duplicate-key, node and depth checks before
allocating the typed AST. Every formula node is evaluated without short circuit,
so work is combined nodes times assignments for analysis, or combined nodes for
one evaluation.

## Finite graphs

Typed bounded decoding avoids retaining a second untyped object tree. Combined
allocation includes lookup tables, adjacency, traversal state and reports, using
actual ID encoding sizes. Native object transport is separately prewalked to
bound cardinality, bytes and depth before JSON encoding; host-runtime object
and allocator overhead is outside the Core allocation model.

Construction charges vertices plus four visits per edge. Queries add their
vertex/edge visits and conservative component-ordering comparisons. The shared
unweighted BFS owns shortest paths. Cycle witnessing uses internal DFS;
Euler trails use Hierholzer. These helpers do not expose a general traversal API.

## Brainfuck

Parsing is linear and iterative. It records original command positions and paired
jumps without keeping another source copy. Core preflight includes source/input,
actual instruction-vector capacity, parser stack, a fixed 120,000-byte `u32` tape,
output capacity, trace capacity, selected tape records and compact JSON storage.

The report bound reserves 4,096 bytes of framing/headroom, then four bytes per
permitted output byte, 48 per selected cell and 320 per permitted trace record.
CLI/CPython object and allocator overhead is outside this Core model; both
transports retain the same cardinality/report ceilings.

The ledger reserves 12 bytes per parsed instruction, eight bytes per parser
index, 48 bytes per trace record and 16 bytes per selected cell. These charges
cover the supported 32/64-bit layouts, including padding; compile-time checks
reject a layout that would exceed them. They keep admission and reported upper
bounds consistent across native and Wasm runtimes and do not claim measured
allocator usage. Trace capacity is the smaller of the requested record count
and the trace byte budget divided by the 48-byte charge.
Comparison shares one parsed program and at most one live tape, while retaining
prior outputs/reports and their JSON bound. Retained outputs shrink to exact length.

### Profile and verification decisions

The strict grammar, modular cell widths and dispatch/EOF defaults are recorded in
[#121](https://github.com/cubearc/cube/issues/121),
[#139](https://github.com/cubearc/cube/issues/139) and
[#140](https://github.com/cubearc/cube/issues/140). Nonwrapping/checked overflow
has no accepted compatibility fixture in this profile. There is no arbitrary
precision, JIT, Ook notation, interactive debugger or host filesystem/network/process
capability. Known-answer fixtures do not depend on a fetched historical SHA-256
program. User-visible grammar, supported widths and completion policies remain
in the [Brainfuck reference](brainfuck.md).

## Word ladders

The wildcard index includes the removed scalar position and all remaining
scalars. It generates substitution edges in deterministic order, then delegates
to shared graph validation and BFS. Numeric internal vertex IDs support words of
up to 32 four-byte scalars without changing the graph's public ID contract.
Lexical state, both adjacency directions and graph query storage count together.

## Historical cipher verification

Algorithms and validation live in Core; interfaces adapt the same functions.
Conformance fixtures compare independently checked known answers. Operation
metadata retains the exact primary-source URLs linked from the
[historical profile reference](ciphers.md#primary-sources-and-reproducibility).

HELP → HIAT is independently calculated for matrix `[3,3,2,5]` under the declared
Hill convention. Short Trithemius and Running Key fixtures check explicit formulas;
they are not attributed to an ACA sheet. Phillips RC uses the detailed geometry
where its published combined final line differs. No historical book text or
third-party cipher implementation is bundled.

For Seriated Playfair, the forward scan and bounded binary search inspect at most
4,096 tail positions; preparation does not repeatedly insert into the whole message.
The keyboard `Geometry::preset` helper extends the US ANSI QWERTY/Dvorak/QWERTZ
presets without changing legacy `Layout` JSON or remapping behavior.

## Scalar traversal

Source arrays reserve eight bytes per scalar. Josephus additionally reserves
four bytes per Fenwick-tree cell. Extraction releases those arrays before reusing
the text-extraction routine. The scratch cap excludes caller-owned input.

## Transposition and fractionation verification

Independent published vectors cover AMSCO, Bifid/Trifid, Bazeries, Fractionated
Morse, Morbit, Turning Grille and Complete Columnar. Hand-derived permutation and
ADFG* vectors exercise the explicit geometry. Core conformance fixtures, CLI,
fresh native Python and Play tests share these values. Profile-specific source
qualifications, including Nihilist's ordinary sums rather than modulo 100, remain
in the user reference.
