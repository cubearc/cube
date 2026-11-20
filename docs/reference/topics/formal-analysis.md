# Graphs, constraints and formal computation

Pass a structured request to `cube <family> analysis <command>`. Python uses
`cube.<operation_id_with_underscores>(request)`. JavaScript and Play use the
version 1 `request` port and return a `report` port. See the
[operation contracts](../operations.md) for complete shapes.

```sh
cube graph analysis traversal --text '{"graph":{"vertices":["A","B","C"],"directed":true,"edges":[{"id":"ab","from":"A","to":"B"}]},"mode":"depth_first"}'
```

Discovery visits A, B and C in that order. B records edge `ab`. Finish order
is B, A, C.

## Graphs

| Command | Input and evidence |
| --- | --- |
| `graph analysis traversal` | Choose `depth_first`, `topological` or `strong_components`. DFS accepts an optional start and visits the remaining roots in vertex order. Only directed graphs support topological ordering and SCCs. Topological ordering chooses the earliest ready vertex and returns an edge-ID cycle witness if cyclic. SCCs and their members follow original vertex order. |
| `graph analysis weighted-path` | Supply `start`, `goal` and `algorithm.method`. `dijkstra` requires nonnegative edges. `bellman_ford` accepts signed edges and reports a reachable negative-cycle witness when present. `a_star` additionally requires a consistent nonnegative `heuristic` array in vertex order with zero at the goal. Finite results carry vertices and edge IDs. Unreachable and unbounded results are distinct. |
| `graph analysis all-pairs-paths` | Floyd–Warshall uses signed edges, zero diagonals and the lightest parallel edge. Vertices index the distance and unbounded matrices. Distance is null for unreachable and unbounded pairs. A pair is unbounded when a negative cycle lies on a walk between its endpoints. Optional `paths` gives up to 256 ordered start–goal queries. |
| `graph analysis disjoint-sets` | Supply ordered distinct u32 `ids` and ordered `actions` with tags `union`, `find` and `same_component`. Each component's representative is its first ID in the input. Queries record their original action index. |
| `graph analysis bipartite-matching` | Supply disjoint ordered `left` and `right` vertex lists and edges from left to right. Hopcroft–Karp resolves choices using input order and keeps the first edge identity for duplicate adjacency. Set `minimum_cover` to true for the ordered Kőnig cover. |

Graphs allow up to 4,096 vertices and 32,768 edges. All-pairs paths lower the
vertex ceiling to 256. Disjoint sets allow 65,536 IDs and 1,048,576 actions
subject to the 8 MiB request cap. Edge weights are signed 64-bit. Costs use
checked signed 128-bit arithmetic. JSON costs and finite matrix distances are
decimal strings. Python returns integers. Keep other JSON integers outside the
safe JavaScript range in decimal strings.

`max_work` lowers the default hard ceiling. Traversal and disjoint sets have
4,194,304 units. Weighted paths, matching and Floyd–Warshall have 16,777,216.
Floyd charges relaxation candidates separately from bounded path reconstruction.
Exhausted graph operations return a resource-limit error. Tarjan's SCC and
Hopcroft–Karp matching follow their published algorithms.
[Tarjan, 1972](https://doi.org/10.1137/0201010)
[Hopcroft and Karp, 1973](https://doi.org/10.1137/0202019)

## Exact cover and SAT

- `logic analysis exact-cover` takes explicit ordered primary `columns` and
  `rows` with IDs and column membership. Provide `max_solutions` from 0 to
  1,000. Up to 256 columns, 4,096 rows and 65,536 incidences are admitted.
  Dancing links choose the smallest active column with input-order ties.
  Each solution lists row IDs in original order. An empty universe has one
  empty cover. Check `complete` and `stop_reason` before drawing conclusions
  from an empty or truncated result. See [Knuth's Algorithm X](https://arxiv.org/abs/cs/0011047).
- `logic analysis dpll` takes a `formula` containing ordered positive
  `variables` and signed `clauses`. Each signed literal must name a declared
  variable. Limits are 256 variables, 4,096 clauses and 65,536 literals.
  Duplicate literals collapse and tautologies are removed. An empty clause
  is false and the empty formula is true. DPLL uses unit propagation, then the
  first unassigned variable with false explored first.
- `logic analysis cdcl` accepts the same CNF with `action: "solve"` and
  optional `proof: true`. Profile `cube_first_uip_rup_v1` learns first-UIP
  clauses and backjumps. With proof enabled, UNSAT carries an ordered RUP
  certificate ending with an empty clause. Submit that certificate as
  `clauses` with `action: "verify_proof"` and the original `formula`. The
  verifier returns `verified`, `invalid_proof` with a clause index, or
  `incomplete`. See [CDCL backend qualification](../../research/cdcl-backend.md).

All three operations default to 1,048,576 work units. Third-party callers can
lower this with `max_work`. CDCL solve also accepts `max_conflicts` and
`max_learned_literals` within 1,048,576 and 65,536 respectively. A SAT report
includes a complete assignment. An exhausted or cancelled search returns
`incomplete` with a reason. Python accepts a `CancellationToken` through
the keyword argument `cancel` for exact cover, DPLL and CDCL.

## Lambda terms and SKI

`esolang analysis lambda` accepts ASCII identifiers, application separated by
whitespace or parentheses, and abstractions written as `\x.body`. Application
associates to the left. An abstraction extends to the end of its parenthesized
scope. `normal_order` is leftmost outermost full beta reduction, including under
abstractions. `applicative_order` reduces leftmost innermost first. Set `eta`
to true to also reduce `\x.f x` when the binder has no occurrence in `f`.
The renderer returns canonical fresh binder names and preserves free names.

| Example term | Normal-order observation |
| --- | --- |
| `(\x.\y.x) y` | The result `(\v0.y)` retains the free `y`. |
| `(\n.\f.\x.f (n f x)) (\f.\x.f x)` | Church successor applied to one reduces to two. |
| `(\t.\f.t) yes no` | The Church true selector returns `yes`. |
| `(\f.(\x.f (x x)) (\x.f (x x))) (\y.y)` | A bounded Y-combinator reduction can end with a limit reason. |

`esolang analysis ski` accepts space-separated `S`, `K` and `I` atoms
and parenthesized applications. Enable `free_symbols` for additional ASCII
identifiers. Leftmost outermost rewrites reduce `I x` to `x`, `K x y` to
`x` and `S x y z` to `x z (y z)`.

Both term reducers allow 4,096 input nodes, nesting depth 128, 65,536 live
nodes, 100,000 reductions and 1,048,576 traversal and copy units. Lower work
or reductions with `max_work` or `max_reductions`. Set `trace` to retain up
to 4,096 steps with zero-based preorder redex indexes. Limits return the last
committed term with a reason. An incomplete reduction establishes no
termination property.

## Cellular automata and reproducible streams

`esolang analysis cellular-automaton` takes the generation count and an
`initial` grid. For `kind: "life"`, include `rows`, `cols` and a flat
row-major `cells` string of zeros and ones. The B3/S23 rule counts the eight
Moore offsets. For `kind: "elementary"`, pass a byte-sized rule number and a
one-row `cells` string. Its bit index is four times the left cell plus two
times the center plus the right cell. The boundary defaults to `finite_dead`.
`toroidal` wraps every offset, counting duplicates on dimensions of one or two.
Inspect `final_grid` and any requested `snapshots` in caller order. The limits
are 1,048,576 cells, 4,096 generations, 16,777,216 cell updates and an 8 MiB report.

`number analysis lfsr` generates up to 1,048,576 bits with width 2 through
64, explicit nonzero `state` and `tap_mask` within that width. Each step
emits the low bit, shifts right and inserts the parity of the masked state.
Select `msb_first` or `lsb_first` byte packing and `reject` or `pad_zero` for
a partial byte. The result includes lowercase hex, exact valid-bit count and
the final state.

`number analysis prng` produces up to 65,536 u32 words using either the
single-u32-seeded `mt19937ar_2002` profile or Marsaglia's `xorshift32`
with shifts 13, 17 and 5. Xorshift requires a nonzero seed. Both take an
explicit `seed` or a matching saved `state`. Use the returned `final_state`
as the next request's `source`. The MT state contains all 624 words and the
cursor. Initialization, twisting and output use up to 4,194,304 work units.
These streams support reproducible experiments.
[MT19937 authors' 2002 reference](https://www.math.sci.hiroshima-u.ac.jp/m-mat/MT/MT2002/mt19937ar.html)
[Marsaglia, 2003](https://doi.org/10.18637/jss.v008.i14)
