# Transposition and fractionation profiles

These historical ciphers are for challenge analysis, not modern confidentiality
or authentication. Rust owns each algorithm; CLI, native Python, JavaScript/Wasm and Play call the
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
Duplicate and unknown fields fail. Size/index fields that advertise canonical
decimal strings in their schemas accept that lossless transport; signed Skip
endpoints are never rounded through a JavaScript number. Swagman and Nihilist
Transposition key ranks use ordinary JSON integers.

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
| Cadenus | Canonical uppercase `keyword`; fixed 25-row blocks and stable column order |
| Swagman | `key`: a 4..8 square of one-based Latin ranks; periodic column routing |
| Nihilist Transposition | One-based `key` of side 1..10; required `convention` (`writing_in`/`taking_out`) and `readout` (`rows`/`columns`) |
| Ubchi | Uppercase `key_phrase` words separated by one ASCII space; exact caller `nulls`, one scalar per word |
| Route | `rows`, `columns`, `fill`, `read`; each route has `kind` and `corner` |
| Spiral | Same route DTO, requiring a spiral read route |
| Caesar box | Explicit `side`; full square, top-left row fill/column read |
| Turning grille | Even `side` 2..64, sorted row-major `holes`, explicit `orientations` permutation of quarter-turns 0..3 |
| Cardan extract | `rows`, `columns`, sorted `holes`; selects from caller cover text; source-dependent |
| Cardan overlay | Same mask plus exact `replacements`; returns `text`, `positions`, `original_selected` |

### ACA table permutations

Cadenus labels rows `AZYXVUTSRQPONMLKJIHGFEDCB`; `W` shares the rotation of
`V`. Each original column starts at its key-letter row, columns take stable
keyword order, and ciphertext is read by rows. Plaintext scalars are never
merged or case-folded. Each block contains `25 × keyword length` cells. The
keyword ceiling is 4,096 ASCII letters, and the complete block must also fit
65,536 cells; widths above 2,621 exceed that geometry even for empty input.

Swagman requires every key row and column to permute ranks `1..height`. Source
text fills a rectangle with that height. Each column routes through the
corresponding periodic key column; ciphertext is read down columns. The last
key period may be short. Nihilist Transposition applies the same key to both
axes of each full square. A writing-in key assigns source positions to key
destinations; a taking-out key visits source positions in key order. These
conventions are inverses and can produce different answers.

All three return `text`, `original_length` and `padding_count`. Encoding a ragged
block requires an explicit `padding` scalar and adds only the minimum number
of copies. Decoding preserves the complete text when `original_length` is
omitted. To remove padding, supply that length and the exact pad scalar; the
length must describe the minimum complete block and every declared suffix
scalar must match. Legitimate occurrences of the pad inside the source remain
data. The aggregate 65,536-cell ceiling includes padding. Swagman completes a
multiple of its height; Cadenus and Nihilist complete their respective blocks.

The [Cadenus](https://www.cryptogram.org/downloads/aca.info/ciphers/Cadenus.pdf),
[Swagman](https://www.cryptogram.org/downloads/aca.info/ciphers/Swagman.pdf) and
[Nihilist Transposition](https://www.cryptogram.org/downloads/aca.info/ciphers/NihilistTransposition.pdf)
references contain the source examples and table conventions. Cadenus uses the
same 25-row schedule beyond the ACA puzzle recommendation of six key letters,
within the explicit ceilings above.

### Ubchi null evidence

Ubchi uses the same stable Columnar key twice. Remove the declared spaces from
`key_phrase` to rank its letters, retaining ties in original order. Its actual
word count determines the number of caller-supplied null scalars. These are
inserted after the first pass. Decode reverses the second pass, validates and
removes only that exact suffix, then reverses the first pass.

The report contains `text`, the original scalar length and `null_count`. No
plaintext normalization, automatic null choice or numeric-key word-count guess
occurs. The phrase has at most 4,096 letters and 8,191 ASCII bytes; `nulls` has
exactly one scalar per word, at most 4,096 scalars/16,384 UTF-8 bytes. Source
plus nulls must fit 65,536 scalars and 1 MiB. This is the two-pass profile in
[LANAKI Lecture 24, pages 1–2](https://www.cryptogram.org/downloads/LANAKI%20Crypto%20Lessons/LESSON24.pdf).

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

The coordinate profiles below require exact canonical text;
no whitespace/punctuation stripping. All are bounded by 65536 source symbols,
with 1 MiB input/output ceilings (ASCII profiles are therefore at most 65536
bytes). Alphabets/squares are explicit caller permutations.

| Profile | Required typed parameters and convention |
| --- | --- |
| Bifid | `square` (25 I/J cells), `period` 1..4096; rows then columns per period |
| CM Bifid | Independent `input_square` and `output_square` (25 I/J cells each), `period` 1..4096 |
| Trifid | `alphabet` (27 unique exact scalars), `period`; layer/row/column in row-major 3³ cube |
| ADFGX | `square` (25 I/J cells), `key`; labels ADFGX, row before column, then incomplete Columnar |
| ADFGVX | `square` (36 A-Z/0-9 cells), `key`; labels ADFGVX, J is independent |
| Nihilist Substitution | `square`, nonempty canonical `key` <=4096; ordinary coordinate addition 22..110, no modulo 100 |
| Bazeries | Nonzero decimal `key` 1..999999 without zero digits; explicit 25-letter `square` |

Bifid/CM Bifid/Nihilist/Bazeries merge J→I and require uppercase A-Z. Trifid matches exact
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

CM Bifid reads plaintext coordinates in the first square, joins row then column
streams per actual period, and looks up the regrouped pairs in the second.
Decode takes coordinates in the second square and inverts that stream before
looking up the first. Encoding with swapped squares does not perform this
inverse. The [ACA CM Bifid example](https://www.cryptogram.org/downloads/aca.info/ciphers/CMBifid.pdf)
uses period seven, including a shorter final group. Both squares require 25
unique canonical I/J letters; equal squares reduce to ordinary Bifid.

## VIC English 2010

`cube cipher vic` implements the fixed literal English profile in the
[complete worked construction dated 31 May 2010](https://raincomplex.net/writing/title/vic-cipher/).
It derives a checkerboard and two transposition keys, applies ordinary then
disrupted transposition, and inserts a supplied five-digit indicator. The
mechanisms follow [Kahn's VIC description](https://www.cia.gov/resources/csi/static/Number-One-From-Moscow.pdf);
the English table and five-row derivation are the selected profile.

Plaintext uses exact uppercase A–Z and periods. Spaces, lowercase, numerals and
other symbols reject without implicit preparation. Ciphertext is ungrouped ASCII
digits in complete five-digit groups. The profile has no Cyrillic or numeral
mode. All key fields are required: `phrase` is exactly 20 uppercase letters,
`mnemonic` contains eight distinct uppercase letters, and `date_digits` contains
the exact six source-profile digits. No calendar format or transliteration is
inferred. `personal_number` is 1..24; the two derived widths together must fit
the fixed 50-digit block. Numeric ranking treats zero as the highest digit and
resolves ties left to right.

Retain the exact `split_at` plaintext position for both directions. Encode swaps
the two pieces and inserts the two-period start marker between them. Decode
uses that retained position, so literal runs of periods remain exact data.
Supply `padding` as zero to four exact digits, just enough to complete the
minimum five-digit group after checkerboard substitution. Decode validates and
removes exactly those digits; it never searches for or guesses padding.

`message_indicator` is exactly five digits, including leading zeros. The last
date digit selects its group counting from the end, including the indicator
itself; this API explicitly defines zero as the tenth group. Short frames that
cannot contain the selected position reject. Decode requires the inserted group
to match the supplied indicator before accepting the inverse.

| Resource | Maximum |
| --- | --- |
| Plaintext | 65,534 ASCII symbols/bytes |
| Checkerboard text including the start marker | 65,536 symbols |
| Ciphertext including pad and indicator | 131,080 ASCII digits |
| Key derivation | Fixed 50-digit block; combined transposition widths at most 50 |

The result has the named `text` port. Key fields are sensitive; the canonical
contract defines capture and export policy. There is no automatic randomness
or acquisition, and no derived-key trace is returned. See
[Core and interface limits](concepts/resource-limits.md#core-and-interface-limits)
for lower host budgets.

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
