# Bounded CDCL backend qualification

Reviewed 2026-11-20 for [Cube #430](https://github.com/cubearc/cube/issues/430).
Cube uses the Rust `cube_first_uip_rup_v1` profile for `logic.cdcl` v1.
The implementation shares explicit CNF validation and unit propagation with
DPLL. It introduces no dependency for native Python, JavaScript or WebAssembly.

| Candidate reviewed | License and packaging | Budget and proof findings |
| --- | --- | --- |
| CaDiCaL `rel-2.1.3` (`f13d744`) | MIT. A C++ build and Rust boundary would need qualification on each target. | Limits count conflicts, decisions, preprocessing rounds and local search rounds. The header exposes a terminator and proof tracers. The interface does not establish a hard allocation cap or a shared literal-inspection bound inside preprocessing and learning. [Pinned header](https://github.com/arminbiere/cadical/blob/rel-2.1.3/src/cadical.hpp), [license](https://raw.githubusercontent.com/arminbiere/cadical/rel-2.1.3/LICENSE) |
| Varisat `0.2.2` | MIT or Apache-2.0. Rust library and proof tooling. | Supports writing proofs, proof processors and on-the-fly proof checking. The documented Solver API does not expose hard allocation, literal-work and conflict admissions together or a callable interruption point in each preprocessing and learning loop. [Release](https://docs.rs/crate/varisat/0.2.2), [Solver API](https://docs.rs/varisat/latest/varisat/solver/struct.Solver.html) |
| MiniSat `2.2.0` | MIT. C++ library. | The limited solver API counts conflicts and propagations, with asynchronous interruption. These counters do not cover all clause inspections, preprocessing and allocation. The project lists its proof-logging source as a separate older release. [Release archive](https://minisat.se/MiniSat.html), [Solver header](https://github.com/niklasso/minisat/blob/master/minisat/core/Solver.h), [license](https://github.com/niklasso/minisat/blob/master/LICENSE) |

The profile uses deterministic first-unassigned false-first decisions, ordered
literal scans, first-UIP resolution and nonchronological backjumping. The report
provides `work`, `conflicts` and `learned_literals`. The version fixes the
ordering and conflict rule. The focused integration is the version 1
[formal operation](../reference/topics/formal-analysis.md#exact-cover-and-sat)
in the same milestone.

## Bounds and evidence

Original CNF admits up to 256 variables, 4,096 clauses and 65,536 literals.
At most 65,536 learned literals are retained, including at most that many
nonempty learned clauses. The solver stores dense 16-bit literals, one
assignment and reason per variable, a decision stack of at most 256 entries,
and one 256-entry first-UIP scratch array. Learned vectors are limited to the
number of variables plus vector growth, and total capacity is bounded by a
fixed multiple of admitted literals. Optional proof output uses signed
32-bit IDs with the same retained-literal bound. The solver and replay
working set fit the 64 MiB operation ceiling from these structural limits.
The transport additionally caps compact requests at 2 MiB and reports at
8 MiB.

Each normalized clause and literal, propagation scan, conflict, rollback and
resolution step consumes work. Cancellation is polled in the normalization,
search, learning and verification loops. The solver starts with a ceiling of
1,048,576 work units and 1,048,576 conflicts. Callers can lower those ceilings
and learned-literal storage. Work, conflicts, storage or cancellation can
return an incomplete report with the exact reason.

RUP verification negates each proposed clause, runs unit propagation over
the original and previously verified clauses, and requires a conflict.
The final empty clause is mandatory. The verifier accepts the certificate as
explicit data and does not use solver decisions, reasons or conflict graphs.
The first-UIP approach follows [Marques-Silva and Sakallah](https://users.encs.concordia.ca/~chvatal/GRASP99.pdf).

Reproduce the qualification tests with
`cargo test --locked -p cube-core --test formal_operations`. They compare
both solvers against exhaustive truth tables, independently check returned
models and separately replay UNSAT certificates. They exercise cancellation
during normalization and distinct work, conflict and learned-literal stops.

On macOS, the debug CLI admitted 256 variables, 4,096 clauses and 65,536
literal occurrences in one request, using 70,144 work units and 29,593,600
bytes of peak resident memory for the entire process. One additional
occurrence returned a resource-limit error. The four-pigeon, three-hole
instance produced a verified certificate after 1,620 work units and six
conflicts. The six-pigeon, five-hole instance used 74,757 work units and
retained 289 learned literals. These measurements complement the structural
bounds and do not imply a universal process RSS outside that environment.
