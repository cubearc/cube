# Classical cryptanalysis evidence

These operations accept caller-owned data and run entirely offline. A score is a
model-dependent heuristic, never the probability that a candidate or key is correct.
Use the typed request schema in [operation contracts](../operations.md) for every
field. CLI commands read one JSON request from stdin or `--file`; native Python
functions accept a dictionary and return a structured dictionary.

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
16 Mi scalar visits, and `limits.max_results` can lower 4096 retained results.
Every source scalar, including filtered punctuation, consumes work; empty attempts
consume one unit. A limit or cooperative Rust cancellation returns a marked prefix.
Serialized reports are capped at 4 MiB; exceeding that ceiling is a typed resource
error, never a silently truncated result. No model, input or report is uploaded.

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
ties. All data and aliases count toward a conservative 3 MiB retention reserve
inside the 4 MiB report ceiling. Reaching a retention/work limit or cancellation
marks a prefix incomplete; omitted candidates are not failures. No modern cipher
or key material discovery is supported.

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
partial-key copy and bounded insertion comparison consumes work. The 16 Mi visit ceiling applies across all lengths,
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
visits, strict <=4 MiB JSON transport and <=4 MiB serialized report. Work/retention
or cancellation returns explicit partial evidence; a serialized report exceeding
its hard ceiling is a typed resource error. Rust operations accept an optional
atomic cancellation flag. Native Python accepts the existing optional keyword
`cancellation=CancellationToken()` and releases the GIL during analysis so another
thread can cancel. CLI and native Python use the same typed Core calls;
Play uses its existing killable worker and safe structured report viewer.
