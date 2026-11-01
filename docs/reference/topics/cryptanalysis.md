# Classical cryptanalysis evidence

These operations accept caller-owned data and run entirely offline. A score is a
model-dependent heuristic, never the probability that a candidate or key is correct.
Use the typed request schema in [operation contracts](../operations.md) for every
field. CLI commands read one JSON request from stdin or `--file`; native Python
functions accept a dictionary and return a structured dictionary.

## Resource stops

Work limits, result limits, cancellation and search pruning can return an explicitly
incomplete prefix. Exceeding the serialized report cap raises a resource error.
The subsections below define the charged work and evidence retained in each case.
Read [complete and partial results](../../concepts/results.md) before treating
missing candidates as a negative conclusion.

## Language scoring

`analysis.language.score` / `cube cryptanalysis score` / `cube.language_score`
accept an ordered candidate list and an explicit model. `counts` supports scalar
n-grams of width 1–8. With smoothing `a`, entry count `c`, total `T` and `V`
entries, each observed score is `ln((c+a)/(T+a*(V+1)))`; absent grams use the
single aggregate unseen bucket `ln(a/(T+a*(V+1)))`. `probabilities` accepts entry
masses plus an explicit unseen mass that together sum to one within `1e-9`.
This is an overlapping n-gram scoring model, not a conditional language generator.

`normalization: exact` retains every Unicode scalar. `ascii_letters` selects only
ASCII letters and uppercases them, without folding accented letters. Model keys
must already be canonical and unique. `dictionary` uses exact, case-sensitive
Unicode-whitespace tokens with punctuation retained, weights in [-100,100], and
an explicit unseen weight. It does not segment unspaced words.

Each term is rounded to integer nanounits (natural logarithms for n-grams); the
rounding error is at most 0.5e-9 per term. Reports include the total, sample count,
mean truncated toward zero, method version, SHA-256 of serialized typed model,
and stable descending mean rank. Model entry order is part of its identity.
Native Python retains exact integer scores. JSON/Play use canonical decimal strings
for integers outside JavaScript's exact range; null remains distinct from zero.
Empty samples have null scores and rank last. Equal scores retain caller order.

```sh
printf '%s' '{"candidates":["B","A",""],"model":{"kind":"dictionary","entries":[{"text":"A","weight":2}],"unseen_weight":-1}}' | cube cryptanalysis score
```

The input texts collectively use at most 1 MiB, the model at most 1 MiB and 10000
entries, and the candidate list at most 4096 items. `limits.max_work` can lower
16,777,216 scalar visits, and `limits.max_results` can lower 4096 retained results.
Every source scalar, including filtered punctuation, consumes work; empty attempts
consume one unit. A limit or cooperative Rust cancellation returns a marked prefix.
Serialized reports are capped at 4 MiB; exceeding that ceiling is a typed resource
error, never a silently truncated result. No model, input or report is uploaded.

## Frequency distance

`analysis.frequency.compare` (`cryptanalysis frequency`, `frequency_compare`)
computes the [Pearson statistic](https://www.itl.nist.gov/div898/handbook/eda/section3/eda35f.htm)
`sum((O-E)^2/E)` from aligned `bins` containing unique exact `label`, nonnegative
integer `observed` and finite `expected` mass. The required `expected_policy` is
`counts` (totals must agree within `1e-9 * max(1, observed_total)`) or
`normalize_proportions` (rescale supplied masses to the observed total).
Positive masses lie in [1e-300,1e12]; zero is permitted. A zero/zero bin is omitted,
while a positive observation with zero expectation reports `zero_expected_mismatch`
(infinite distance). A distance beyond finite f64 range reports `out_of_range`.
Empty observations are `undefined`; normalization requires positive total mass.

Lower distance means closer agreement. The raw distance has no logarithm applied;
compare finite calculations with relative tolerance 1e-12. Each report retains
per-bin contributions, processed expected total, full observed total, normalization
policy and SHA-256 of the ordered labels/expected masses/policy. For a complete
finite result, degrees of freedom are positive expected bins minus one, assuming
the supplied fixed model has no fitted parameters. This operation does not assess
sample adequacy or infer a p-value, posterior probability or cipher identity.

The request is capped at 1 MiB and 10000 bins; labels use 1–256 UTF-8 bytes and
observed counts sum to at most 16 Mi. Each bin charges `max(1, observed)` units
without expanding counts. The shared work/result limits and cancellation retain
an incomplete prefix, with degrees of freedom undefined. At most 4096 bin results
and 4 MiB of serialized evidence are retained. No caller data is acquired online.

## Period evidence

`analysis.period.estimate` / `cube cryptanalysis period` / `cube.period_estimate`
uses the ASCII-letter view, an explicit list of periods 1–4096, n-gram width 2–8,
and caller `expected_ic` in [1/26,1]. It reports repeated n-grams in lexical order,
zero-based filtered positions and original UTF-8 byte offsets, pair-distance
histograms and per-period divisor counts. All position pairs are considered within
the work ceiling; position collection stops at 65536 records and marks incomplete.
There is no silent sampling. Each source scalar, n-gram scalar, column visit,
pair and divisor test consumes work. At most 4096 periods are accepted.

Observed IC is `sum(c*(c-1))/(N*(N-1))`, undefined below two letters. Mean column
IC excludes short columns and is undefined if none qualify. Ranking uses ascending
absolute distance from the caller's expected IC, retaining caller order for ties.
It is a heuristic. With uniform IC `u=1/26` and expected IC `e`, Friedman is
`(e-u)*N / ((N-1)*IC-u*N+e)`, undefined for N<2, a uniform expected model or nonpositive denominator.
No estimate is rounded into a claimed key length. Counts remain separate from
this estimate. Repeated candidate periods are charged independently.

The model conventions follow the [Stanford n-gram text](https://web.stanford.edu/~jurafsky/slp3/3.pdf);
the column construction follows [Michigan Tech's IC tutorial](https://pages.mtu.edu/~shene/NSF-4/Tutorial/VIG/Vig-IOC-Len.html).

## Finite key and transposition candidates

`analysis.caesar.rank` (`cryptanalysis caesar`, `caesar_rank`) tries explicit
`shifts`, or 0–25 if omitted. `analysis.affine.rank` (`cryptanalysis affine`,
`affine_rank`) tries explicit `{a,b}` keys, or all 312 coprime-a pairs in ascending
a then b order. Invalid supplied keys produce safe rejection records and consume
work. Empty lists mean no attempts. All text transforms reuse Core's inverse,
preserving non-ASCII scalars, punctuation and case.

`analysis.transposition.rank` (`cryptanalysis transposition`, `transposition_rank`)
requires an ordered `configurations` list. Supported tagged forms are `rail_fence`
with rails, initial rail, direction and explicit rail readout; `columnar` with a
zero-based column permutation; and `route` with rows, columns and named fill/read
routes. Route/corner meanings are the existing Core transposition conventions.
Full rectangles are required for routes; Columnar keeps ragged lengths. Every alias
reports the input scalar position supplying each plaintext scalar. No padding,
factorial key enumeration or inferred route is added.

Search text is capped at 65536 scalars, key/configuration lists at 1024 and retained
plaintexts at 4096. Each transform and score is charged even when invalid or a
duplicate. Transposition reserves a conservative 12 scalar visits plus explicit
readout/permutation items for every attempt. Exact plaintext deduplication retains
all attempted key/configuration aliases, including duplicate list positions.
Results sort by descending mean score, with first-observed candidate order for
ties.

All data and aliases count toward a conservative 3 MiB retention reserve
inside the 4 MiB report ceiling. Reaching a retention/work limit or cancellation
marks a prefix incomplete; omitted candidates are not failures.

No modern cipher or key material discovery is supported.

## Vigenere beam search

`analysis.vigenere.rank` (`cryptanalysis vigenere`, `vigenere_rank`) requires
`lengths` (1–64, at most 32 entries), `beam_width` (1–256) and the same explicit
model. It forms ASCII uppercase columns, scores all 26 inverse Caesar shifts in
each column, and expands partial keys in deterministic order. Partial scores sum
column mean scores; ties use lexical keys. N-grams are evaluated *within each
column*, so a unigram model is often the more appropriate column heuristic.
Dictionary models retain their exact token rule and may produce uninformative ties.

Final surviving keys decrypt the original text with the existing letter-only key
advancement and are rescored on the complete plaintext. Reports retain original
UTF-8 byte offsets of eligible letters, model identity, key aliases, total pruned
states and final candidate scores. Every column extraction, transform, score and
partial-key copy and bounded insertion comparison consumes work. The 16,777,216-visit ceiling applies across all lengths,
including duplicate lengths and discarded partial keys. Any pruning sets
`complete=false` and `stop=pruned` unless another limit or cancellation stopped the
search first. A beam is never advertised as exhaustive when it discarded states.

## Homophonic token observations

`analysis.homophonic.inspect` (`cryptanalysis homophonic`, `homophonic_inspect`)
accepts an explicit token list (<=65536 items, <=4096 distinct, <=1 MiB serialized).
A token is a nonempty exact string of at most 256 UTF-8 bytes, not a Unicode
character. Lexically ordered token/adjacent-pair counts and the exact IC numerator
and denominator describe only the processed prefix. IC is null below two tokens.
No separators, normalization or token boundaries are inferred.

Optional `groups` reuse the existing homophonic `{symbol,tokens}` definition and
its validator: <=4096 unique tokens overall, 1–256 per unique scalar label,
<=64 KiB serialized definition, and no duplicate ownership. Group counts follow
caller group order, remain separate hypotheses, and retain a definition hash.
Grouped IC covers mapped tokens only; `unmapped` makes that denominator explicit.
The optional result ceiling limits each observed table; reaching it marks a prefix
incomplete. Declared groups and exact observed tokens remain distinct evidence.

## Explicit known plaintext

`analysis.crib.verify` (`cryptanalysis crib`, `crib_verify`) accepts ordered
candidates, a crib, a nonnegative offset, and `view`: `bytes`, `scalars`, or
`ascii_letters`. Every datum is tagged `{"kind":"text","text":"..."}` or
`{"kind":"bytes","bytes":[...]}`. Scalar views require valid UTF-8. Offsets are
zero-based in the declared view; each compared position also carries its original
UTF-8 byte offset, or null when outside the candidate. Byte view compares arbitrary
bytes, including NUL and non-UTF-8. The ASCII view uppercases eligible letters and
preserves provenance; it does not fold accents or infer offsets.

Reports list exact matched/mismatched positions. Missing positions mismatch;
empty and interrupted cribs have `all_match=null`. A supplied classical key can
optionally re-encode a text candidate against explicit `ciphertext`, comparing the
complete output bytes without normalization. It uses the same closed Caesar,
Affine, Vigenere or transposition key types as candidate reports. Invalid keys or
missing re-encoding inputs produce separate safe evidence errors. No keys are
searched. A crib match proves only the listed positions under that view, never
identity, authenticity, or the rest of the plaintext.

Limits are <=4096 candidates, <=1 MiB aggregate candidate/crib/ciphertext data,
<=65536 crib positions and retained comparison records, <=16 Mi comparison/work
visits, strict <=4 MiB JSON transport and <=4 MiB serialized report.

Work/retention
or cancellation returns explicit partial evidence; a serialized report exceeding
its hard ceiling is a typed resource error. Rust operations accept an optional
atomic cancellation flag.

Native Python accepts the existing optional keyword
`cancellation=CancellationToken()` and releases the GIL during analysis so another
thread can cancel. CLI and native Python use the same typed Core calls;
Play uses its existing killable worker and safe structured report viewer.

## Substitution constraints

`analysis.substitution.solve` (`cryptanalysis substitution`, `substitution_solve`)
solves the explicit `observed_bijection_v1` profile. Supply an ordered `alphabet`
of 26–64 unique non-whitespace Unicode scalars, `ciphertext`, and `outside`:
`preserve` or `reject`. There is no case folding or normalization. Optional
`cribs` contain canonical `{cipher,plain}` scalar pairs. Optional `aliases`
contain `{alias,symbol}` pairs: distinct noncanonical cipher scalars explicitly
mapped to canonical alphabet symbols. Aliases are observations, not extra keys.

An optional `dictionary` declares `segmentation: unicode_whitespace` and exact
`words`. Every ciphertext token must match a dictionary word under one shared
bijection, with punctuation retained and outside-alphabet literals unchanged.
Repeated tokens share a constraint and retain all original UTF-8 start offsets.
Duplicate dictionary words share a `dictionary_classes` entry retaining every
original `input_indices` position. Candidate `word_choices` select those classes;
the Cartesian product of duplicate source choices is represented without expansion.
An empty dictionary rules out nonempty tokens. Without a dictionary, the solver
enumerates mappings only for observed alphabet symbols.

Each candidate contains plaintext, the constrained `mapping`, a `canonical_key`
(targets in alphabet order), `free_cipher`, `free_plain` and decimal `key_count`.
Every permutation of the free targets is a valid full key in that class; the
count is exactly their factorial. The canonical key fills free positions in
alphabet order. These completions produce identical plaintext and are not
silently discarded. For example, `ABBA` with dictionary `NOON, DEED, NOON`
has two plaintext classes, each representing `24!` keys.

`candidates` retain deterministic search order. With a dictionary, minimum
remaining domain chooses a token, ties by first appearance; word choices follow
first dictionary appearance. Without one, both source and target order follow
the alphabet. Optional `model` reuses [language scoring](#language-scoring), with
its own digest. `ranking` contains candidate indices by descending mean score,
with search order on ties. `request_sha256` binds the serialized typed request,
including defaults, limits, source ordering and model.

`contradiction` means a proved inconsistent crib or a complete search with no
solutions. An interrupted empty search proves nothing. `plaintext_unique` and
`key_unique` are null when unresolved; multiple found classes prove nonuniqueness,
and a free-key count greater than one proves key nonuniqueness. Uniqueness applies
only to the supplied constraints, never to historical identity or authenticity.

| Resource | Default / hard ceiling |
| --- | --- |
| Ciphertext | 65536 scalars and 1 MiB |
| Dictionary | 4096 input words, 256 scalars each, 1 MiB aggregate UTF-8 |
| Distinct ciphertext tokens | 4096 |
| Cribs / cipher aliases | 4096 / 256 |
| Optional model | Language-scoring limits |
| `limits.max_work` | 1048576 visits; may be lowered to zero |
| `limits.max_results` | 128 / 4096 candidates |
| `limits.max_memory_bytes` | 64 MiB / 64 MiB conservative workspace reserve |
| JSON request / report | 4 MiB each; strict transport depth 16 and 400000 nodes |

Structural validation is bounded independently. Work charges all ciphertext
scalars, source dictionary scalars (including duplicates), crib/alias entries,
unique-word indexing, token/domain indexing, search nodes, attempted assignments,
full support tests (including failed tests), plaintext rendering and optional
scoring. A support test reserves its whole word length before examining it.
The reported workspace upper bound reserves request duplication, index edges,
search stack, scoring scratch, retained report and serialization copies before
allocating domains. It is a conservative operation reserve, not process RSS;
smaller memory budgets may reject even small requests.

Constraint metadata must fit a conservative 1 MiB reserve minus 8192 bytes;
candidates collectively retain at most 3 MiB of canonical JSON. Metadata/preflight
or final serialization overflow raises a typed resource error. Search work,
cancellation or candidate retention returns a marked prefix. Reaching exactly
`max_results` is complete only if the remaining search is exhausted; finding
another solution marks `result_limit`. No stochastic search, hidden dictionary,
external solver or downloaded model is used.

## Cipher-family suggestions

`analysis.family.suggest` (`cryptanalysis families`, `family_suggest`) requires
`profile: ascii_classical_evidence_v1` and exactly one of `text` or `bytes`.
The byte view selects ASCII letters and uppercases them; it never automatically
decodes binary or folds accents. `observations` records byte/letter counts,
diversity, IC, control and non-ASCII bytes, dominance and transport-like evidence.
These are facts about the declared view, separate from the suggestions.

Eligibility requires at least 64 letters, 8 distinct letters, letter fraction
at least 0.75, no letter occupying 0.25 or more of the stream, no binary controls
or non-ASCII bytes, and no all-hex or unspaced transport-like signature. That
signature requires at least 64 bytes, only alphanumerics or `+/=_-`, and at least
one digit or one of those punctuation marks. Ineligible inputs return explicit
`eligibility_failures` and no family claims.

`rules` defaults to the following order. A caller may supply an empty subset or
reorder it; duplicates and unknown rules are invalid.

| Rule | Compatibility evidence | Remaining ambiguity |
| --- | --- | --- |
| `nonuniform_coincidence_v1` | Whole-letter IC in [0.052,0.095] | Plaintext, monoalphabetic substitution and transposition have indistinguishable count evidence |
| `periodic_columns_v1` | At least 24 letters and 8 distinct letters per column, squared coefficient of variation over occupied bins at least 0.1 in every column, mean IC in [0.052,0.095], and mean IC at least 0.012 above global IC | Repeating-key polyalphabetic text, periodic plaintext, interleaved or constructed streams can share this signal |

`periods` defaults to 2–16; up to 32 unique periods in 2–64 may be supplied.
Periods lacking 24 letters per column are omitted. `columns` records the exact
column evidence, including sample floor, minimum diversity and variation.
These thresholds are a finite policy assumption for this profile, not universal
language facts or calibrated confidence. Uniform or random-looking data without
qualified column evidence produces no identity claim. Suggestions include rule
IDs, compatible families, assumptions, alternative explanations and tested periods.
Nothing is transformed or automatically executed.

Input is limited to 1 MiB and strict 4 MiB JSON transport. Work defaults to
16 Mi visits, with one per input byte (one for empty input), one per rule, and
`letter_count + 27 * period` for each tested column observation. Structural
validation has fixed bounds. At most two suggestions are possible; shared
`limits.max_results` can lower their retention. Work/cancellation returns partial
facts and completed rules only, with `observations: null` if the input scan did
not finish. A result cap marks omitted suggestions incomplete; report overflow
is a typed error. Read the [measured evaluation](../../research/classical-analysis-evaluation.md)
for known false positives and counterexamples.
