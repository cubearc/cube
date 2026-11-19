# Exact numerics and statistical analysis

Use `cube number analysis <name>` to compute a report from an explicit JSON
request. Pass the same request to the native Python `number_<name>` function
or the `request` port in JavaScript, WebAssembly and Play. These operations use
version 1 direct execution. See [the operation reference](../operations.md) for
field schemas and interface availability.

```sh
cube number analysis crt --text '{"pairs":[{"residue":2,"modulus":6},{"residue":8,"modulus":9}]}'
```

The result is `{"status":"solution","residue":8,"modulus":18}`, so the solutions
are all integers congruent to 8 modulo 18.

## Representations and arithmetic

| Command | Request and result |
| --- | --- |
| `roman` | Encode 1 through 3,999 or decode one canonical ASCII numeral. Decoding defaults to uppercase. Enable `ascii_case_insensitive` to accept lowercase ASCII. |
| `balanced-ternary` | Encode a signed decimal magnitude through 2¹²⁸−1, or decode up to 82 trits. `T` represents −1. Enable `canonicalize` to remove leading zeros on decode. |
| `gray` | Encode or decode an exact unsigned integer of width 1 to 128. Supply a decimal string. Enable `binary` for exactly that many MSB-first binary digits. |
| `decimal-check` | Summarize digit sum, decimal root and signed residue modulo nine, or check an addition, subtraction or multiplication claim. A congruent result can also occur for an incorrect claim. |
| `crt` | Combine up to 256 unsigned 64-bit residue and modulus pairs in caller order. The empty system is 0 modulo 1. Inconsistent constraints return the first conflicting pair and its gcd. Modulus zero and overflow fail. |
| `primality` | Test a u32 with `trial_division` (default) or `miller_rabin`. The latter uses bases 2, 7 and 61, whose proved range includes every u32. |
| `sieve` | List primes up to an inclusive limit of 1,000,000 using `eratosthenes` (default) or `atkin`. The list fails if it exceeds `max_results`, which defaults to 100,000. |
| `factor` | Trial division returns sorted u32 prime factors with multiplicity. `pollard_rho` searches for a single proper u64 divisor using explicit seeds, constant and step limit, with up to 16 ordered retries. An incomplete search can return no divisor. |
| `recurrence` | Request a checked i64 arithmetic or geometric progression, a u64 Fibonacci or Lucas prefix, or a positive u64 Collatz run. Prefix lengths and Collatz retention are capped at 4,096. Collatz has separate step, cycle and overflow outcomes. |

Signed decimal inputs accept one optional ASCII sign and leading zeros, with up
to 4,096 digits where the operation accepts decimal integer text. JSON carries
new wide exact fields as decimal strings. Where a field supports ordinary
64-bit JSON integers as well, values outside the safe JavaScript range use
decimal strings. Python returns builtin integers for exact numeric fields.

The algorithm references are [Jaeschke's fixed-base range proof](https://doi.org/10.1090/S0025-5718-1993-1192971-8),
[Atkin and Bernstein's prime sieve](https://doi.org/10.1090/S0025-5718-03-01501-1)
and [Pollard's factor search](https://doi.org/10.1007/BF01933667).

## Combinatorial order

| Command | Order and bounds |
| --- | --- |
| `lehmer` | Rank or unrank a distinct permutation of zero-based IDs, or convert its Lehmer digits. Up to 34 items. The empty permutation has rank zero. |
| `factoradic` | Encode or decode an unsigned 128-bit integer using up to 35 most-significant-first digits. The final digit has weight 0! and is zero. Encode zero as `[0]`. |
| `lexicographic-rank` | Rank or unrank an exact multiset of up to 34 scalars using an explicit ordered, distinct alphabet. Identical scalars share a class. |
| `partitions` | Count up to `n = 256` or enumerate up to `n = 64` in reverse lexicographic order. Supply `max_part` or `part_count` to restrict the result. The partition of zero is the empty list. |
| `de-bruijn` | Generate a cycle for 1 to 256 distinct ordered scalars and order 1 to 32, with at most 1,048,576 cyclic symbols. `linearized` appends the first `order − 1` cyclic symbols, including repeats for a one-symbol alphabet. |
| `permutation-list` | Enumerate up to 32 u32 tokens. `multiset` emits unique permutations in ascending token order. `johnson_trotter` uses adjacent swaps on distinct tokens, starting in increasing order. The report records the original indexes of each token. |

Partition and permutation listings default to 100 results, with a hard
maximum of 10,000. The work ceiling is 1,048,576. Partition counting is charged
against that ceiling before enumeration. The report carries the exact total
and distinguishes a complete answer from result-limit and work-limit prefixes.
The de Bruijn rotation starts at the first symbol, using the lexicographic
concatenation of Lyndon words whose lengths divide the order.
See [Fredricksen and Maiorana's construction](https://doi.org/10.1016/0012-365X(78)90002-X).

## Exact rational reports

Use integer entries or objects such as
`{"numerator":"-3","denominator":"4"}`. Reduced results have signed 128-bit
numerators and positive unsigned 128-bit denominators. Arithmetic that
exceeds the checked domain returns a resource error.

- `matrix` accepts explicit row and column counts from 0 to 32 and flat
  row-major cells. Use transpose, multiplication, determinant, reduction,
  solve or inverse. Reduction uses the first nonzero pivot and reports rank
  and pivot columns. Solve distinguishes unique, inconsistent and
  underdetermined systems. The empty square matrix has determinant one and an
  empty inverse. A singular square matrix reports `singular: true`.
- `interpolation` returns a forward difference table or evaluates an exact
  Lagrange or Newton interpolant. Use 1 to 64 samples with distinct rational
  x coordinates and 1 to 256 explicit evaluation points. Set `coefficients`
  to get the power basis, ordered from constant term to highest power.
  The difference table of an empty sequence is empty.
  See [NIST DLMF §3.3](https://dlmf.nist.gov/3.3).

## Statistical evidence

| Command | Report and assumptions |
| --- | --- |
| `entropy` | Estimate bits from at most 10,000 unsigned counts and a positive total. Shannon and Miller–Madow omit zero bins. Rényi accepts α in [0.5, 16], uses Shannon at exactly 1, and requires other α values to differ from 1 by at least 10⁻⁶. |
| `correlation` | Request up to 256 ordered lags on vectors of up to 65,536 finite samples. Lag k pairs x[i] with y[i+k] over the shared range. Choose raw products (default), population covariance or Pearson. Empty overlaps fail. Normalized zero-variance overlaps return null. Scalar Pearson and Spearman require equal lengths of at least two, with average ranks for Spearman ties. |
| `runs` | Count up to 1,048,576 categories or classify finite samples against an explicit threshold. The default tie policy is `error`. Use `drop` to omit ties and count them. The flag `large_sample` requires more than ten observations in each category. The z score is null when its variance is zero or undefined. |
| `benford` | Count the first significant digit of each signed decimal integer string and report the nine theoretical weights log₁₀(1+1/d). Zero is counted separately. At most 65,536 values, 4,096 bytes each and 1 MiB of value text. |
| `markov` | Count adjacent transitions for up to 256 distinct ordered state IDs and 65,536 observations. Declare a single sequence or an array of independent sequences. Zero-total rows have null probabilities. Other row probabilities are reduced exact fractions. |

Floating reports use absolute or relative tolerance 10⁻¹⁰ for correlation
and 10⁻¹² for entropy, runs and Benford weights. The runs statistic uses
[NIST's Wald–Wolfowitz formulas](https://www.itl.nist.gov/div898/handbook/eda/section3/eda35d.htm).
JavaScript and Play can enforce smaller host budgets. Read the active
`cube.limits.direct` and the [resource limits guide](../../concepts/resource-limits.md)
when choosing input sizes.

## Letter values

Use `letter-values` with `action: "table"` to retrieve a preset and its digest.
For `action: "evaluate"`, set `mapping` to a preset or a custom array of
`[scalar, u64]` pairs. The report records the original scalar and byte offset
for each input, including omissions. Matching defaults to exact Unicode
scalars and errors on unmapped input. Choose `case: "ascii_upper"` for ASCII
folding or `unmapped: "skip"` to keep an unmapped record with a null value.

The sum uses a checked u64 and optional `max_sum`. Choose `product` or
`positional_weighted_sum` for a checked u128 aggregate. Weights default to
the original one-based scalar positions. Explicit weights give one u64 per
original scalar, including omissions. An empty mapped product is one and an
empty mapped sum is zero. Set `reduce` to include the aggregate's decimal
digital root. The input ceiling is 1 MiB and 65,536 scalars. Custom tables
allow up to 4,096 entries and 64 KiB. Reports are capped at 8 MiB.

| Preset | Mapping |
| --- | --- |
| `ordinal`, `reverse_ordinal` | ASCII A–Z with values 1–26 or 26–1 |
| `reduction`, `reverse_reduction` | Decimal roots of those ordinal values. `pythagorean` and `full_reduction` are aliases of `reduction`. |
| `chaldean_cheiro` | Exact ASCII A–Z table from Cheiro's book, chapter XII, printed p. 70. Accept as a published Latin-letter profile. |
| `hebrew_ordinary`, `hebrew_final_500` | 22 Hebrew letters with the five finals mapped to their base values or to 500 through 900. Accept both explicitly named variants. |
| `abjad_eastern`, `abjad_maghrebi` | 28 Arabic letters in the scientific and archaic orders documented by Thomann. Accept both explicitly named variants. |

Niqqud, Arabic combining marks, hamza variants, presentation forms and
letters outside these tables need an explicit custom mapping or omission
policy. No NFC, transliteration or contextual substitution runs implicitly.
Table digests use SHA-256 over the ascending scalar rows, each encoded as
four big-endian bytes for the scalar followed by eight big-endian bytes for
its value.

The presets contain scalar-to-integer facts transcribed from [Cheiro, Herbert Jenkins, seventh impression](https://archive.org/details/in.ernet.dli.2015.70770),
[National Library of Israel, Hebrew Codicology, numeral notation](https://web.nli.org.il/sites/NLI/English/collections/manuscripts/hebrewcodicology/Documents/HC%20ENGLISH%20ACCUMULATED%201-5%2019.7.17%20%28Autosaved%29.pdf)
and [Thomann, SCIAMVS 19, pp. 182–183](https://www.sciamvs.org/files/SCIAMVS_19_167-200_Thomann.pdf).
Scalar identities follow the [Hebrew chart](https://www.unicode.org/charts/PDF/U0590.pdf)
and [Arabic chart](https://www.unicode.org/charts/PDF/U0600.pdf) in Unicode 17.0.
Verification examples include Cheiro's printed `JOHN` = 18 and `SMITH` = 17,
`שלום` = 376 or 936 by Hebrew variant, and `سضغ` = 1,860 or 1,290 by
Abjad order. The implementation ships the factual mappings and original tests.
