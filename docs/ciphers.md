# Explicit historical cipher profiles

Use explicit keys and alphabets to reproduce historical ciphers for study and
challenges. These profiles provide no modern confidentiality or authentication.
Each invocation starts from supplied parameters, without global state or randomness.
The generated [reference](reference/operations.md) contains versioned schemas
and named input/output ports.

## Calling an operation

Letter, square and matrix cipher commands accept `--parameters PATH`, exact text
from a positional argument, `--text`, `--file` or stdin, and `--decode` for the inverse. The parameter
file is local, regular, UTF-8 JSON, at most 2 MiB; duplicate keys, excessive nesting
and unknown fields reject. It is never downloaded or modified. Porta and Portax
are reciprocal, so either direction performs the same transformation.

In a scratch directory, save a key and check the round trip:

```sh
printf '%s' '{"key":"9321492"}' > gronsfeld.json
cube cipher gronsfeld --parameters gronsfeld.json 'THISONE'
# CKKTSWG
cube cipher gronsfeld --parameters gronsfeld.json --decode 'CKKTSWG'
# THISONE
```

Native Python functions accept `(text, parameters)`; see the
[SDK signatures](../bindings/python/python/cube/_native.pyi).

String operations return strings. Playfair/Seriated Playfair return text,
prepared and fillers; Hill returns text and padding_count; Digrafid returns text,
prepared and padding_count. CLI prints reports as JSON. Play exposes the same
fields as separate named ports. Decode never guesses padding provenance.

Errors have stable categories and safe parameter/offset context, without rejected
user text or key values. Solitaire deck/passphrase and results are private:
automatic history capture stays disabled and private exports use the existing
explicit flow. These profiles have no pipeline adapters. See
[operations, pipelines and Play](concepts/data.md#operations-pipelines-and-play)
for availability and execution permissions.

## Letter-preserving schedules

These profiles accept <=1 MiB UTF-8, transform ASCII letters preserving case,
and preserve every other scalar exactly, including NUL and combining marks.
Only transformed letters advance the key. Indexes are zero-based modulo 26.

| Family / Python stem | Required parameters |
| --- | --- |
| gronsfeld | Key: 1 through 65,536 decimal digits |
| trithemius | `start`, `step`: signed i32 |
| autokey | Seed: 1 through 65,536 ASCII letters; feedback: plaintext or ciphertext |
| running_key | Key: 0 through 1,048,576 ASCII letters |
| porta | Key: 1 through 65,536 ASCII letters |
| interrupted_key | ASCII-letter `key` (1–65,536 bytes), explicit `resets` list |
| progressive_key | ASCII-letter `key` (1–65,536 bytes), signed i32 `increment` |
| vigenere_multiplicative | ASCII-letter `key` (1–65,536 bytes), explicit unit `multiplier` |

Gronsfeld adds repeated key digits and subtracts them for decode.
Trithemius adds `start+step*i`, reducing signed endpoints before multiplication.

Autokey uses Vigenère addition with the seed, followed by the selected plaintext
or ciphertext feedback. State is held in a seed-length ring.

Running Key uses material once and rejects insufficient key letters. An empty
key works only when no ASCII letters need a key. No book is retrieved.

Porta uses the reciprocal ACA 13 tableaux: AB/CD/.../YZ select the tableau,
exchanging A–M and cyclic N–Z.

For Variant Beaufort, Vigenère decode computes C=P-K; Vigenère encode
is its inverse, using the same advancement and preservation.

### Interrupted Key

`cube cipher interrupted-key` restarts Vigenère's keyword before each offset in
`resets`. Offsets count only ASCII letters from zero; spaces, punctuation, NUL
and other Unicode scalars neither consume key letters nor affect the offsets.
The list is required, strictly increasing, and contains at most 65,536 entries.
Each offset must address an existing input letter. Zero is allowed; an empty
list performs ordinary repeated-key Vigenère. Text with no ASCII letters requires
an empty list, and even empty text requires a valid key.

This implements the deterministic transform described by the
[ACA Interrupted Key sheet](https://www.cryptogram.org/downloads/aca.info/ciphers/InterruptedKey.pdf).
The caller supplies the interruption schedule; the operation does not infer word
boundaries, generate random interruptions or require a full keyword occurrence.
Decode uses the same key and offsets.

### Progressive Key

`cube cipher progressive-key` adds an explicit progression to repeated-key
Vigenère. For ASCII-letter offset `i`, key length `L` and `A=0`:

```text
C[i] = (P[i] + K[i mod L] + increment * floor(i/L)) mod 26
```

The first group adds zero. Increment 1 adds A, B, C, … to successive groups;
increment 2 adds A, C, E, …. Zero and negative increments are supported, including
the full signed i32 range; reduction modulo 26 precedes multiplication.
The last partial group uses its actual letters. Decode subtracts both shifts.
The convention follows the
[ACA Progressive Key sheet](https://www.cryptogram.org/downloads/aca.info/ciphers/ProgressiveKey.pdf).

### Vigenère–Multiplicative

`cube cipher vigenere-multiplicative` fixes the composition from
[Biswas, Sakeef and Siddique, rs-58356/v1, section VI.B](https://doi.org/10.21203/rs.3.rs-58356/v1).
It first adds the repeating letter key, then multiplies each result, with A=0:

```text
C[i] = multiplier * (P[i] + K[i mod L]) mod 26
P[i] = inverse(multiplier) * C[i] - K[i mod L] mod 26
```

The required integer multiplier is one of `1, 3, 5, 7, 9, 11, 15, 17, 19, 21,
23, 25`; other values reject, including for empty text. The additive letter key
and numeric multiplier are separate parameters. With key `MOBILE` and multiplier
7, `IAMSTUDYING` becomes `KUNACMBGLRP`.

This is the specified classical composition, not a claim that every similarly
named variant has the same equation or that the preprint establishes modern
security. It has the same preservation and byte limits as the schedules above.
All three key profiles use linear bounded work, validate parameters before output
and return exactly as many UTF-8 bytes as their input. Key and interruption
storage are bounded separately; no stream state persists between calls.

## Canonical schedules and evolving alphabets

Except Ragbaby, these profiles require uppercase ASCII A–Z. Lowercase,
whitespace and unsupported symbols reject. Default text limit is 1 MiB.

| Profile | Transformation |
| --- | --- |
| Gromark | Mixed alphabet with a running digit chain |
| Periodic Gromark | Keyword-ranked periodic alphabet |
| CONDI | Shift updated from recovered plaintext |
| Nicodemus | Keyed column blocks |
| Quagmire | Explicit straight or mixed alphabets |
| Slidefair | Adjacent pairs in Vigenère geometry |
| Portax | Reciprocal half-block pairs |
| Phillips | Eight square states |
| Chaocipher | Two evolving alphabets |

### Gromark

Supply `alphabet`, an A–Z permutation, and `primer`, exactly five decimal digits.
Shift each straight plaintext index by its running digit, then index the mixed
alphabet. The digit chain is `d[i+5]=(d[i]+d[i+1]) mod 10`, held in a five-digit
ring. Printed primer and check digits are excluded from the text.

### Periodic Gromark

The keyword is at most 64 KiB and must have 2 through 9 distinct letters.
Other widths reject. The period equals the number of distinct keyword letters.

Deduplicate the keyword, append unused A–Z, write rows of that width and read
columns in alphabetical keyword order. Ranking digits seed the chain. Group `g`
adds the mixed-alphabet index of distinct key letter `g mod period` plus each
chain digit. Primer and check digits are excluded from the text.

### CONDI

Supply `alphabet`, an A–Z permutation, and `shift` from 0 through 25.
Shift within the keyed alphabet. The next shift is the recovered plaintext's
one-based keyed-alphabet position modulo 26.

### Nicodemus

The key has 1 through 4,096 ASCII letters. `depth` accepts 1 through 64, default 5.
Write rows of key width, Vigenère-shift columns, then emit depth rows per column
in stable alphabetical key order. Equal letters retain original column order;
short blocks keep their actual lengths.

### Quagmire

Supply `profile` (`i`, `ii`, `iii` or `iv`), `plain_alphabet`, `cipher_alphabet`,
an uppercase `indicator`, and a key of 1 through 65,536 ASCII letters.

| Profile | Alphabet requirement |
| --- | --- |
| I | Straight cipher alphabet |
| II | Straight plain alphabet |
| III | Equal mixed alphabets |
| IV | Distinct mixed alphabets allowed |

Encode as `C[(P(p)-P(indicator)+C(key[i])) mod 26]`.

### Slidefair

The key has 1 through 65,536 ASCII letters and text length must be even.
Use ACA Vigenère two-row geometry, with the bottom row starting at the key letter.
One key letter applies per adjacent pair. Different columns exchange columns;
equal columns move right for encode and left for decode. No padding is added.

### Portax

The key has 1 through 4,096 ASCII letters and text length must be even.
The reciprocal ACA 13-position geometry treats AB/CD/.../YZ as equivalent keys.
Split each `2*key_length` group into top and bottom halves and pair vertically.
A final short group uses initial key letters. No padding is added.

### Phillips

Supply an explicit I/J square and `profile`: `rows`, `columns` or `rows_columns`.
Eight five-letter states move down/right for encode and up/left for decode.
Input is at most 327,680 letters, or 65,536 five-letter groups.

Phillips permutations of original indexes are 01234, 10234, 12034, 12304,
12340, 21340, 23140, 23410. The column profile equals the row profile with
transposed square. The ACA RC combined final line has two typos: its detailed
steps give NPYGP and TCITY, which the geometry and tests follow.

### Chaocipher

Supply A–Z permutations `left` and `right`. Look up plain in right and cipher in
left. Rotate left to the cipher index, then move index 1 to 13. Rotate right to
the plain index plus one, then move index 2 to 13. State resets each call.

### Ragbaby

Ragbaby requires a 24-letter permutation excluding J/X and merge_aliases boolean.
Encode may explicitly merge J→I/X→W; decode never emits J/X. Only supported
uppercase letters transform; nonletters survive. Word and letter counts start at
one; shift is word+letter-1. Separators end words; ASCII hyphens/apostrophes stay
inside words. Lowercase/non-ASCII letters reject. Limits: 1 MiB and 65536 words.

### Solitaire/Pontifex

Solitaire/Pontifex takes <=65536 A–Z letters and deck: permutation 1..54, jokers
A=53/B=54. Original joker moves, triple cut, bottom count cut and output lookup
apply; joker outputs consume another round. passphrase defaults empty; nonempty
ASCII passphrase requires ordered deck and adds a letter-valued count cut after
each keying round. max_steps defaults/caps at 1048576 and includes keying and
rejected outputs. No RNG, automatic X padding or extra final keying step.

## Squares and exact matrices

All 5x5 squares are row-major permutations of ABCDEFGHIKLMNOPQRSTUVWXYZ.
Canonical text already uses this alphabet: J, lowercase and punctuation reject.

### Two-Square and Four-Square

Both require even text of at most 1 MiB and add no fillers.
Two-Square aligns the explicit first and second squares horizontally. For
plaintext positions `(r1,c1)` and `(r2,c2)`, emit `second[r1,c2]` and `first[r2,c1]`.
Decode uses opposite lookup order.

Four-Square places straight plaintext squares top-left/bottom-right and keyed
first/second squares top-right/bottom-left. Emit crossed corners in keyed-square
order.

### Playfair and Seriated Playfair

Supply `square` and two distinct fillers: `filler` defaults to X and `alternate`
to Q. Playfair splits double letters in adjacent digraphs and pads a missing
second letter, using the alternate when the first equals the filler. Standard
right/down/rectangle geometry becomes left/up for the inverse.

Seriated Playfair also takes `period` from 1 through 4,096. Top and bottom rows
pair vertically. Equal vertical letters insert a bottom filler without consuming
the original symbol. Nonfinal groups stay full; the final short group uses the
smallest top row that allows greedy consumption of the suffix.

Playfair encode accepts <=1 MiB, prepared/cipher text <=2 MiB; serialized report
<=16 MiB. fillers are zero-based scalar positions in prepared. Decode returns
prepared text and empty fillers; it cannot distinguish inserted X/Q from originals.
Play wire/result limits may be stricter than direct Core limits.

### Tri-Square

Supply `first`, `second` and `middle` squares. Plaintext must have even length,
at most 65,536 bytes; ciphertext has complete triples, at most 98,304 bytes.
Emit `first[choice_row,c1]`, `middle[r1,c2]`, then `second[r2,choice_column]`.
Optional row/column choice arrays have exactly one value from 0 through 4 per pair;
empty arrays use `pair_index mod 5`, reset each call. Decode ignores those choice
dimensions.

### Hill

Hill uses an exact unique Unicode alphabet: 2..256 scalars; dimension: 2..8; matrix:
dimension² signed i32 entries, row-major and column-vector multiplication.
Reduce entries modulo alphabet length. Exact Bareiss determinants and adjugate
cofactors support composite moduli; determinant must be coprime to modulus even
on encode. Limits: 65536 text/prepared scalars, 64 KiB serialized definition.
padding is null/omitted or {symbol,count}, count smaller than dimension. Encode
adds exactly count symbols and requires full blocks. Decode retains all symbols
and reports zero added padding. No Unicode normalization.

### Digrafid

Digrafid requires horizontal: 3x9, vertical: 9x3, extra symbol, period: 1..4096,
and null/optional padding. Alphabets each permute A–Z plus the same nonletter
extra. Pair triple is (horizontal_column, horizontal_row*3+vertical_column,
vertical_row), each 0..8. Flatten three coordinate rows per period of digraphs,
then read triples to form ciphertext; inverse reverses this even for a short
final period. Odd input requires explicit padding, retained on decode. Input
<=65536 symbols; prepared/ciphertext <=131072 symbols. No I/J merge or stripping.

## Primary sources and reproducibility

ACA sheets define the named profiles; operation metadata links exact sources:
[cipher index](https://www.cryptogram.org/resource-area/cipher-types/).
Quagmire profiles use the individual
[I](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireI.pdf),
[II](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireII.pdf),
[III](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireIII.pdf) and
[IV](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireIV.pdf) sheets.
Phillips RC follows its detailed steps where its final combined line differs:
[RC sheet](https://www.cryptogram.org/downloads/aca.info/ciphers/PhillipsRC.pdf).
Solitaire follows [Schneier v1.2, 1999-05-26](https://www.schneier.com/academic/solitaire/).
Chaocipher follows [Rubin, updated 2010-07-30](https://www.chaocipher.com/ActualChaocipher/Chaocipher-Revealed-Algorithm.pdf).
Hill's source is [Hill 1929, pp. 306–312](https://doi.org/10.1080/00029890.1929.11986963).
See [verification notes](resource-accounting.md#historical-cipher-verification)
for fixture provenance and implementation details.

## Caller-owned book, keyboard and symbolic profiles

### Book coordinates

`cipher.book.encode` accepts exact caller source and plaintext inputs. Rust uses `ciphers::book::encode(source, text, &EncodeOptions)`;
Python uses `book_encode(source, text, parameters)`. The parameters are
`scheme` (`word_character`, `line_character`, `line_word_character`), `index_base`
(0 or 1) and `matching` (`first` or `round_robin`). No implicit source is selected.

The report contains comma-separated coordinates, `source_sha256` of the exact
UTF-8 source, coordinate count and indexed-unit count. Round-robin cycles through
matching coordinates independently for each scalar in source order. The existing
word/line tokenization and decoder are shared: whitespace splits words, line
splitting follows Rust `str::lines`, and character indexes count Unicode scalars.
Word initials use character index 1 (base 1), or the existing text selector.

Unrepresentable scalars fail; repeated input coordinates need not be reproduced
by re-encoding decoded text. Each source/text is at most 1 MiB, with at most
65,536 indexed units and emitted coordinates; reports are at most 4 MiB.

```python
import cube

p = {"scheme": "word_character", "index_base": 1, "matching": "round_robin"}
r = cube.book_encode("aa aa", "aaaaa", p)
assert r["coordinates"] == "1:1,1:2,2:1,2:2,1:1"
assert cube.book_decode("aa aa", r["coordinates"]) == "aaaaa"
```

### Keyboard coordinates

`text.keyboard.coordinates.encode/decode` use an explicit `layout` object with
`geometry` and `entries: [{key, layer, symbol, row, column}]`, plus required
`index_base` 0 or 1. Coordinates are JSON tuples `[row,column,layer]`; layers are
literal IDs, symbols are single exact Unicode scalars. Positions are zero-based
`[byte_offset,scalar_offset]` pairs in the input text for encoding and reconstructed
text for decoding.

`definition_sha256` hashes compact serialized validated layout
values; it does not hash the caller's JSON formatting.

Geometry has 1..512 entries,
rows/columns 0..511 and a 64 KiB serialized ceiling. Key/layer/geometry IDs are
1..128 UTF-8 bytes. Duplicate symbols, key/layer pairs and coordinate/layer pairs
fail; the same physical key must retain its row/column across layers.

### Keyboard shifts

`text.keyboard.shift.encode/decode` take `layout`, signed exact `dx` and `dy`, and
`edge` (`reject`, default, or `wrap`). Positive displacement moves right/down in
the same layer; shift report coordinates always have origin zero. `reject` fails
when a used symbol reaches a missing cell. `wrap` requires contiguous rows and
columns from zero in each layer: move/wrap the row first, then move/wrap in the
destination row's length.

Ragged vertical shifts can collide. Forward reports
`bijective`; reverse rejects every non-total or non-bijective mapping, even for an
empty message. Full i64 displacements are normalized without overflow. Keyboard
text/coordinate transport is at most 1 MiB and 65,536 scalars/coordinates; each
report is at most 4 MiB. No OS keyboard detection or key events are involved.

### Symbol token tables

`symbol.tokens.decode/encode` apply caller-owned exact token tables. Parameters
require `table: {source_id,profile_id,license_id,entries}`, explicit `boundaries`
(`{"kind":"concatenated"}` or `{"kind":"separated","separator":"|"}`), and
`output_separator`. Entries contain `token`, `value`, and optional `aliases`.
Provenance IDs identify caller evidence; they confer no permission or network
lookup. Inputs and tables are never normalized, case-folded or downloaded.

Forward decoding reports each original/canonical token, alias use and byte/scalar
offset, plus text, provenance, definition hash and bijectivity. Encoding requires
unique values and no aliases. Duplicate ownership, unknown tokens and empty
separated records fail. Concatenated input and separator-free output must have
prefix-free alphabets; no greedy tokenization is guessed. Separators must not
occur inside their tokens. Scalar bijections reuse Core's existing alphabet map.

The table is at most 64 KiB, 4,096 canonical-plus-alias forms, 256 UTF-8 bytes per
token/value and 1,024 bytes per provenance ID. Input is at most 1 MiB/65,536 tokens,
output/report at most 4 MiB. These substitution/puzzle profiles provide no secrecy.
No historical/fictional alphabet, font, score or puzzle solution is bundled.

### Parameter transport

Strict parameters reject unknown or duplicate fields. Integer fields accept exact
JSON numbers or canonical decimal strings, preserving full i64 values across
JavaScript transport. Use the generated contracts for exact named ports and
[installed availability](concepts/data.md#operations-pipelines-and-play).
