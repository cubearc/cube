# Explicit historical cipher profiles

These operations are for historical study and challenges, with no modern
confidentiality or authentication. Algorithms and validation live in
cube_core::ciphers::poly. CLI, native Python and Play call those APIs; each
invocation starts from supplied parameters, with no global state or RNG.
The generated [reference](reference/operations.md) contains versioned schemas
and named input/output ports.

## Calling an operation

Every new CLI family accepts --parameters PATH, exact text from a positional
argument, --text, --file or stdin, and --decode for the inverse. The parameter
file is local, regular, UTF-8 JSON, at most 2 MiB; duplicate keys, excessive nesting
and unknown fields reject. It is never downloaded or modified. Porta and Portax
are reciprocal, so either direction performs the same transformation.

    printf '%s' '{"key":"9321492"}' > /tmp/gronsfeld.json
    cube cipher gronsfeld --parameters /tmp/gronsfeld.json 'THISONE'
    # CKKTSWG
    cube cipher gronsfeld --parameters /tmp/gronsfeld.json --decode 'CKKTSWG'
    # THISONE

Python uses (text, parameters), with parameter dictionaries described in the
native stubs:

    import cube
    assert cube.gronsfeld_encode("THISONE", {"key": "9321492"}) == "CKKTSWG"
    assert cube.hill_encode("HELP", {
        "alphabet": "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "dimension": 2,
        "matrix": [3, 3, 2, 5], "padding": None,
    }) == {"text": "HIAT", "padding_count": 0}

String operations return strings. Playfair/Seriated Playfair return text,
prepared and fillers; Hill returns text and padding_count; Digrafid returns text,
prepared and padding_count. CLI prints reports as JSON. Play exposes the same
fields as separate named ports. Decode never guesses padding provenance.

Errors have stable categories and safe parameter/offset context, without rejected
user text or key values. Solitaire deck/passphrase and results are private:
automatic history capture stays disabled and private exports use the existing
explicit flow. No registration grants filesystem/network access or a pipeline
adapter. The existing 53 pipeline adapters are unchanged.

## Letter-preserving schedules

These profiles accept <=1 MiB UTF-8, transform ASCII letters preserving case,
and preserve every other scalar exactly, including NUL and combining marks.
Only transformed letters advance the key. Indexes are zero-based modulo 26.

| Family / Python stem | Required parameters | Schedule |
| --- | --- | --- |
| gronsfeld | key: 1..65536 decimal digits | Add repeated digits, subtract for decode. |
| trithemius | start, step: signed i32 | Add start+step*i, reducing signed endpoints before multiplication. |
| autokey | seed: 1..65536 ASCII letters; feedback: plaintext/ciphertext | Vigenère addition, seed then the selected feedback. A seed-length ring retains state. |
| running_key | key: 0..1048576 ASCII letters | Vigenère addition using material once; reject insufficient letters. Empty key works only when no ASCII letters need one. No book retrieval. |
| porta | key: 1..65536 ASCII letters | Reciprocal ACA 13 tableaux, AB/CD/.../YZ selection exchanging A–M and cyclic N–Z. |

Variant Beaufort already exists: Vigenère decode computes C=P-K; Vigenère encode
is its inverse, using the same advancement and preservation.

## Canonical schedules and evolving alphabets

Except Ragbaby, these profiles require uppercase ASCII A–Z. Lowercase,
whitespace and unsupported symbols reject. Default text limit is 1 MiB.

| Family | Explicit parameters and transformation |
| --- | --- |
| gromark | alphabet: A–Z permutation; primer: exactly five decimal digits. Shift straight plaintext index by running digit, then index mixed alphabet. Recurrence d[i+5]=(d[i]+d[i+1]) mod 10 in a five-digit ring. Printed primer/check digit are excluded from text. |
| periodic_gromark | keyword <=64 KiB; period equals distinct keyword length. Deduplicate, append unused A–Z, write rows of that width and read columns in alphabetical keyword order. Ranking digits seed the chain; group g adds mixed-alphabet index of distinct key letter g mod period plus each chain digit. No primer/check digit in text. Qualified decimal-ranking profile: 2..9 distinct letters; other widths reject rather than inventing a schedule. |
| condi | alphabet: A–Z permutation; shift: 0..25. Shift within keyed alphabet; next shift is recovered plaintext's one-based keyed-alphabet position modulo 26. |
| nicodemus | key: 1..4096 ASCII letters; depth: 1..64, default 5. Write rows of key width, Vigenère-shift columns, emit depth rows per column in stable alphabetical key order. Equal letters retain original column order; short blocks keep their actual lengths. |
| quagmire | profile i/ii/iii/iv; plain_alphabet, cipher_alphabet; uppercase indicator; key: 1..65536 ASCII letters. I requires straight cipher alphabet, II straight plain alphabet, III equal mixed alphabets, IV allows distinct alphabets. Encode C[(P(p)-P(indicator)+C(key[i])) mod 26]. |
| slidefair | key: 1..65536 ASCII letters; even text. ACA Vigenère two-row geometry, bottom row starts at key letter; one key letter per adjacent pair. Different columns exchange columns; equal columns move right for encode, left for decode. No padding. |
| portax | key: 1..4096 ASCII letters; even text. Reciprocal ACA 13-position geometry with AB/CD/.../YZ key equivalence. Split 2*key_length groups into top/bottom halves and pair vertically; final short group uses initial key letters. No padding. |
| phillips | explicit I/J square; profile rows/columns/rows_columns. Eight five-letter states; down/right encode, up/left decode. <=327680 letters, or 65536 five-letter groups. |
| chaocipher | explicit A–Z left/right permutations. Look up plain in right, cipher in left. Rotate left to cipher index, move index 1 to 13; rotate right to plain index+1, move index 2 to 13. Reset each call. |

Phillips permutations of original indexes are 01234, 10234, 12034, 12304,
12340, 21340, 23140, 23410. The column profile equals the row profile with
transposed square. The ACA RC combined final line has two typos: its detailed
steps give NPYGP and TCITY, which the geometry and tests follow.

Ragbaby requires a 24-letter permutation excluding J/X and merge_aliases boolean.
Encode may explicitly merge J→I/X→W; decode never emits J/X. Only supported
uppercase letters transform; nonletters survive. Word and letter counts start at
one; shift is word+letter-1. Separators end words; ASCII hyphens/apostrophes stay
inside words. Lowercase/non-ASCII letters reject. Limits: 1 MiB and 65536 words.

Solitaire/Pontifex takes <=65536 A–Z letters and deck: permutation 1..54, jokers
A=53/B=54. Original joker moves, triple cut, bottom count cut and output lookup
apply; joker outputs consume another round. passphrase defaults empty; nonempty
ASCII passphrase requires ordered deck and adds a letter-valued count cut after
each keying round. max_steps defaults/caps at 1048576 and includes keying and
rejected outputs. No RNG, automatic X padding or extra final keying step.

## Squares and exact matrices

All 5x5 squares are row-major permutations of ABCDEFGHIKLMNOPQRSTUVWXYZ.
Canonical text already uses this alphabet: J, lowercase and punctuation reject.

| Family | Geometry and preparation |
| --- | --- |
| two_square | Even text <=1 MiB. Explicit first/second squares horizontally aligned. For plaintext positions (r1,c1)/(r2,c2), emit second[r1,c2], first[r2,c1]. Decode uses opposite lookup order. No fillers. |
| four_square | Even text <=1 MiB. Straight plaintext squares top-left/bottom-right; keyed first top-right, second bottom-left. Emit crossed corners in keyed-square order. No fillers. |
| playfair | square; filler default X, alternate default Q, distinct. Adjacent digraphs split doubles/pad missing second letter; use alternate when first equals filler. Standard right/down/rectangle geometry, inverse left/up. |
| seriated_playfair | Same filler fields; period: 1..4096. Top/bottom rows paired vertically. Equal vertical letters insert bottom filler without consuming original symbol. Nonfinal groups stay full; final short group uses smallest top row allowing greedy consumption of suffix. Forward scan plus bounded binary search over <=4096 tail positions; no quadratic whole-message insertion. |
| tri_square | first/second/middle squares; even plaintext <=65536 bytes, complete ciphertext triples <=98304 bytes. Emit first[choice_row,c1], middle[r1,c2], second[r2,choice_column]. Optional rows/columns arrays: exactly one 0..4 per pair, or empty for pair_index mod 5, reset per call. Decode ignores random-choice dimensions. |

Playfair encode accepts <=1 MiB, prepared/cipher text <=2 MiB; serialized report
<=16 MiB. fillers are zero-based scalar positions in prepared. Decode returns
prepared text and empty fillers; it cannot distinguish inserted X/Q from originals.
Play wire/result limits may be stricter than direct Core limits.

Hill uses exact unique Unicode alphabet: 2..256 scalars; dimension: 2..8; matrix:
dimension² signed i32 entries, row-major and column-vector multiplication.
Reduce entries modulo alphabet length. Exact Bareiss determinants and adjugate
cofactors support composite moduli; determinant must be coprime to modulus even
on encode. Limits: 65536 text/prepared scalars, 64 KiB serialized definition.
padding is null/omitted or {symbol,count}, count smaller than dimension. Encode
adds exactly count symbols and requires full blocks. Decode retains all symbols
and reports zero added padding. No Unicode normalization.

Digrafid requires horizontal: 3x9, vertical: 9x3, extra symbol, period: 1..4096,
and null/optional padding. Alphabets each permute A–Z plus the same nonletter
extra. Pair triple is (horizontal_column, horizontal_row*3+vertical_column,
vertical_row), each 0..8. Flatten three coordinate rows per period of digraphs,
then read triples to form ciphertext; inverse reverses this even for a short
final period. Odd input requires explicit padding, retained on decode. Input
<=65536 symbols; prepared/ciphertext <=131072 symbols. No I/J merge or stripping.

## Primary sources and reproducibility

Real contract fixtures compare independent known answers. ACA sheets specify
the named profiles, with exact per-operation URLs in metadata:
[cipher index](https://www.cryptogram.org/resource-area/cipher-types/).
Quagmire alternate-profile tests use the individual
[I](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireI.pdf),
[II](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireII.pdf),
[III](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireIII.pdf) and
[IV](https://www.cryptogram.org/downloads/aca.info/ciphers/QuagmireIV.pdf) sheets.
Phillips RC follows its detailed steps where its final combined line differs:
[RC sheet](https://www.cryptogram.org/downloads/aca.info/ciphers/PhillipsRC.pdf).
Solitaire follows [Schneier v1.2, 1999-05-26](https://www.schneier.com/academic/solitaire/).
Chaocipher follows [Rubin, updated 2010-07-30](https://www.chaocipher.com/ActualChaocipher/Chaocipher-Revealed-Algorithm.pdf).
Hill's source is [Hill 1929, pp. 306–312](https://doi.org/10.1080/00029890.1929.11986963).
HELP→HIAT is an independently calculated fixture for matrix [3,3,2,5] under
the declared convention; short Trithemius/Running Key fixtures similarly check
explicit formulas rather than attributing those examples to an ACA sheet.
No historical book text or third-party implementation is bundled.

## Caller-owned book, keyboard and symbolic profiles

`cipher.book.encode` extends the existing decoder with exact caller source and
plaintext inputs. Rust uses `ciphers::book::encode(source, text, &EncodeOptions)`;
Python uses `book_encode(source, text, parameters)`. The required parameters are
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
p = {"scheme": "word_character", "index_base": 1, "matching": "round_robin"}
r = cube.book_encode("aa aa", "aaaaa", p)
assert r["coordinates"] == "1:1,1:2,2:1,2:2,1:1"
assert cube.book_decode("aa aa", r["coordinates"]) == "aaaaa"
```

`text.keyboard.coordinates.encode/decode` use an explicit `layout` object with
`geometry` and `entries: [{key, layer, symbol, row, column}]`, plus required
`index_base` 0 or 1. Coordinates are JSON tuples `[row,column,layer]`; layers are
literal IDs, symbols are single exact Unicode scalars. Positions are zero-based
`[byte_offset,scalar_offset]` pairs in the input text for encoding and reconstructed
text for decoding. `definition_sha256` hashes compact serialized validated layout
values; it does not hash the caller's JSON formatting. Geometry has 1..512 entries,
rows/columns 0..511 and a 64 KiB serialized ceiling. Key/layer/geometry IDs are
1..128 UTF-8 bytes. Duplicate symbols, key/layer pairs and coordinate/layer pairs
fail; the same physical key must retain its row/column across layers. The Rust
`text::keyboard::coordinates::Geometry::preset` helper extends existing US ANSI
QWERTY/Dvorak/QWERTZ data without changing legacy `Layout` JSON or remapping.

`text.keyboard.shift.encode/decode` take `layout`, signed exact `dx` and `dy`, and
`edge` (`reject`, default, or `wrap`). Positive displacement moves right/down in
the same layer; shift report coordinates always have origin zero. `reject` fails
when a used symbol reaches a missing cell. `wrap` requires contiguous rows and
columns from zero in each layer: move/wrap the row first, then move/wrap in the
destination row's length. Ragged vertical shifts can collide. Forward reports
`bijective`; reverse rejects every non-total or non-bijective mapping, even for an
empty message. Full i64 displacements are normalized without overflow. Keyboard
text/coordinate transport is at most 1 MiB and 65,536 scalars/coordinates; each
report is at most 4 MiB. No OS keyboard detection or key events are involved.

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

All seven operations have typed named ports in the generated operation reference,
real conformance fixtures and direct CLI/native-Python/Play adapters. Their
structured provenance does not introduce a pipeline adapter: engine API 1,
recipe format 1 and existing adapters are unchanged. Strict parameters reject
unknown/duplicate fields. Integer fields accept exact JSON numbers or canonical
decimal strings; strings preserve full i64 values across JavaScript transport.
