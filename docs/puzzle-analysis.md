# Bounded puzzle analysis

These direct Rust/CLI/native-Python operations consume caller data. They contain
no dictionary, language service, probabilistic simulation or inferred image
interpretation. The operation catalog remains the discovery source; none of
these structured reports has a pipeline adapter. Engine API 1 and recipe format
1 are unchanged.

## Multiword anagrams

```sh
cube text anagram search ab --dictionary '["ab","a","b"]'
cube text anagram search aa --dictionary '["a"]' --reuse
```

```python
import cube

r = cube.anagram_search("ab", ["ab", "a", "b"])
assert r["solutions"] == [[0], [1, 2]]
assert r["complete"]
```

The Rust entry point is `text::anagram::search(source, dictionary, profile,
options)`. Preparation uses the shared statistics/multiset normalization,
ASCII-case and filtering rules, in that order. Defaults preserve every Unicode
scalar. `normalization` is `none`, `NFC`, `NFD`, `NFKC` or `NFKD`; `case` is
`preserve` or `ascii_lower`; `filter` is `all` or `ascii_letters`. No other case
folding, tokenization or language assumptions apply.

The report's canonical dictionary retains the first occurrence of each distinct
prepared **text**, with all original zero-based alias indexes. Different words
with identical inventories remain distinct words. Each solution indexes that
canonical dictionary in nondecreasing order. With `reuse=false` (default), its
indexes are strictly increasing; aliases do not grant extra uses. Search prunes
inventory deficits through the same residual subtraction used by multiset
comparison. It emits each unordered word multiset once.

`max_words` defaults to 8 and accepts 1..8. `max_results` defaults to 100 and
accepts 0..1000. An empty source has the empty solution; a zero result/work budget
can prevent listing it and explicitly marks the search incomplete. Empty prepared
dictionary entries are invalid, including with an empty source. An exhausted
search with no solution is complete; a truncated search cannot establish
impossibility. `stop_reason` is `work_limit` or `result_limit`. The result limit
uses one additional discovered solution to prove truncation, without retaining
that extra result. If the work limit prevents that proof, `work_limit` takes
precedence. `nodes` counts visited residual states, including completed states.

Limits include 10,000 supplied entries, 64 prepared scalars per entry, 1 MiB raw
source, 4 MiB aggregate raw dictionary UTF-8, 1,000,000 work units, 32 MiB scratch
and 8 MiB compact report. Shared preparation also bounds each raw word to 1 MiB,
normalized text to 8 MiB and source inventories to 10,000 distinct scalars.
Preprocessing work exhaustion is a typed resource error before search; execution
exhaustion returns a deterministic incomplete prefix. Work charges raw bytes,
prepared visits/counts, string-index comparison allowances, candidate count
visits, search nodes and emitted path indexes. Depth never exceeds eight.

## Word ladders

```sh
cube text word-ladder shortest --dictionary '["cat","cot","cog","dog"]' \
  --start cat --end dog
cube text word-ladder validate --dictionary '["cat","cot"]' \
  --path '["cat","cot"]'
```

```python
dictionary = ["dog", "cog", "dot", "cot", "cat", "cot"]
r = cube.word_ladder_shortest(dictionary, "cat", "dog")
assert [r["dictionary"][i]["text"] for i in r["path"]] == [
    "cat", "cot", "cog", "dog"
]
assert cube.word_ladder_validate(dictionary, ["cat", "cot", "cog", "dog"])["valid"]
```

Rust provides `text::word_ladder::{shortest,validate}`. Only normalization and
ASCII-case preparation are selectable. Punctuation, whitespace and combining
scalars remain part of words. Every dictionary word and endpoint must have the
same nonzero prepared scalar length. An edge changes exactly one scalar; repeated
adjacent words, insertions and deletions are unsupported. Dictionary aliases retain
original indexes; canonical prepared text sorts lexicographically, resolving BFS
ties. Numeric internal vertex IDs allow up to 32 four-byte scalars per word
without changing the graph's ID contract.

A wildcard index includes the removed scalar position and all remaining scalars.
It generates substitution edges; ordered edges then go to the shared `Graph`
validation and BFS. The report contains the canonical dictionary, indexed path,
edge count and total construction/query work. `start=end` yields one vertex;
missing endpoints are invalid model input. A fully searched disconnected graph
returns `status="no_path"`, `path=null`, `complete=true`. Construction or BFS
exhaustion fails with a resource error and never returns `no_path`.

Validation reports the first membership or adjacency failure: `position` is the
failing word index and `step` the preceding zero-based edge index (null for the
first word). An incompatible word length is a typed invalid-input error. Empty
paths/models are invalid. For validation, `edge_count=0` means no adjacency graph
was constructed; it does not describe the dictionary graph.

Ceilings are 10,000 supplied aliases, 4,096 distinct words, 32 scalars/word, 1 MiB
aggregate dictionary UTF-8, 32,768 undirected edges, 16 Mi work, 32 MiB aggregate
scratch and 8 MiB report. Validation paths additionally have at most 4,096 words
and 1 MiB UTF-8. Work charges preparation, index comparison allowances, bounded sort comparisons,
neighbor pairs and shared graph visits. The graph's own 4 Mi visit ceiling also
applies. Limits combine lexical state with graph allocations and both adjacency
directions. Dense neighbor buckets fail explicitly when the edge cap is exceeded.

CLI dictionary selectors are mutually exclusive `--dictionary JSON` and
`--dictionary-file PATH`. Files must be regular UTF-8 files containing JSON string
arrays; they are read with a finite cap (4 MiB for anagrams, 1 MiB for ladders).
JSON syntax/escaping counts toward that transport cap. Rust/Python receive values
or explicit bounded JSON; they do not open dictionaries or contact a network.

## Exact finite probabilities and dice

```sh
cube probability finite analyze '{"outcomes":[{"id":"H","weight":1},{"id":"T","weight":1}],"event_a":["H"]}'
cube probability dice sum '{"dice":[[{"value":1,"weight":1},{"value":2,"weight":1}]]}'
cube probability dice compare '{"left":{"dice":[]},"right":{"dice":[]}}'
```

```python
r = cube.finite_probability({
    "outcomes": [{"id": "H", "weight": 1}, {"id": "T", "weight": 1}],
    "event_a": ["H"],
})
assert r["probability_a"] == {"numerator": 1, "denominator": 2}

d6 = [{"value": n, "weight": 1} for n in range(1, 7)]
r = cube.dice_sum({"dice": [d6, d6]})
assert r["total_mass"] == 36
assert [p["mass"] for p in r["distribution"]] == [1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1]
```

Rust owns `probability::{finite_analyze,dice_sum,dice_compare}`, with typed models
and bounded unique-key parsers. CLI inputs use the ordinary positional/text/file/
stdin selectors. Python accepts builtin JSON data or JSON text, checks its size,
node count and depth before encoding, and rejects coercions, cyclic structures
and duplicate JSON object keys. Unknown model/configuration fields are errors.

A finite model has `outcomes: [{id,weight,value?},...]`, `event_a: [id,...]` and
optional `event_b: [id,...]`. IDs are unique nonempty strings up to 64 UTF-8 bytes.
Weights are nonnegative u64 integers or canonical unsigned decimal strings
(`0` or a digit string without leading zeros). At least one weight is positive.
Events are sets of known IDs, expressed as lists without duplicates. `event_b`
enables P(B), P(A intersect B), P(A|B) and a posterior distribution of the same
outcomes conditioned on B. Zero-mass conditioning is invalid. No independence,
likelihood, causality or model truth is inferred from a finite table.

Values are optional signed i32 integers; provide all or none. Expectations use
`{negative,numerator,denominator}`, where magnitude and denominator are u128,
denominator is positive and zero is always `{false,0,1}`. Probabilities use reduced
nonnegative `{numerator,denominator}`. Positive/negative weighted sums, products,
and counts are checked u128 intermediates. Arithmetic overflow is a resource
error, even if a more general arbitrary-precision method could reduce the result.

A dice model is `{"dice": [[{"value": i32, "weight": u64},...],...]}`.
Each die has 1..64 face entries and positive total mass; up to eight dice are
explicitly assumed independent. Repeated numeric faces aggregate; zero weights
produce no support. Zero dice yield the identity distribution at sum zero with
mass one. Bounded convolution retains sorted signed i64 sums and positive u128
masses without materializing Cartesian outcomes. A comparison receives
`{"left": dice_model, "right": dice_model}` and additionally assumes independence
between the collections. Win means strictly left sum > right sum; ties are
separate. Equal expectations and pairwise winning relations imply no transitivity.
Reports retain the supplied assumptions and explicit independence statement.

CLI JSON serializes exact fraction components, weights and masses as decimal
strings. Python returns builtin exact integers for those same fields; outcome
IDs stay strings. Face/sum keys are exact signed fixed-width integers. Limits:
4,096 outcomes/retained PMF points, 1 MiB JSON, 16 Mi work, 32 MiB scratch and
8 MiB report. Work counts outcome/event visits, face aggregation, convolution
pairs, output points and distribution comparison pairs. Intermediate support,
arithmetic and all limits fail without silently partial probability results.

## Spatial lag measurements

```sh
cube image autocorrelation --input stripes.png --region 0,0,4,1 \
  --channel gray --lags '[{"dx":1,"dy":0}]'
```

```python
image = cube.Image.from_samples(4, 1, "gray", bytes([0, 255, 0, 255]))
r = image.autocorrelation("gray", [{"dx": 1, "dy": 0}], region=(0, 0, 4, 1))
assert r["measurements"][0]["correlation"] == -1.0
assert r["measurements"][0]["squared_difference"] == 195075
```

Rust provides `Image::autocorrelation(region,channel,lags,limits)`. The required
region uses absolute top-left coordinates; `(dx,dy)` pairs `(x,y)` with
`(x+dx,y+dy)` inside that same region. Positive x points right, positive y down.
Only overlap is visited: no wrapping, padding or samples outside the region.
Zero-overlap lags fail before sample computation; zero lag is allowed. Order and
duplicate lags are preserved. Empty lag lists return empty measurements.

Each lag reports source/target overlap rectangles, count N, sums X/Y, squared
sums XX/YY, cross-product XY and squared difference. With 8-bit samples and at
most 1 Mi pairs per lag, every raw moment is provably bounded by 1 Mi * 65025
and fits u64. Exact centered intermediates fit u128/i128. Population covariance
is `(N*XY-X*Y)/N²`; normalized correlation is
`(N*XY-X*Y)/sqrt((N*XX-X²)*(N*YY-Y²))`. Zero variance makes correlation null.
Derived f64 results allow absolute tolerance 1e-12 across platforms; exact moments
are the authoritative evidence.

Selection is one physical gray/red/green/blue/alpha/index channel. No implicit
grayscale, blending, gamma/profile processing or palette conversion occurs.
Use `--expand-palette`/`expand_palette=True` before measuring indexed **colors**;
`index` deliberately measures physical palette indexes. The report retains
representation and source-format provenance. Alpha is an ordinary physical
sample, including transparent pixels.

Ceilings are 1 Mi selected pixels, 256 signed-i32 lags, 16 Mi aggregate
pixel-pair visits, 16 MiB scratch and 1 MiB report, within existing image decode
limits. The scratch estimate covers retained measurements and serialization;
it excludes the already-owned immutable image. Explicit palette expansion uses
the existing image sample limits. All overlaps and aggregate work are preflighted.
There is no automatic period selection, significance test, template search or
hidden-content recognition.

Work units are deterministic accounting conventions, not CPU instruction counts.
All APIs report conservative memory/report upper bounds and work consumed. Caller
limits may only lower the hard ceilings. Conservative preflight can reject a
request before consuming its nominal input/count ceiling; a ceiling is not a
promise that every maximal combination fits all other budgets simultaneously.
