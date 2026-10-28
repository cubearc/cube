# Classical solving and family-suggestion evaluation

Recorded 2026-10-27 for [#218](https://github.com/cubearc/cube/issues/218) and
[#219](https://github.com/cubearc/cube/issues/219). These are research decisions;
no solver, family-identification operation or pipeline adapter is advertised.
The [original standard-library probe](classical-analysis-probe.py) is outside the
installed SDK and Cargo crates. Run it after `cargo build --locked -p cube-cli`:

```sh
python3 docs/research/classical-analysis-probe.py
```

## Substitution solving: defer production, retain concrete explicit searches

[#145](finite-domain-evaluation.md) establishes a 32-variable finite-domain
research profile, not a shipped solver. Its support-scan accounting cannot simply
be replaced with a node count, and its 32-value ceiling does not establish the
requested 64-symbol bijection profile. [#131](https://github.com/cubearc/cube/issues/131)
explicitly deferred a generic candidate engine. Keep both boundaries. Reuse typed
substitution transforms and the concrete finite searches in `cryptanalysis`;
defer an unconstrained monoalphabetic solver until a realistic caller dictionary
profile and accounting for remaining-key aliases are accepted. No implementation
follow-up is accepted by this decision.

The original probe enumerates full 26-letter bijections with fixed crib mappings,
lexical DFS and all-different forward pruning. Each attempted domain value costs
one unit, including rejected values. It compares a seeded (7) swap hill climber
using only caller crib agreement; it cannot prove exhaustion or contradiction.

| Fixed symbols | DFS maps | Domain visits | Outcome | Seeded score visits |
| --- | ---: | ---: | --- | ---: |
| 0 | 4096 | 183218 | result limit | 0 |
| 20 | 720 | 32162 | complete | 81940 |
| 24 | 2 | 78 | complete | 98328 |
| 26 | 1 | 0 | complete | 106522 |

The seeded run satisfies every supplied constraint in these fixtures, but selects
one of many maps and offers no uniqueness evidence. A duplicate target mapping
proves contradiction before search. Zero-work exhaustion and repeatability are
asserted. The two-free-symbol and six-free-symbol counts are checked against
permutation counts (2 and 720). This tiny crib corpus does not qualify a dictionary
solver or claim performance for 64-symbol alphabets, long texts or language scores.
The complete probe used about 0.30 MiB traced Python memory and 4.5 seconds on the
local run; these are experiment observations, not Rust latency/heap guarantees.

A future explicit profile would require unique scalar alphabet (26–64), ciphertext
<=65536 scalars, ordered `{cipher,plain}` crib pairs, optional caller-owned model
and an exact token dictionary with declared segmentation. Defaults: deterministic
lexical propagation/backtracking, no hidden model, no heuristics unless a seed and
method are explicit. Unknown/repeated symbols and malformed models are typed
input errors; inconsistent valid constraints return `contradiction`. Proposed
results contain full maps, exact plaintext, all surviving-map aliases, independent
model scores and constraint reasons. `complete=false` plus `work_limit`,
`result_limit` or `cancelled` never means unsatisfiable. Scoring and failed branches
must also be charged within 1 Mi visits; <=4096 retained candidates, <=64 MiB
working memory and <=4 MiB report remain unqualified ceilings, not availability.
The multiplicity above is the precise blocker to promising exhaustive map aliases
under those ceilings. Any future Rust, CLI and native Python interface must share
one typed Core implementation; pipeline eligibility is undecided and currently
absent. Solving is not reversible and never establishes authenticity.

## Family suggestions: defer automated labels, expose observations

The probe calls the real `cube investigate triage --json` for every corpus item.
Current triage examines at most 12 header bytes and suggests inspections; it does
not claim cipher identity. Experimental rules use full ASCII-letter counts:
IC >=0.055 suggests both monoalphabetic substitution and transposition, lower IC
suggests polyalphabetic, and any repeated trigram suggests a possible period.
Rule order is fixed; tied explanations retain that order. These thresholds are
experiment assumptions, not calibrated probabilities.

Original prose, its Caesar transform and its reversed transposition produce the
same IC rules. A structured `AB` repetition negative also receives classical
suggestions. Encoded, compressed and random bytes may contain an ASCII subset
unrelated to language. The corpus also includes short and Unicode negatives and a
periodic additive transform. Exact/one-under byte budgets and repeatability are
asserted. The experiment demonstrates ambiguity rather than a measured reduction
in false positives against labeled held-out data. Defer family labels; reuse the
shipped explicit frequency, repeat, IC and period reports so users can evaluate
multiple hypotheses. No automatic transformation or service is accepted.

If revisited, the candidate API is bounded bytes plus an explicit text view and
rule set, returning ordered `{rule_id, observations, assumptions, families}` and
completion status. Default: no labels for fewer than 20 eligible letters; no
probability field. Invalid view/rule IDs are errors. Proposed limits remain 1 MiB
input, 64 rules, 256 suggestions, 16 Mi observation visits and 4 MiB report;
measured prototype uses only three rules and ten inputs, so those larger limits
are not yet qualified. Future Rust/CLI/native Python availability and pipeline
eligibility are absent until that evaluation succeeds. Suggestions are not
reversible, are not modern cipher breaking and do not establish authenticity.

## Primary sources, reuse and portability

- [ACA cipher types](https://www.cryptogram.org/resource-area/cipher-types/), inspected
  2026-10-27: reference for distinct historical profiles, not a licensed model
  corpus or solver backend. No ACA prose, dictionaries or code are bundled.
- [AIMA CSP source](https://aima.cs.berkeley.edu/python/csp.html) documents forward
  checking, backtracking and seeded/min-conflict alternatives. The maintained
  [aima-python repository](https://github.com/aimacode/aima-python) has an
  [MIT license](https://github.com/aimacode/aima-python/blob/f104e03fdc1582f014b3a4c93f1a9b7d2da70114/LICENSE).
  Revision `f104e03fdc1582f014b3a4c93f1a9b7d2da70114` (2026-06-30)
  was inspected on the decision date. It is a reference implementation, not a qualified Rust dependency: Python runtime,
  broad dynamic predicates and different accounting/explanation semantics would
  change Cube's native packaging and contract. No code is copied or dependency added.
- [Michigan Tech IC derivation](https://pages.mtu.edu/~shene/NSF-4/Tutorial/VIG/Vig-IOC.html)
  supports observed pair coincidence. Its language samples do not justify treating
  a threshold as cipher-family proof. Formula-only implementation adds no runtime
  dependency or portability constraint.

All experimental fixtures and code here are original under this repository's
license. Only the measured Python experiment is claimed; it is portable standard
Python plus the existing Cube CLI, without network or external solver execution.
