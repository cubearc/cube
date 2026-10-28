# Bounded puzzle analysis

Search a supplied dictionary, calculate exact probabilities or measure spatial
samples. These operations use caller data through direct Rust, CLI and native
Python interfaces, without pipeline adapters. Supply the dictionary or model;
Cube does not provide a language service or infer an image interpretation.
Reports expose work consumed and conservative memory/report upper bounds.

See [resource limits](concepts/resource-limits.md) and
[complete and partial results](concepts/results.md) for shared terminology.

## Multiword anagrams

```sh
cube text anagram search ab --dictionary '["ab","a","b"]'
cube text anagram search aa --dictionary '["a"]' --reuse
```

The first command returns canonical dictionary-index solutions `[[0], [1,2]]`.
The second permits reuse, so the word `a` can be selected twice.

### Anagram preparation and results

Preparation uses the [shared text profile](reference/text-preparation.md).
Defaults preserve every scalar. Rust callers use
`text::anagram::search(source, dictionary, profile, options)`.

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

### Anagram limits and stopping

| Resource | Maximum |
| --- | --- |
| Supplied dictionary entries | 10,000 |
| Prepared scalars per entry | 64 |
| Raw source | 1 MiB |
| Aggregate raw dictionary UTF-8 | 4 MiB |
| Work | 1,000,000 units |
| Scratch memory | 32 MiB |
| Compact report | 8 MiB |
| Search depth | 8 |

The shared profile's per-word preparation and source-inventory limits also apply.
Preprocessing work exhaustion raises a typed resource error before search.
Execution exhaustion returns a deterministic incomplete prefix, as described above.

Work counts raw bytes, preparation visits/counts, string-index comparison
allowances, candidate count visits, search nodes and emitted path indexes.

## Word ladders

```sh
cube text word-ladder shortest --dictionary '["cat","cot","cog","dog"]' \
  --start cat --end dog
cube text word-ladder validate --dictionary '["cat","cot"]' \
  --path '["cat","cot"]'
```

The shortest path in the first command is `cat → cot → cog → dog` with three
edges. The second validates the one-edge path `cat → cot`.

### Ladder preparation and results

Rust provides `text::word_ladder::{shortest,validate}`. Only normalization and
ASCII lowercasing are selectable, using the shared profile
[normalization and case options](reference/text-preparation.md#order-and-defaults).
Punctuation, whitespace and combining scalars remain part of words. Every dictionary word and endpoint must have the
same nonzero prepared scalar length. An edge changes exactly one scalar; repeated
adjacent words, insertions and deletions are unsupported. Dictionary aliases retain
original indexes; canonical prepared text sorts lexicographically, resolving BFS
ties. The report contains the canonical dictionary, indexed path,
edge count and total construction/query work. `start=end` yields one vertex;
missing endpoints are invalid model input. A fully searched disconnected graph
returns `status="no_path"`, `path=null`, `complete=true`. Construction or BFS
exhaustion fails with a resource error and never returns `no_path`.

Validation reports the first membership or adjacency failure: `position` is the
failing word index and `step` the preceding zero-based edge index (null for the
first word). An incompatible word length is a typed invalid-input error. Empty
paths/models are invalid. For validation, `edge_count=0` means no adjacency graph
was constructed; it does not describe the dictionary graph.

### Ladder limits

| Resource | Maximum |
| --- | --- |
| Supplied aliases | 10,000 |
| Distinct words | 4,096 |
| Prepared length per word | 32 scalars |
| Aggregate dictionary UTF-8 | 1 MiB |
| Undirected edges | 32,768 |
| Work | 16,777,216 units |
| Combined scratch | 32 MiB |
| Report | 8 MiB |
| Validation path | 4,096 words and 1 MiB UTF-8 |

Work counts preparation, index and sort comparison allowances, neighbor pairs
and shared graph visits. The graph's separate 4,194,304-visit ceiling also applies.
Memory includes lexical state, graph allocations and both adjacency directions.
Dense neighbor buckets fail when the edge cap is exceeded. See
[ladder construction notes](resource-accounting.md#word-ladders) for index details.

### Dictionary input

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

The finite model reports P(H)=1/2. The single two-face die assigns mass one to
each of sums 1 and 2. Comparing two zero-dice collections reports a tie with
probability one.

### Probability input

Rust provides `probability::{finite_analyze,dice_sum,dice_compare}` with typed models
and strict JSON parsers. CLI inputs use the ordinary positional/text/file/
stdin selectors. Python accepts builtin JSON data or JSON text, checks its size,
node count and depth before encoding, and rejects coercions, cyclic structures
and duplicate JSON object keys. Unknown model/configuration fields are errors.

### Finite outcomes

A finite model has `outcomes: [{id,weight,value?},...]`, `event_a: [id,...]` and
optional `event_b: [id,...]`. IDs are unique nonempty strings up to 64 UTF-8 bytes.
Weights are nonnegative u64 integers or canonical unsigned decimal strings
(`0` or a digit string without leading zeros). At least one weight is positive.
Events are sets of known IDs, expressed as lists without duplicates. `event_b`
enables P(B), P(A intersect B), P(A|B) and a posterior distribution of the same
outcomes conditioned on B. Zero-mass conditioning is invalid. No independence,
likelihood, causality or model truth is inferred from a finite table.

### Exact result values

Values are optional signed i32 integers; provide all or none. Expectations use
`{negative,numerator,denominator}`, where magnitude and denominator are u128,
denominator is positive and zero is always `{false,0,1}`. Probabilities use reduced
nonnegative `{numerator,denominator}`. Positive/negative weighted sums, products,
and counts are checked u128 intermediates. Arithmetic overflow is a resource
error, even if a more general arbitrary-precision method could reduce the result.

### Dice and independence

A dice model is `{"dice": [[{"value": i32, "weight": u64},...],...]}`.
Each die has 1..64 face entries and positive total mass; up to eight dice are
explicitly assumed independent. Repeated numeric faces aggregate; zero weights
produce no support. Zero dice yield the identity distribution at sum zero with
mass one. Bounded convolution retains sorted signed i64 sums and positive u128
masses without materializing Cartesian outcomes. A comparison receives
`{"left": dice_model, "right": dice_model}` and additionally assumes independence
between the collections.

Win means strictly left sum > right sum; ties are
separate. Equal expectations and pairwise winning relations imply no transitivity.
Reports retain the supplied assumptions and explicit independence statement.

CLI JSON serializes exact fraction components, weights and masses as decimal
strings. Python returns builtin exact integers for those same fields; outcome
IDs stay strings. Face/sum keys are exact signed fixed-width integers.

### Probability limits

| Resource | Maximum |
| --- | --- |
| Outcomes or retained probability-mass points | 4,096 |
| JSON input | 1 MiB |
| Dice | 8 |
| Face entries per die | 64 |
| Work | 16,777,216 units |
| Scratch | 32 MiB |
| Report | 8 MiB |

Work counts outcome/event visits, face aggregation, convolution pairs, output
points and distribution comparison pairs. Exceeding intermediate support,
arithmetic or resource limits raises an error; probability results are never
silently partial.

## Spatial lag measurements

For a caller-supplied 4 × 1 grayscale `stripes.png` with samples
`[0,255,0,255]`, measure adjacent pairs:

```sh
cube image autocorrelation --input stripes.png --region 0,0,4,1 \
  --channel gray --lags '[{"dx":1,"dy":0}]'
```

The correlation is -1 and the squared difference is 195075. To measure the
same samples already held in memory, use the native SDK:

```python
import cube

image = cube.Image.from_samples(4, 1, "gray", bytes([0, 255, 0, 255]))
r = image.autocorrelation("gray", [{"dx": 1, "dy": 0}], region=(0, 0, 4, 1))
assert r["measurements"][0]["correlation"] == -1.0
assert r["measurements"][0]["squared_difference"] == 195075
```

### Regions and lags

Rust provides `Image::autocorrelation(region,channel,lags,limits)`. The required
region uses absolute top-left coordinates; `(dx,dy)` pairs `(x,y)` with
`(x+dx,y+dy)` inside that same region. Positive x points right, positive y down.
Only overlap is visited: no wrapping, padding or samples outside the region.
Zero-overlap lags fail before sample computation; zero lag is allowed. Order and
duplicate lags are preserved. Empty lag lists return empty measurements.

### Measurements

Each lag reports source/target overlap rectangles, count N, sums X/Y, squared
sums XX/YY, cross-product XY and squared difference. With 8-bit samples and at
most 1 Mi pairs per lag, every raw moment is provably bounded by 1 Mi * 65025
and fits u64. Exact centered intermediates fit u128/i128. Population covariance
is `(N*XY-X*Y)/N²`; normalized correlation is
`(N*XY-X*Y)/sqrt((N*XX-X²)*(N*YY-Y²))`. Zero variance makes correlation null.
Derived f64 results allow absolute tolerance 1e-12 across platforms; exact moments
are the authoritative evidence.

### Physical channels

Selection is one physical gray/red/green/blue/alpha/index channel. No implicit
grayscale, blending, gamma/profile processing or palette conversion occurs.
Use `--expand-palette`/`expand_palette=True` before measuring indexed **colors**;
`index` deliberately measures physical palette indexes. The report retains
representation and source-format provenance. Alpha is an ordinary physical
sample, including transparent pixels.

### Spatial limits

| Resource | Maximum |
| --- | --- |
| Selected pixels | 1,048,576 |
| Signed-i32 lag pairs | 256 |
| Aggregate pixel-pair visits | 16,777,216 |
| Scratch | 16 MiB |
| Report | 1 MiB |

Existing image decode/sample limits also apply. Scratch covers retained
measurements and serialization, excluding the already-owned immutable image.
Palette expansion uses the image sample limits. All overlaps and aggregate work
are checked before measurement.

There is no automatic period selection, significance test, template search or
hidden-content recognition. Caller limits can lower the hard ceilings. See
[why a request can fail before using the whole allowance](concepts/resource-limits.md#why-a-request-can-fail-before-using-the-whole-allowance).
