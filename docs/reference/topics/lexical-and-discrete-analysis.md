# Lexical and discrete analysis

Query explicit finite models and count arrangements.
The [discrete model reference](../../discrete-foundations.md) defines scalar
multisets, tagged Boolean formulas, finite graphs, combinatorial counts and Pascal rows.

## Results and limits

- Results are typed reports; these operations have no pipeline adapters.
- Counts use `u128` in Rust, exact integers in Python, and decimal strings in CLI JSON.
- Exhausting a work, memory or report limit is an error. It does not mean the mathematical problem is impossible.

Finite-domain constraint solving remains [research](../../research/finite-domain-evaluation.md),
not an available solver.
