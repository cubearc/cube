# Lexical and discrete foundations

All operations in this guide are implemented once in `cube-core`, with CLI and
native Python adapters. Reports are structured results, not text-pipeline values.
No new pipeline adapters, engine versions or recipe formats are introduced.

## Exact scalar multisets

`text.multiset.compare` and `text.multiset.subtract` return the same full evidence:
canonical source/requested inventories, source-minus-requested residuals,
requested-minus-source deficits, `contains` and `equal`. Records contain one
`scalar` and exact `count`, ordered by Unicode code point. Empty inputs and NUL
are valid. Nothing is silently trimmed or clipped.

Preparation and counting are shared with `text.statistics`. Defaults preserve all
Unicode scalars; explicit normalization is none/NFC/NFD/NFKC/NFKD, case handling
is preserve/ASCII-lower and filtering is all/ASCII-letters. Normalization runs
first, then ASCII case handling and filtering. This is not linguistic case folding,
grapheme analysis or dictionary lookup. Reports include the normalization data
version, matching existing statistics behavior.

```sh
cube text multiset compare --text listen --requested silent
cube text multiset subtract --text ab --requested aab
cube text multiset compare --file source.txt --requested-file requested.txt --normalization NFC
```

```python
import cube
assert cube.multiset_compare("listen", "silent")["equal"]
r = cube.multiset_subtract("ab", "aab")
assert not r["contains"]
assert r["deficits"] == [{"scalar": "a", "count": 1}]
assert cube.multiset_compare("é", "e\u0301", profile={"normalization": "NFC"})["equal"]
```

Each raw input is <=1 MiB, normalized output <=8 MiB, prepared input <=1 Mi
scalars and each inventory <=10,000 distinct scalars. Combined Core memory is
<=32 MiB and compact reports <=8 MiB. Work <=16 Mi units charges raw bytes, every
normalized scalar visited (including filtered-out scalars), retained scalar counts
and conservative tree/merge visits. `limits`/CLI flags may lower `max_work`,
`max_memory_bytes` and `max_report_bytes`. Core preflight charges vector growth,
both counting trees, at most one live normalization temporary, retained records
and the report buffer. Counts remain complete; a lower cap raises a typed error.

## Exact combinatorial counts

`number.permutations` is P(n,k), `number.binomial` is C(n,k), and
`number.multinomial` counts an explicit list of group sizes. They accept builtin
unsigned integers with n<=128, k<=n, at most 128 groups and checked total<=128.
Bool, float, signed text, prefixes and coercions are rejected. Zero-sized and
empty groups are valid; empty selections count as one.

Results must fit u128. Overflow is a resource/range error, not saturation,
wrapping or approximation. Cancellation happens before multiplication for binomial
coefficients, so fitting results never fail just because a factorial would overflow.
All C(128,k) fit, while 52! does not. No arrangements or general factorial API
are generated.

```sh
cube number permutations --n 5 --k 2
# 20
cube number binomial --n 128 --k 64 --json
cube number multinomial --groups 2,1,1 --json
cube number multinomial --groups '' --json
cube number pascal --rows 4
```

```python
assert cube.permutations(5, 2) == 20
assert cube.binomial(5, 2) == 10
assert cube.multinomial([2, 1, 1]) == 12
assert cube.pascal_triangle(4) == [[1], [1, 1], [1, 2, 1], [1, 3, 3, 1]]
```

Rust `Count.value` is u128; CLI count JSON and Pascal rows use decimal strings.
Python returns exact builtin ints and lists of ints. Count reports require a
conservative 128-byte bound, with a caller cap <=4 KiB; arithmetic/cancellation
work is <=65,536 charged operations and scratch <=64 KiB. Lower `max_work` and
`max_report_bytes` through count limits. Scalar APIs retain no large scratch.

Pascal takes a **row count**: zero is empty, and 129 includes row indices 0..128.
It uses the same exact binomial coefficients through the Pascal recurrence;
its maximum 8,385 coefficients plus 129 row visits cost 8,514 work units.
Retained triangle output is separate from <4 KiB active row scratch. A compact
JSON cap <=1 MiB is preflighted as `128 + 4*rows + 42*coefficients` bytes.
Python Pascal limits use `max_work`/`max_output_bytes`; CLI uses matching flags.

## Tagged Boolean formulas

A model contains `variables` in caller order, one `formula` and optional `other`
for comparison. Supported tagged forms are:

| Tag `op` | Required fields | Meaning |
| --- | --- | --- |
| `const` | Boolean `value` | Constant truth value |
| `var` | `name` | Declared variable |
| `not` | `operand` | Negation |
| `and`, `or`, `xor` | `left`, `right` | Conjunction, disjunction, exclusive OR |
| `implies` | `left`, `right` | False only for true -> false |
| `iff` | `left`, `right` | Equal truth values |

There is no textual expression parser or natural-language evaluator. Unknown or
duplicate fields, malformed arities, non-Boolean constants, duplicate declarations
and undeclared variables fail. IDs are nonempty UTF-8 strings <=64 bytes.
Evaluation requires exactly one `{name,value}` binding per declared variable;
the assignment is a list, so duplicate names cannot be erased by map conversion.
Both formulas share the declared union variable order.

```sh
cube logic boolean analyze --text '{"variables":["A"],"formula":{"op":"or","left":{"op":"var","name":"A"},"right":{"op":"not","operand":{"op":"var","name":"A"}}}}'
cube logic boolean evaluate --text '{"variables":["A"],"formula":{"op":"var","name":"A"}}' --assignment '[{"name":"A","value":true}]'
```

```python
model = {
    "variables": ["A", "B"],
    "formula": {"op": "implies", "left": {"op": "var", "name": "A"},
                "right": {"op": "var", "name": "B"}},
    "other": {"op": "var", "name": "B"},
}
r = cube.boolean_analyze(model, limits={"max_rows": 1})
assert r["true_count"] == 3 and r["false_count"] == 1
assert not r["listing_complete"]
witness = r["relationship"]["implication_counterexample"]
e = cube.boolean_evaluate(model, witness)
assert e["value"] and not e["other_value"]
```

Analysis enumerates false before true, with the last declared variable changing
fastest. A constant-only model has one empty assignment. Counts, satisfiability,
tautology and first true/false witnesses always reflect the **entire** bounded
scan. A row cap affects listing only. Comparisons return implication/equivalence
flags and first counterexamples; implication counterexamples have left=true and
right=false. Work exhaustion raises a resource error before evaluation rather than
returning an unsatisfiable/equivalent conclusion.

Limits: <=12 variables, <=256 combined nodes across both trees, depth <=32,
<=4,096 assignments, <=2 Mi charged node visits, <=1 MiB raw JSON and <=8 MiB each
for conservative combined allocation and compact report bounds. Every formula
node is evaluated, without short circuit; work is exactly combined nodes times
assignments for analysis, or combined nodes for one evaluation. Row listing defaults
to 32 and permits 0..4096. Report preflight uses actual ID escape/retention sizes;
full tables with long IDs can exceed the report/memory budget and fail explicitly.
Raw JSON parsing reuses duplicate-key/node/depth bounds before typed AST allocation.
Native Python accepts builtin model/list data or raw JSON strings, preserving
strict duplicate-key checks for string inputs.

## Explicit finite multigraphs

A graph is `{vertices: [id,...], directed: bool, edges: [{id,from,to},...]}`.
Vertex and edge IDs are unique, nonempty UTF-8 strings <=64 bytes. Endpoints must
already exist; vertices/weights are never inferred. Edges are ordered and retain
their identities, including loops and parallel edges. JSON duplicate/unknown fields
and malformed types fail before a query. Typed bounded decoding avoids retaining
a second untyped graph object tree.

```python
graph = {
    "vertices": ["a", "b", "isolated"], "directed": False,
    "edges": [{"id": "x", "from": "a", "to": "b"},
              {"id": "y", "from": "a", "to": "b"}],
}
assert cube.graph_components(graph)["result"] == [["a", "b"], ["isolated"]]
assert cube.graph_shortest_path(graph, "a", "b")["result"]["edges"] == ["x"]
assert cube.graph_cycle(graph)["result"]["edges"] == ["x", "y"]
assert cube.graph_euler_trail(graph, start="b")["result"]["circuit"]
```

Save that explicit graph as `graph.json` for CLI use:

```sh
cube graph components --file graph.json
cube graph shortest-path --file graph.json --source a --target b
cube graph cycle --file graph.json
cube graph euler-trail --file graph.json --start b
```

`components` ignores direction, includes isolates, orders components by their
first input vertex and lists members in input order. `shortest-path` uses shared
unweighted BFS, returning vertex and edge IDs; adjacency edge input order resolves
route ties. Source=target is a zero-edge path; no route is JSON null/Python None.
`cycle` returns a closed witness or null; an undirected loop is a one-edge cycle,
and two parallel undirected edges form a two-edge cycle. Directed orientation is
preserved. DFS is internal to cycle witnessing, not a general traversal API.

`euler-trail` uses Hierholzer to cover every edge once. Undirected non-isolated
vertices must form one weak component and have zero or two odd degrees; loops
contribute degree two. With two odd vertices the start must be either odd vertex.
Directed graphs need one +1 out-in and one -1 vertex, or all balanced, and weak
connectivity of non-isolates. The directed open trail starts at the +1 vertex.
A requested incompatible start is a parameter error. Degree/connectivity failure
is an explicit `exists=false` result with a reason. Isolates do not invalidate an
edge-covering trail. With no edges, an existing requested start or the first input
vertex gives a trivial circuit; no vertices gives empty evidence.

Graph ceilings: 4,096 vertices, 32,768 edges, 2 MiB raw/compact transport, 32 MiB
combined allocation, 8 MiB complete compact report and 4 Mi charged visits per
query. Reports include construction work (vertices + four visits per edge), query
vertex/edge visits and conservative component-ordering comparisons. Preflight
counts lookup/adjacency/traversal/report storage with actual ID encoding sizes.
Lower `max_work`, `max_memory_bytes` and `max_report_bytes` through limits/flags;
exhaustion raises a resource error, never no-route/no-cycle/no-trail evidence.
Core allocation bounds do not include the baseline/object overhead of CPython or
other host runtimes. Native object transport is separately prewalked before JSON
encoding to cap cardinality, bytes and depth.

## Finite-domain research boundary

[#145's evaluation](research/finite-domain-evaluation.md) records a concrete
feasibility decision, original independently checked fixtures, AC-3/backtracking
measurements, dependency evidence and a proposed finite model/limit contract.
It adds no production solver, dictionary, generic framework or placeholder
operation. Boolean truth scans and graph traversals remain distinct algorithms.
