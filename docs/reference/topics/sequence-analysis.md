# Sequence analysis

Compare two exact sequences, locate byte patterns, or inspect the edits between
two inputs. Each profile specifies its units, tie rules, and resource limits.
The [operation contracts](../operations.md) define the complete JSON requests
and result fields.

Run a profile with JSON from `--text`, `--file`, or standard input. This
comparison returns distance 2 using unrestricted Damerau–Levenshtein.

```sh
cube sequence edit-distance --text '{"operands":{"unit":"scalars","left":"CA","right":"ABC"},"method":"dl"}'
```

## Units and offsets

| Unit | Request values | Meaning |
| --- | --- | --- |
| `bytes` | Canonical padded Base64 strings | Each decoded byte is one unit |
| `bits` | Canonical padded Base64 strings | Hamming compares all eight bits of each byte |
| `scalars` | UTF-8 strings | Each Unicode scalar is one unit |
| `lines` | UTF-8 strings or objects with a `base64` field | LF terminates a line and stays in that line |

Scalar comparison uses the supplied Unicode exactly. A precomposed letter and
its decomposed spelling are distinct sequences. Source spans are zero-based
and half-open. `start` and `end` count units. `byte_start` and `byte_end` refer
to original bytes, including UTF-8 encoding or line endings.

## Similarity and common spans

| Operation | Result |
| --- | --- |
| `sequence.edit_distance` | Exact Levenshtein, unrestricted Damerau–Levenshtein, OSA, Ukkonen, or Myers bit-vector distance |
| `sequence.hamming` | Mismatch count and total units for equal-length inputs |
| `sequence.jaro` | Jaro or Jaro–Winkler score and its complement |
| `sequence.lcs` | Hirschberg subsequence with paired source positions |
| `sequence.common_spans` | Longest common prefix, suffix and contiguous substring |
| `sequence.ratcliff_obershelp` | Ordered matching blocks and twice their total length divided by both input lengths |
| `sequence.monge_elkan` | Mean of best token comparisons, with optional mean of both directions |

Edit distance defaults to Levenshtein. Ukkonen requires `max_distance` in
0–4096. Above the threshold it returns a null distance and sets
`above_threshold`. The Myers bit-vector profile requires 1–63 left units.
OSA limits a portion of the sequence to one edit. Unrestricted
Damerau–Levenshtein permits further edits after a transposition.

Hamming accepts up to 16 MiB per packed byte input or 1 MiB per scalar
string. Its normalized ratio is null for two empty inputs. Jaro chooses the
earliest unused matching right scalar within half the longer length minus
one, with a minimum radius of zero. Jaro–Winkler applies its prefix boost
above the threshold, which defaults to 0.7. Scale defaults to 0.1 and may
range from 0 to 0.25.

Hirschberg chooses the lowest right split on a tie. Contiguous witnesses
choose the earliest left start, then the earliest right start. Ratcliff–
Obershelp applies that rule in every partition, so interchanging the inputs
can change the selected blocks and score. Monge–Elkan retains duplicate
tokens and favors the first matching right index on a tie.

Most pair requests accept up to 65,536 units and 1 MiB per input. Edit and
subsequence matrices are limited to 4,194,304 cells. Ratcliff–Obershelp
accepts up to 16,384 units and charges at most 8,388,608 cell visits across
partitions. Monge–Elkan accepts up to 256 tokens per side, 64 scalars per
token and 64 KiB total token bytes. Its pair and work budgets include both
directions when requested.

## Alignment and numeric series

`sequence.align` takes a byte or scalar pair and an explicit global or local
mode. Match, mismatch, and linear gap default to 2, −1, and −1. An affine gap
of length `k` costs `open + (k − 1) × extend`. The report includes the optimum,
paired spans, and an exact traceback. Equal scores prefer a match or
substitution, then deletion, then insertion. Equal gap states favor extension.
A local comparison with no positive alignment returns an empty result at zero.
The matrix is capped at 1,048,576 cells and each input at 16,384 units.

`sequence.dtw` takes two numeric series and returns a closed-endpoint path and
accumulated cost. Local cost is squared difference by default, with absolute
difference available. An optional `radius` binds the absolute index
difference. Equal costs prefer diagonal, up, then left. Samples are finite
and within ±1e150. The path uses at most 1,048,576 visited cells. Two empty
series return an empty path with zero cost and a disconnected pair is rejected.

## Sets, distributions, and documents

`sequence.set_similarity` takes explicit token arrays or overlapping Unicode
q-grams. Q-grams use windows of 1–16 scalars without padding. Jaccard, Dice,
Overlap, and Tversky compare sets. The q-gram L1 profile retains frequencies.
Tversky accepts nonnegative fractions for alpha and beta, defaulting both to
one. Exact ratios use decimal strings for numerator and denominator.

`sequence.vector_similarity` accepts dense arrays or sorted unique sparse
keys with nonnegative finite weights. Cosine reports a null score for a zero
vector. KL and Jensen–Shannon normalize distributions to unit mass, use base
two by default, and reject a zero-mass distribution. KL represents positive
mass over zero as `positive_infinity` with a null numeric field. Jensen–
Shannon returns divergence.

Build a caller-owned model with `sequence.tfidf_model` and supply it to
`sequence.tfidf_compare`. The model counts document frequency and uses
`ln((N + 1)/(df + 1)) + 1` with raw term counts and L2 cosine. An unseen term
has frequency zero. The SHA-256 identity covers the UTF-8 profile name, a zero
separator, the big-endian 32-bit document count, and the sorted entries. Each
entry consists of its big-endian 32-bit byte length, token bytes, and
big-endian 32-bit frequency. Empty documents produce a null cosine score.

## Byte search

| Operation | Profiles |
| --- | --- |
| `sequence.exact_search` | KMP, Boyer–Moore with both shifts, or verified Rabin–Karp |
| `sequence.multi_search` | Sparse Aho–Corasick, retaining duplicate pattern IDs |
| `sequence.bitap` | Shift-And exact search or Wu–Manber Levenshtein ending states |

Search inputs are canonical Base64. The haystack limit is 16 MiB. Exact and
multipattern records contain half-open byte ranges. Multipattern ordering
is by end, start, then original pattern index. With overlap disabled, the
first eligible record in that order wins. Exact and multipattern counts
cover the complete scan. `complete` reports whether a result cap omitted
listed records. The default listing cap is 1,000.

Bitap accepts a 1–63 byte pattern. Approximate mode requires `max_errors` in
0–8 and below the pattern length. Each hit names the exclusive end and its
minimum edit distance. Set `start_witness` to compute the earliest start with
bounded traceback. Exact matches have a start automatically. Search stops
with a resource error when its work budget is exhausted.

## Edit scripts and unified diff

The Myers profile produces a minimum insertion/deletion script for bytes,
scalars, or lines. A substitution is deletion followed by insertion. The
Patience and Histogram profiles work on lines. Patience anchors a stable
increasing sequence of unique matches. Histogram selects low-occurrence
anchors, with an occurrence cap of 64 by default and a hard cap of 256.
Both use bounded Myers in unmatched regions and report `minimal: false`.

The line anchor policies follow the Patience and Histogram algorithms in
Git v2.51.0, with Cube's documented earliest-source tie rules. See the
[Patience reference](https://raw.githubusercontent.com/git/git/v2.51.0/xdiff/xpatience.c)
and [Histogram reference](https://raw.githubusercontent.com/git/git/v2.51.0/xdiff/xhistogram.c).

Set `unified` on a line request to render UTF-8 hunks. Context defaults to
three lines and accepts 0–1,000. Labels default to `a/input` and `b/input`.
Rendered lines preserve CRLF and mark missing final newlines. Control
characters in labels are rejected. A report has an 8 MiB ceiling, and the
anchored profiles charge fallback work against their existing work budget.

## SDKs

Use the same request shape with the native Python function for the operation,
such as `cube.sequence_edit_distance`. The JavaScript/TypeScript SDK accepts
the versioned ID with a named `request` input and returns the `report` output.
All three interfaces use the same Rust implementation.
