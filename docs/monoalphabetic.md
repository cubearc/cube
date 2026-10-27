# Monoalphabetic substitution profiles

Use explicit alphabets and tables to reproduce classical transformations. These
profiles support historical analysis and challenges; they provide no modern
confidentiality or authentication. Algorithms and validation live in Rust Core;
CLI and native Python adapt their exact results.

## Exact alphabets and keywords

```sh
cube cipher substitution --source ABC --target CAB --text ABC
# CAB
cube cipher substitution --source ABC --target CAB --decode --text CAB
# ABC
cube cipher substitution --source ABC --keyword B --text ABC
# BAC
```

```python
import cube

assert cube.substitution_encode("ABC", "ABC", target="CAB") == "CAB"
assert cube.substitution_decode("CAB", "ABC", target="CAB") == "ABC"
assert cube.substitution_encode("ABC", "ABC", keyword="B") == "BAC"
assert cube.substitution_encode("ab", "ab", target="🦀猫") == "🦀猫"
```

Supply exactly one target alphabet or keyword. Source and target contain the same
number of unique Unicode scalars, 1..4096 each. Keyword construction keeps the
first occurrence of each key scalar, then appends unused source scalars in source
order. Every key scalar must belong to the source; an empty key yields identity.
There is no normalization, case folding or linguistic alphabet inference.
Combining marks and NUL are ordinary distinct scalars.

For alphabets containing NUL, use CLI `--definition FILE` instead of alphabet
flags: the strict JSON object contains `source`, exactly one `target`/`keyword`,
and optional `unmapped`. This transport accepts escaped scalars that OS command
arguments cannot represent. The file is read-only and capped at 64 KiB; plaintext
can come from exact stdin or `--file`. Definition mode conflicts with alphabet
and policy flags so no override is inferred.

The default `unmapped="reject"` fails at the original UTF-8 byte offset of an
unmapped scalar. `--unmapped preserve` / `unmapped="preserve"` leaves it untouched.
Preserve can lose invertibility when source and target sets differ: mapping `a`
to `x` transforms `ax` to `xx`, whose inverse mapping yields `aa`. The bijection
is guaranteed on the declared alphabet; disjoint preserved symbols also round-trip.
ASCII Caesar, Atbash, Affine and their existing signed-shift rules are unchanged.

Aristocrat and Patristocrat describe simple-substitution presentation conventions,
with word divisions retained or removed. Prepare that formatting explicitly;
substitution never strips spaces or guesses word boundaries. For a keyword
permutation over the same letter set, explicit `unmapped="preserve"` can retain
word divisions and punctuation.

Rust callers construct `ciphers::mono::alphabet::Mapping::new` or `::keyword` and
call `encode` / `decode` with the explicit unsupported-symbol policy.

## ASCII rotations

```sh
cube cipher rot5 --text 'A123'
# A678
cube cipher rot18 --text 'Hello 123!'
# Uryyb 678!
cube cipher rot47 --text 'Hello, World!'
# w6==@[ (@C=5P
```

The native functions are `rot5`, `rot18` and `rot47`. ROT5 changes only ASCII
0–9; ROT18 combines existing letter ROT13 with digit ROT5. ROT47 changes exactly
U+0021..U+007E, preserving space, controls, NUL and other Unicode scalars.
Each profile preserves UTF-8 byte length and applying it twice restores the input.
These are separate digit/printable profiles; letter ROT1–25 still use Caesar.

## Caller-owned homophonic tables

```python
import cube

mapping = {
    "ascii_uppercase": True,
    "entries": [
        {"symbol": "A", "tokens": ["1", "2"]},
        {"symbol": "B", "tokens": ["3"]},
    ],
}
assert cube.homophonic_encode("aabA", mapping) == ["1", "2", "3", "1"]
assert cube.homophonic_decode(["1", "2", "3"], mapping) == "AAB"
assert cube.homophonic_encode(
    "AA", mapping, counters=[{"symbol": "A", "index": 65535}]
) == ["2", "1"]
```

`ascii_uppercase` is required. When true, the table contains uppercase ASCII
source letters where applicable, and input ASCII letters canonicalize to uppercase.
Other scalars match exactly. When false, all scalar identities remain exact.
Each source has 1..256 nonempty string tokens; each token belongs to exactly one
source. Tokens can contain Unicode or delimiter characters because transport is
an explicit JSON array. Concatenating `"1"` and `"22"` never produces an accepted
implicit token `"122"`.

Each source cycles through its caller-listed token choices independently. Counters
reset on every call, with optional canonical-symbol indexes in 0..65535 reduced
modulo that source's choice count. Unknown/duplicate counter symbols fail. There
is no random generator or hidden encoder state. Decoding restores canonical
plaintext; encoding an arbitrary decoded ciphertext need not choose the same
homophones. Unsupported source symbols and unknown tokens fail without echoing them.

From the repository root, the small synthetic fixture is executable:

```sh
cube cipher homophonic \
  --definition crates/cube-core/tests/fixtures/mono/homophonic.json --text AABA
# ["1","2","3","1"]
cube cipher homophonic \
  --definition crates/cube-core/tests/fixtures/mono/homophonic.json \
  --decode --text '["1","2","3","1"]'
# AABA
```

Rust uses `homophonic::Definition`, `Mapping::new` / `::parse`, and the typed
`Counter` records. CLI definition files are explicit local UTF-8 JSON, opened
read-only with a 64 KiB limit; the SDK accepts ordinary dictionaries.

## Three distinct digit grammars

All three require a complete caller-owned definition: `digits` is a permutation
of the ten ASCII digits, `prefixes` explicitly names row prefixes (empty for
Tridigital), and `alphabet` gives the exact cell order. `aliases` defaults to an
empty list of `{"from": "J", "to": "I"}` records; `digit_escape` defaults to null.
Unknown fields and duplicate JSON keys are errors. ASCII input letters uppercase;
other symbols require an explicit supported cell. No punctuation or whitespace
is silently removed. Transport is a strict ungrouped digit string.

- **Straddling:** two prefix digits reserve first-row holes. Fill its eight
  one-digit cells in the supplied digit order, then two rows of ten two-digit
  cells in prefix order. At most 28 distinct canonical cells are supported;
  trailing unassigned cells reject. A supplied escape marker must be a canonical
  nonalphanumeric cell, with no ordinary digit cells. It encodes a literal digit
  as the marker code followed by that digit; literal marker input rejects.
  A space must have an explicit cell.
- **Monome-Dinome:** the supported [ACA 3×8 profile](https://www.cryptogram.org/downloads/aca.info/ciphers/MonomeDinome.pdf)
  uses 24 uppercase cells, the two prefixes and the eight remaining digits as
  columns. Require `J→I` and exactly one additional explicit merge so the table
  covers all 26 letters. Those aliases are canonicalization, not a bijection.
- **Tridigital:** the [ACA profile](https://www.cryptogram.org/downloads/aca.info/ciphers/Tridigital.pdf)
  fills nine columns with an explicit permutation of A–Z. The tenth digit encodes
  each literal space. Several letters share a digit, so decoding returns ordered
  per-digit alternatives, never guessed plaintext or dictionary words. Prefixes,
  aliases and digit escapes are not accepted for this profile.

```python
import cube

table = {
    "digits": "6703528149",
    "prefixes": "",
    "alphabet": "DRAGONFLYBCEHIJKMPQSTUVWXZ",
}
assert cube.tridigital_encode("THE", table) == "030"
assert cube.tridigital_decode("09", table) == [["A", "E", "T"], [" "]]
```

CLI commands are `cipher straddling`, `cipher monome-dinome` and
`cipher tridigital`, each with `--definition FILE`, ordinary input selectors and
optional `--decode`. Native functions use the corresponding `_encode` / `_decode`
names. Rust selects `checkerboard::Profile` explicitly and uses `Table`; Tridigital
calls `alternatives`, while reversible digit grammars call `decode`.

ACA's separately named 5×5 **Checkerboard** uses letter coordinate labels. It is
not silently treated as the straddling or Monome-Dinome grammar. A caller can
represent an explicit letter-code table with homophonic tokens; the research
record describes that reuse without adding a misleading extra operation ID.

## Grandpré coordinates

`grandpre::Definition` contains 8, 9 or 10 uppercase ASCII-letter rows, each the
same length as the side, plus explicit `row_labels` and `column_labels`. Each
label string contains exactly that many unique ASCII digits, in cell order.
`0123456789` gives zero-origin labels; `1234567890` explicitly gives the
one-origin convention with ten represented by zero. Nothing infers the origin.

Repeated letters are legal. Encoding picks matching cells round-robin in row-major
order, resetting for each call; missing letters reject. Input letters uppercase;
each literal ASCII space becomes a distinct null token. Decoding accepts only
exact row/column digit pairs or null and returns canonical uppercase text/spaces.
The supported subset is narrower than the [ACA's general range](https://www.cryptogram.org/downloads/aca.info/ciphers/Grandpre.pdf).
There is no bundled word list, word validation or automatic all-letter completeness.

CLI `cipher grandpre --definition FILE` encodes to a JSON coordinate array;
`--decode` reads that exact array. Native `grandpre_encode` / `grandpre_decode`
use lists of two integers or `None`; booleans and numeric coercion are rejected.
Rust uses `grandpre::Square`, retaining the existing coordinate `Token` transport
without imposing Polybius's unique-cell requirement.

## Bounds, errors and contracts

| Profile | Input/data ceiling | Table ceiling | Output ceiling |
| --- | --- | --- | --- |
| Exact/keyword substitution | 1 MiB UTF-8 | 4096 scalars/alphabet; 64 KiB combined definition | 4 MiB UTF-8 |
| ROT5/18/47 | 1 MiB UTF-8 | No table/key | Same UTF-8 byte length |
| Homophonic | 1 MiB text or token JSON; 65536 source scalars/tokens | 4096 tokens total; 256 tokens/source; 256 UTF-8 bytes/token; 64 KiB JSON | 1 MiB text/serialized token JSON |
| Digit grammars | 1 MiB UTF-8; 65536 logical tokens | 64 KiB JSON; explicit finite cells | 4 MiB text/alternatives JSON |
| Grandpré | 1 MiB text/token JSON; 65536 tokens | At most 100 cells; 64 KiB JSON | 1 MiB text/serialized coordinates |

These are Core ceilings. CLI and native Python also validate their JSON
transports. Play applies separate limits to complete named input, parameter and
output JSON, where wrappers and escaping count; those interface ceilings can be
stricter than the raw Core limits above.

Algorithms check input/table/token limits and output growth or serialized-byte
budgets; escaped JSON bytes count. Work and temporary memory are bounded by these
finite caps and mapping sizes, without a separate caller-adjustable work/heap
counter. Caller-owned and Python/Play object overhead is separate. No operation
reads files or uses a network in Core. No pipeline adapter or capture permission
is granted by a descriptor.

Typed Core errors distinguish `invalid_parameter`, `invalid_input` and
`resource_limit`, retaining the operation and parameter; source-text errors carry
original UTF-8 byte offsets. The CLI's `--json-errors` preserves empty stdout on
failure. Existing module-owned discovery, native stubs and canonical contracts
provide the stable IDs, exact parameter/result shapes, defaults and availability.
The research-only names ROT8000, Malespín, Vatsyayana and historical codebooks are
not advertised as newly implemented algorithms.
