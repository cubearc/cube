# Finite-domain profile evaluation

Decision recorded on 2026-10-26 for [#145](https://github.com/cubearc/cube/issues/145).
This is research evidence, not an available solver, new operation ID or pipeline
adapter. The [standalone original probe](finite-domain-probe.rs) is not part of a
Cargo crate, the CLI, Python extension or shipped runtime.

## Decision

A small, closed integer finite-domain profile is feasible and merits a focused
ordinary-Rust implementation after its contract is accepted. Reuse Cube's typed
errors, strict bounded JSON transport and report conventions; do not reuse the
Boolean evaluator as a CSP engine. Neither inspected external library is qualified
for Cube's exact support-scan, retention and explanation contract. No dependency
or production solver is added by this research. Generalized AllDifferent,
optimization, arbitrary predicates and heuristic framework integration are deferred.

This decision does not alter the separate ownership of [exact cover #343](https://github.com/cubearc/cube/issues/343)
or [Latin/Sudoku validation #356](https://github.com/cubearc/cube/issues/356).
Candidate engines, natural-language and modal/epistemic reasoning remain excluded.

## Proposed concrete contract

Input is one versioned JSON model with ordered variables `{id, domain}` and ordered
constraints `{id, relation, ...}`. IDs are unique nonempty UTF-8 strings of at most
32 bytes. Domains contain 0..32 distinct signed i32 values in explicit input order;
the initial implementation canonicalizes enumeration by numeric ascending value,
reporting that effective order. At most 32 variables and 256 input constraints are
accepted. An empty individual domain proves unsatisfiability after validation;
a model with no variables/constraints has one empty assignment.

The relation set is closed: `fixed`, `equal`, `not_equal`, `all_different`, exact
signed `offset` and `adjacent`, plus explicit binary allowed-pair tables from the
inventory supplement. Offset/adjacency arithmetic widens to i64 before subtraction
or absolute value; it cannot overflow i32. Fixed values outside the domain create
a contradiction, rather than extending the domain. AllDifferent initially lowers
to pairwise inequality with no claim of generalized arc consistency. Binary
constraints with identical endpoint IDs must either be rejected explicitly or
lowered to their unary diagonal before AC-3; the probe deliberately rejects
aliased binary endpoints and does not establish such a lowering.

Unknown relation/variable IDs, duplicate IDs/domain values/scope entries/table
pairs, malformed arities, out-of-range integers and bool/float coercions are input
or parameter errors. There are no expression strings, callbacks or host effects.
Strict typed array decoding, as used by the graph API, avoids allocating a large
untyped JSON tree before checking cardinality. A 1 MiB raw input ceiling applies
before parsing; transport and model validation remain distinct from search.

For each input, compile binary relations into deterministic ordered arc tables.
At most 512 compiled unordered relation tables are allowed, each at most 32 rows
of u32 support bits (64 KiB total). This explicit compilation cap is in addition
to the 256 input-constraint cap. At most 496 distinct variable pairs exist at the
32-variable limit; constraints on one pair can be intersected, preserving their
origin IDs in a bounded side table. Charge table construction/intersection and
all input scans as well as later search; a large expansion may exhaust work
before execution, never silently weaken a relation.

Use FIFO AC-3 with a membership flag per ordered arc. A value is removed only
when the current neighboring domain has no support in that arc's table; incoming
arcs are reconsidered after a domain shrinks. Stable arc consistency is not a
satisfiability proof. Backtracking selects the first nonsingleton variable in
input order and tries values in ascending order, on independent domain snapshots.
The first 100 solutions can be listed. One further solution establishes explicit
solution-cap truncation; budget exhaustion before that proof stays `work_exhausted`.

A result distinguishes completed enumeration, proven unsatisfiable, solution-cap
truncation, work exhaustion and cancellation. Only complete enumeration with zero
solutions proves unsatisfiability. Partial solutions remain explicitly incomplete.
A proposed API is `enumerate(model, limits, cancellation) -> FiniteDomainReport`;
CLI would use explicit inert model JSON and Python would accept the same bounded
model as builtin data or raw JSON. Neither interface exists for this profile yet.
Direct structured discovery is appropriate only when implemented; no new pipeline
value or adapter is justified by this evaluation.

## Reasons, budgets and retention

Deductions identify the compiled relation/origin constraint IDs, affected variable
and removed value. They are conditional on the current branch/domain snapshot.
Decision and backtrack records are separate kinds; a hypothetical branch removal
must not be presented as a global logical deduction. The probe records numeric
constraint/variable indices, which refer to ordered input metadata, not copied
unbounded descriptions. Human-quality proof narratives are not promised.

The proposed execution ceiling is 1,000,000 charged operations across compilation,
support checks, queue scans/enqueues, domain copies, decisions and search nodes.
Cancellation is checked at those same boundaries. Every attempted support check
and requeue consumes work; failed/duplicate work is not free. Lower caller budgets
must be accepted. The probe charges queue initialization/pops, support checks,
removals, incoming-arc scans/enqueues, branch copies and search visits. Its fixtures
compile fixed tiny tables before the measured search; production compilation work
must additionally be charged, so the recorded counts are search/propagation counts.

Retain at most 10,000 trace records and an explicit trace-byte cap. Trace exhaustion
continues bounded search with `trace_complete=false`; it cannot discard evidence
silently. Proposed numeric trace fields fit a conservative 128-byte compact JSON
bound per record. Input ID tables appear once. Solutions need at most
100 × 32 integer bindings; initial/final domain listings need at most 32 × 32
values. Derive the complete report bound from these cardinalities before running,
with a hard 8 MiB ceiling and no implicit increase of smaller caller caps.

The overall memory ceiling remains 32 MiB. A compact domain is a u32 mask over
an explicit i32 value table. At most 33 × 32 × 4 = 4,224 bytes hold branch masks;
queues hold at most 1,024 ordered arc indices and membership flags. The 512-table
support arena is <=65,536 bytes. Count source/parsed model, origin-ID tables,
retained solutions, actual trace-record capacity and the serialized report buffer
together. The probe's concrete-layout bounds below include an entire 8 MiB report
reserve and one AC-3 queue; they are allocation bounds, not measured process RSS.
CPython/allocator baseline overhead is not established by this standalone test.

## Independently verified fixtures and observations

The probe contains original fixtures, not fetched puzzle programs or diagrams.
Six assignment variables encode colors and drinks at three positions. Each
category has AllDifferent; red is at 0, water at 2, tea equals red, coffee equals
green, and green is one position after/adjacent to red. The unique assignment is
`[0,1,2,0,1,2]` in red/green/blue/tea/coffee/water order.

A 3×3 Latin model uses nine cell variables and pairwise row/column inequalities.
It has 12 solutions. Fixing the first row and column to 0,1,2 leaves the unique
row-major square `[0,1,2,1,2,0,2,0,1]`. The reduced 4×4 model has four solutions.
An independent full-assignment brute-force enumerator checks all these sets,
including enumeration order, without propagation or pruning.

For orthogonality, fix A(r,c)=(r+c) mod 3 and B(r,c)=(r+2c) mod 3. Nine auxiliary
pair variables P have domain 0..8 with binary tables floor(P/3)=A and P mod 3=B;
AllDifferent on P expresses distinct ordered pairs. The same model has 27 variables
and 90 binary relations, with one solution. Replacing B with A yields zero
solutions because ordered pairs repeat. The pair tables are the supplement's
explicit binary relations, not a separate Latin or exact-cover engine.

Observed with local Rustup stable 1.99.1, unoptimized standalone build:

| Fixture | Variables / binary relations | Solutions | Charged search work | Trace records | Allocation bound |
| --- | --- | --- | --- | --- | --- |
| Assignment grid | 6 / 10 | 1 | 276 | 8 | 8,458,112 bytes |
| Latin 3×3 | 9 / 18 | 12 | 7,990 | 144 | 8,459,776 bytes |
| Reduced Latin 3×3 | 9 / 18 | 1 | 458 | 8 | 8,459,776 bytes |
| Reduced Latin 4×4 | 16 / 48 | 4 | 7,182 | 57 | 8,466,016 bytes |
| Orthogonal 3×3 pair | 27 / 90 | 1 | 3,970 | 72 | 8,474,752 bytes |
| Repeated/nonorthogonal pair | 27 / 90 | 0 | 3,827 | 73 | 8,474,752 bytes |

Additional observed checks cover all 512 binary relations on two 3-value domains,
comparing exact solutions and sound pruning against brute force. A three-variable,
two-value inequality triangle is arc-consistent but unsatisfiable; complete search
and independent brute force agree. Every fixture succeeds exactly at its measured
work count and exhausts at one less; a one-record trace cap preserves solutions
and declares truncation. Solution-cap lookahead, deterministic cancellation after
ten charged operations, an empty domain and the 32-variable/32-value envelope
also pass. Reported microsecond timings are local observations only; no benchmark,
platform or worst-case latency guarantee follows.

Reproduce from the repository root, using the stable Rustup toolchain:

```sh
rustfmt --check --edition 2024 docs/research/finite-domain-probe.rs
rustc --edition=2024 -D warnings docs/research/finite-domain-probe.rs -o /tmp/cube-fd-probe
/tmp/cube-fd-probe
```

The probe validates only typed internal fixture models. A production JSON parser,
public error surface, native bindings, merged origin reasons, real concurrent
cancellation and the proposed compilation caps are not implemented or claimed to
have passed integration tests here. Their schemas/bounds are the proposed contract
above, informed by the observed finite-model behavior.

## Dependency and maintenance assessment

The [AIMA chapter's AC-3/backtracking discussion](https://aima.cs.berkeley.edu/newchap05.pdf)
provides the primary algorithm reference (sections 5.2, particularly arc consistency).
The proposed closed profile and all budget numbers are Cube design decisions,
not requirements imposed by that reference.

| Candidate | Evidence inspected | Decision for this profile |
| --- | --- | --- |
| Ordinary Rust subset | Standalone probe, no dependencies, bounded tables/queue/domains; exact finite fixtures and 512 relation oracles | Recommend a focused implementation. Cube must own validation, all charged work and conditional reason records. No generic solver framework. |
| `csp-solver` source 0.7.0 | [Pinned manifest](https://github.com/mkbabb/csp-solver/blob/471aaa4caee5225a62a5efa8d8621573e62bd010/csp-solver/Cargo.toml), [workspace/MSRV](https://github.com/mkbabb/csp-solver/blob/471aaa4caee5225a62a5efa8d8621573e62bd010/Cargo.toml), [MIT license](https://github.com/mkbabb/csp-solver/blob/471aaa4caee5225a62a5efa8d8621573e62bd010/LICENSE); last source commit observed 2026-10-19 | Active Rust/AC-3 candidate, Rust 1.88/edition 2024. Default dependencies include `include_dir` and `smallvec`; puzzle templates are embedded. Optional PyO3 and broad constraint/search/GAC surfaces add packaging/maintenance work. This review does not establish Cube's support-scan/reason guarantees; defer adoption rather than infer them from a node budget. Source version is not a claim about the latest published crate. |
| `selen` 0.15.5 | [Pinned manifest](https://github.com/radevgit/selen/blob/40f0cc86d97d5a7fbe13c0aee0e22075e4b7a859/Cargo.toml), [README/API](https://github.com/radevgit/selen/blob/40f0cc86d97d5a7fbe13c0aee0e22075e4b7a859/README.md), [MIT license](https://github.com/radevgit/selen/blob/40f0cc86d97d5a7fbe13c0aee0e22075e4b7a859/LICENSE); last source commit observed 2026-01-04 | Rust 1.88/edition 2024 with no external dependencies; supports a much broader int/float/bool/optimization model. Its public overview does not qualify the exact finite enumeration/work/reason contract required here. Defer integration; zero dependencies alone is insufficient qualification. |

Both licenses would require preserving their notices if code were redistributed;
no external solver code or assets are imported in this change. Library binaries,
advisory scans and Linux/Windows/native-extension compatibility for those solvers
were not tested, so no dependency safety or platform certification is asserted.
The small pure-Rust model demonstrates transport feasibility without changing
Cube's package dependencies, engine API 1 or recipe format 1.
