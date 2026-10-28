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
`(e-u)*N / ((N-1)*IC-u*N+e)`, undefined for N<2 or nonpositive denominator.
No estimate is rounded into a claimed key length. Counts remain separate from
this estimate. Repeated candidate periods are charged independently.

The model conventions follow the [Stanford n-gram text](https://web.stanford.edu/~jurafsky/slp3/3.pdf);
the column construction follows [Michigan Tech's IC tutorial](https://pages.mtu.edu/~shene/NSF-4/Tutorial/VIG/Vig-IOC-Len.html).
