# Lexical and discrete foundations

Compare character inventories, count arrangements, evaluate Boolean formulas or
query a finite graph. These operations return structured reports through direct
Rust, CLI and native Python calls; they have no pipeline adapters.

See [complete and partial results](concepts/results.md) for result interpretation
and [resource limits](concepts/resource-limits.md) for budget terminology.

## Exact scalar multisets

`text.multiset.compare` and `text.multiset.subtract` return the same evidence:

| Field | Meaning |
| --- | --- |
| `source`, `requested` | Complete prepared inventories |
| `residual` | Source counts left after subtracting requested counts |
| `deficits` | Requested counts missing from the source |
| `contains` | Every requested count is available |
| `equal` | Both inventories have identical counts |

Each record has a `scalar` and exact `count`, ordered by Unicode code point.
Empty inputs and NUL are valid. Preparation follows the
[shared text profile](reference/text-preparation.md); defaults preserve every
scalar without normalization, case changes or filtering.

```sh
cube text multiset compare --text listen --requested silent
cube text multiset subtract --text ab --requested aab
```

The first report has `equal=true`. The second has `contains=false` and a deficit
of one `a`. Counts are always complete; a resource failure returns an error.

### Multiset limits

The shared profile defines the per-input preparation ceilings. The budgets below
cover the complete operation, including **both inputs**; defaults equal the hard
maxima.

| Resource | Maximum |
| --- | --- |
| Combined Core memory | 32 MiB |
| Compact JSON report | 8 MiB |
| Work | 16,777,216 units |

Work counts original bytes, every normalized scalar visited (including discarded
scalars), retained scalars counted, and conservative counting-tree/merge visits.

Use `max_work`, `max_memory_bytes` and `max_report_bytes` in `limits`, or CLI flags
`--max-work`, `--max-memory-bytes` and `--max-report-bytes`, to lower the budgets.

Memory and report bounds are checked before processing; see the
[allocation accounting](resource-accounting.md#scalar-multisets) for included storage.

## Exact combinatorial counts

| Operation | Result |
| --- | --- |
| `number.permutations` | Ordered selections, P(n,k) |
| `number.binomial` | Unordered selections, C(n,k) |
| `number.multinomial` | Arrangements for explicit group sizes |

Inputs are builtin unsigned integers: `n` is at most 128, `k` is at most `n`,
and there are at most 128 groups with a checked total of at most 128. Booleans,
floats, signed text, prefixes and coercions are rejected. Zero-sized and empty
groups are valid; an empty selection counts as one.

Results must fit `u128`. Overflow is a resource/range error. Binomial cancellation
happens before multiplication, so a fitting result does not fail because an
intermediate factorial would overflow. All C(128,k) fit; 52! does not.
These APIs return counts without generating arrangements or providing a general
factorial operation.

```sh
cube number permutations --n 5 --k 2
# 20
cube number multinomial --groups 2,1,1 --json
```

The multinomial count is 12. Rust's `Count.value` is `u128`; CLI count JSON uses
decimal strings and Python returns exact builtin integers.

### Count limits

| Resource | Maximum or bound |
| --- | --- |
| Arithmetic and cancellation work | 65,536 operations |
| Scratch memory | 64 KiB |
| Count report bound | 128 bytes reserved |
| Caller report cap | At most 4 KiB |

Count limits expose `max_work` and `max_report_bytes`. The scalar count APIs
retain no large scratch allocation.

### Pascal rows

Pascal takes a **row count**: zero returns no rows; 129 includes row indexes
0 through 128. The recurrence uses the same exact binomial coefficients.

```sh
cube number pascal --rows 4
```

The report's `rows` field contains `[[1], [1,1], [1,2,1], [1,3,3,1]]`, with
decimal strings in CLI JSON. Python returns the rows as exact integers.

| Resource | Maximum or bound |
| --- | --- |
| Rows | 129 |
| Coefficients | 8,385 |
| Work | 8,514 units: coefficients plus 129 row visits |
| Active row scratch | Less than 4 KiB; excludes retained triangle |
| Compact JSON output | 1 MiB |

The output reservation is `128 + 4*rows + 42*coefficients` bytes. Python Pascal
limits use `max_work` and `max_output_bytes`; CLI uses matching hyphenated flags.

## Tagged Boolean formulas

A model declares `variables` in caller order, one `formula` and optional `other`
for comparison. Both formulas use the same declared variable order.

| Tag `op` | Fields and meaning |
| --- | --- |
| `const` | Boolean `value`; constant truth value |
| `var` | `name`; declared variable |
| `not` | `operand`; negation |
| `and`, `or`, `xor` | `left`, `right`; AND, OR, exclusive OR |
| `implies` | `left`, `right`; false only for true → false |
| `iff` | `left`, `right`; equal truth values |

There is no textual expression parser. Unknown or duplicate fields, malformed
arities, non-Boolean constants, duplicate variable declarations and undeclared
variables fail. IDs are nonempty UTF-8 strings of at most 64 bytes.

Evaluation takes a list with exactly one `{name,value}` binding per variable.
Duplicate names remain detectable because the assignment is a list.
Native Python accepts builtin model/list data or raw JSON text; text input also
preserves duplicate-key rejection.

```sh
cube logic boolean analyze --text '{"variables":[],"formula":{"op":"const","value":true}}'
```

A constant-only model has one empty assignment, so this analysis counts one true
assignment. For declared variables, analysis enumerates false before true, with
the last variable changing fastest.

### Boolean results

Counts, satisfiability, tautology and first true/false witnesses describe the
**entire scan**. `max_rows` changes only the listed assignments. Comparisons also
return implication/equivalence flags and first counterexamples; implication
counterexamples have left=true and right=false.

Work exhaustion raises a resource error before evaluation. It cannot produce an
unsatisfiable or equivalent conclusion from an incomplete scan.

### Boolean limits

| Resource | Maximum or default |
| --- | --- |
| Variables | 12 |
| Combined nodes across both formulas | 256 |
| Formula depth | 32 |
| Assignments | 4,096 |
| Raw JSON | 1 MiB |
| Combined allocation | 8 MiB |
| Compact JSON report | 8 MiB |
| Work | 2,097,152 node visits |
| Listed assignments | Default 32; accepts 0 through 4,096 |

Every node is evaluated without short circuit. Analysis charges combined nodes
multiplied by assignments; single evaluation charges combined nodes.
Long IDs can make a full listing exceed memory or report budgets even within
the row cap. See [Boolean allocation accounting](resource-accounting.md#boolean-formulas).

## Explicit finite multigraphs

Supply `{vertices: [id,...], directed: bool, edges: [{id,from,to},...]}`.
Vertex and edge IDs are unique nonempty UTF-8 strings of at most 64 bytes.
Every endpoint must already exist. Vertices and weights are never inferred.
Edges retain input order and identity, including loops and parallel edges.
Duplicate/unknown JSON fields and malformed types fail before a query.

### Components, paths and cycles

| Query | Result and ordering |
| --- | --- |
| `components` | Ignores direction; includes isolates; components follow their first input vertex and members follow input order |
| `shortest-path` | Unweighted shortest path with vertex and edge IDs; edge input order breaks ties |
| `cycle` | Closed witness or null; preserves directed orientation |

A path from a vertex to itself has zero edges. No route returns JSON null or
Python `None`. In an undirected graph, one loop is a cycle, and two parallel edges
form a two-edge cycle.

### Euler trails

An Euler trail covers every edge exactly once. Undirected non-isolated vertices
must form one weak component and have zero or two odd degrees; a loop contributes
degree two. If two vertices have odd degrees, either can be the start.

A directed graph needs one vertex with out-degree minus in-degree equal to +1
and one with -1, or all vertices balanced. Non-isolated vertices must be weakly
connected. An open directed trail starts at the +1 vertex.

An incompatible requested start is a parameter error. A degree or connectivity
failure returns `exists=false` with a reason. Isolates do not invalidate the trail.
With no edges, an existing requested start or the first input vertex gives a
trivial circuit; with no vertices, the evidence is empty.

```sh
cube graph euler-trail --text '{"vertices":["a"],"directed":false,"edges":[]}'
```

This graph has a trivial circuit at `a` with no edges.

### Graph limits

| Resource | Maximum |
| --- | --- |
| Vertices | 4,096 |
| Edges | 32,768 |
| Raw or compact input transport | 2 MiB |
| Combined allocation | 32 MiB |
| Complete compact report | 8 MiB |
| Work per query | 4,194,304 visits |

Work includes construction (vertices plus four visits per edge), query visits
and conservative component-ordering comparisons.

Lower `max_work`,
`max_memory_bytes` and `max_report_bytes` through limits or CLI flags.

Exhaustion raises a resource error; it never returns no-path, no-cycle or no-trail
evidence. See [graph allocation accounting](resource-accounting.md#finite-graphs).

## Finite-domain research boundary

Finite-domain constraint solving remains
[research](research/finite-domain-evaluation.md), with an evaluation of AC-3 and
backtracking. It is not a production solver or an available dictionary service.
