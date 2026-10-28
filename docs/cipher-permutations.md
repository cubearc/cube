# Transposition and fractionation profiles

These historical ciphers are for challenge analysis, not modern confidentiality
or authentication. Rust owns each algorithm; CLI, native Python and Play call the
same typed functions. The [generated contracts](reference/operations.md) contain
exact named ports, schemas, errors and limits. These profiles have no pipeline
adapters; see [operation availability](concepts/data.md#operations-pipelines-and-play).

## Exact transpositions

All functions in `cube_core::ciphers::transposition` preserve Unicode scalars,
including whitespace, combining marks and NUL, without normalization. Functions
accept `(text, &parameters, decode)` except the two Cardan source operations.

### Input and output

Input is at most 65,536 scalars and 1 MiB; columns/rails/keywords at most 4,096.
Outputs preserve the exact scalar multiset and UTF-8 byte count unless explicit
padding or Cardan overlay/selection changes the source.

Parameters use strict JSON transport: at most 2 MiB, depth 8 and 200,000 nodes.
Duplicate and unknown fields fail. Integer fields and index/token lists also accept
canonical decimal strings as advertised by the lossless catalog transport; signed
Skip endpoints are never rounded through a JavaScript number.

### Column keys

A column key is either `{"kind":"keyword","text":"BALLOON"}` or
`{"kind":"permutation","order":[1,0,2,3,6,4,5]}`. Keyword sorting uses Unicode
scalar order then original position. Permutations are zero-based output-to-input
orders, with every position exactly once.

### Profile parameters

CLI commands use the hyphenated profile name, for example
`cube cipher sequence-transposition`. Parameter names below use their exact API
spelling.

| Profile | Exact parameter convention |
| --- | --- |
| Scytale | `columns`; row fill, column read, ragged last row, no padding |
| Rail fence | `rails`, `initial_rail`, `direction` (`up`/`down`); reflect at endpoints before advancing; rails=1 is identity |
| Redefence | Same rail layout plus explicit `order` of rail readout |
| Sequence transposition | `order` is a full permutation of scalar positions; empty text requires empty order |
| Skip | Signed 64-bit `start`, `step`; positions `(start+i*step) mod n`; require coprime step for n>1; empty/singleton accept every signed field |
| Columnar | `key`; incomplete ragged columns; lengths follow the original length |
| Complete columnar | `key`, optional `padding` scalar, optional `original_length`; report `text`, `original_length`, `padding_count` |
| Double columnar | Independent `first`, `second` column keys; incomplete passes; decode reverses pass order |
| Myszkowski | `keyword`; tied ranks read across columns row by row; singleton ranks down columns |
| Amsco | `key`, `initial_cell` (default 1), required `pattern` (`aca` or `continuous`); only the last cell truncates |
| Route | `rows`, `columns`, `fill`, `read`; each route has `kind` and `corner` |
| Spiral | Same route DTO, requiring a spiral read route |
| Caesar box | Explicit `side`; full square, top-left row fill/column read |
| Turning grille | Even `side` 2..64, sorted row-major `holes`, explicit `orientations` permutation of quarter-turns 0..3 |
| Cardan extract | `rows`, `columns`, sorted `holes`; selects from caller cover text; source-dependent |
| Cardan overlay | Same mask plus exact `replacements`; returns `text`, `positions`, `original_selected` |

### Routes and grilles

Route kinds are `rows`, `columns`, `clockwise`, `counterclockwise`; corners are
`top_left`, `top_right`, `bottom_right`, `bottom_left`. Row/column routes reflect
both axes to start at that corner. Spirals start along the boundary in the chosen
winding direction and turn inward; each cell is visited exactly once. Rectangles
must be full and shape multiplication is checked. Padding is caller supplied.
Turning Grille fills holes sorted in each explicit orientation and reads rows;
rotated holes must cover the entire square exactly once. Aggregate block cells
cannot exceed 65536. Cardan overlays retain evidence and do not claim an inverse.

### Padding and AMSCO cells

Complete Columnar never guesses padding. Encode needs a pad scalar for an
incomplete rectangle. Decode without `original_length` preserves the padded text;
with a length, it validates the minimal rectangular pad count and every trailing
pad scalar. Existing traversal extraction/Josephus operations keep their previous
semantics; Skip here is a bijection without removal.

The ACA AMSCO profile alternates the first cell of every row as well as every
cell within a row. `continuous` alternates across the complete cell stream.
They differ at even widths. Both contracts are deterministic and explicit.

```python
import cube

p = {"rails": 3, "initial_rail": 0, "direction": "down"}
assert cube.rail_fence_encode("WEAREDISCOVEREDFLEEATONCE", p) == "WECRLTEERDSOEEFEAOCAIVDEN"
assert cube.skip_encode("ABCDE", {"start": 1, "step": 2}) == "BDACE"
```

CLI example: save `{"columns":3}` to a local parameter file, then run
`cube cipher scytale --text ABCDEFGH --parameters parameters.json` → `ADGBEHCF`.
Add `--decode` to reverse. Cardan commands reject `--decode`.

## Coordinates and hybrids

Functions in `cube_core::ciphers::fractionation` require exact canonical text;
no whitespace/punctuation stripping. All are bounded by 65536 source symbols,
with 1 MiB input/output ceilings (ASCII profiles are therefore at most 65536
bytes). Alphabets/squares are explicit caller permutations.

| Profile | Required typed parameters and convention |
| --- | --- |
| Bifid | `square` (25 I/J cells), `period` 1..4096; rows then columns per period |
| Trifid | `alphabet` (27 unique exact scalars), `period`; layer/row/column in row-major 3³ cube |
| ADFGX | `square` (25 I/J cells), `key`; labels ADFGX, row before column, then incomplete Columnar |
| ADFGVX | `square` (36 A-Z/0-9 cells), `key`; labels ADFGVX, J is independent |
| Nihilist Substitution | `square`, nonempty canonical `key` <=4096; ordinary coordinate addition 22..110, no modulo 100 |
| Bazeries | Nonzero decimal `key` 1..999999 without zero digits; explicit 25-letter `square` |

Bifid/Nihilist/Bazeries merge J→I and require uppercase A-Z. Trifid matches exact
scalars including its 27th symbol. Final fractionation periods use their actual
length. ADFG* permits at most 131072 ciphertext labels, requires even length and
rejects unknown labels. It reuses the validated Polybius square and concrete
column schedule. Nihilist encode returns a list of integers; decode takes that
list, validating each difference as an actual 11..55 coordinate. CLI decode input
is the JSON token list; no concatenated decimal guessing.

Bazeries uses the ACA square orientation: the ordinary I/J alphabet goes down
plaintext columns and the explicit ciphertext square across rows. Reverse
successive groups in repeating key-digit sizes (short final group allowed),
then substitute; decode substitutes first then reverses. There is no implicit
English number spelling. For ACA key 3752 the explicitly supplied square is
`THREOUSANDVFIYWBCGKLMPQXZ`; this reproduces the published answer.

## Morse fractionation

Fractionated Morse, Morbit and Pollux require uppercase A-Z/0-9 with single
interior ASCII word spaces. Morse conversion reuses the existing International
Morse codec. Streams use dot/dash/x, one x between letters and xx between words;
no terminal pause is assumed. Source symbols <=65536, Morse stream <=1 MiB,
ciphertext <=65536 symbols. No random homophone selection occurs.

Fractionated Morse takes an explicit A-Z `alphabet`, indexed by lexicographic
triples over dot,dash,x excluding xxx. Morbit takes a permutation of digits 1..9
in `digits` indexed by all nine pairs. Encode appends the minimum x padding to a
full group. Decode requires explicit optional `padding_count` evidence (0..2 for
triples, 0..1 for pairs) to return plaintext. If omitted, it returns `text:null`
and `padding_ambiguous:true` while preserving the exact stream. A wrong count,
invalid grammar, invalid table or unsupported symbol fails predictably.

Pollux takes ten `classes` symbols for digits 0..9. Each is dot, dash or x and all
three classes must exist. Encoding uses independent round-robin counters over
each class's digits in ascending order. Decode accepts any owned digit, with no
terminal padding. All Morse reports expose separate `text`, `ciphertext`, `morse`,
`padding_count`, `padding_ambiguous` ports, preserving exact digit/pad evidence.

```python
p = {"alphabet": "ROUNDTABLECFGHIJKMPQSVWXYZ"}
r = cube.fractionated_morse_encode("COME AT ONCE", p)
assert r["ciphertext"] == "CBIILTMHVVFL"
assert cube.fractionated_morse_decode(r["ciphertext"], {**p, "padding_count": r["padding_count"]})["text"] == "COME AT ONCE"
```

Reference profiles were checked against the [ACA cipher specification index](https://www.cryptogram.org/resource-area/cipher-types/)
and [CrypTool ADFG(V)X](https://legacy.cryptool.org/en/cto/adfg-v-x), reviewed
2026-10-27. Each canonical operation links its exact source. See [fractionation verification notes](resource-accounting.md#transposition-and-fractionation-verification)
for cross-interface fixture coverage.
The Nihilist ACA vector is deliberately qualified to ordinary sums: the source's
printed 08 is 108 in this profile. No modulo-100 variant is advertised.
