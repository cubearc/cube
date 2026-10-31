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
For ASCII Caesar, Atbash, Affine and signed shifts, see
[classical ciphers](reference/topics/classical-ciphers.md).

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

All profiles accept at most 1 MiB of UTF-8 input. Homophonic and Grandpré also
accept token JSON within that byte limit. Homophonic, digit grammars and Grandpré
have a separate 65,536 source-scalar/logical-token ceiling.

### Definition limits

Each supplied definition is at most 64 KiB; substitution counts its combined
alphabet definition. ROT profiles need no table or key.

| Profile | Definition contents |
| --- | --- |
| Exact/keyword substitution | At most 4,096 scalars per alphabet |
| Homophonic | At most 4,096 tokens total, 256 per source and 256 UTF-8 bytes per token |
| Digit grammars | Explicit finite cells as defined by the profile |
| Grandpré | At most 100 cells |

### Output limits

| Profile | Maximum |
| --- | --- |
| Exact/keyword substitution | 4 MiB UTF-8 |
| ROT5/18/47 | Same UTF-8 byte length as input |
| ROT8000 | 3 MiB UTF-8 |
| Homophonic | 1 MiB text or serialized token JSON |
| Digit grammars | 4 MiB text or alternatives JSON |
| Grandpré | 1 MiB text or serialized coordinates |

These are Core ceilings. CLI and native Python validate JSON transport too;
Play applies separate named-input, parameter and output budgets. See
[Core and interface limits](concepts/resource-limits.md#core-and-interface-limits).

Algorithms check input/table/token limits and output growth or serialized-byte
budgets; escaped JSON bytes count. Work and temporary memory are bounded by these
finite caps and mapping sizes, without a separate caller-adjustable work/heap
counter. Caller-owned and Python/Play object overhead is separate. Core does not
read files or use a network. These operations have no pipeline adapters;
[execution permissions](concepts/data.md#operations-pipelines-and-play) are separate
from catalog discovery.

Typed Core errors distinguish `invalid_parameter`, `invalid_input` and
`resource_limit`, retaining the operation and parameter; source-text errors carry
original UTF-8 byte offsets. The CLI's `--json-errors` preserves empty stdout on
failure. Existing module-owned discovery, native stubs and canonical contracts
provide the stable IDs, exact parameter/result shapes, defaults and availability.
## ROT8000 and paired profiles

`cube cipher rot8000 'A 😀'` returns `籊 😀`. The fixed
[ROT8000 profile](https://github.com/rottytooth/rot8000/tree/c47905631615173e51ac88e0901c4be642a1c8aa)
rotates an ordered set of 63,404 BMP units by 31,702 positions. Excluded BMP
units and non-BMP scalars remain exact. It accepts valid UTF-8, performs no
normalization and is its own inverse. The input ceiling is 1 MiB and the output
ceiling is 3 MiB. This profile has its own compact implementation; it does not
use the caller-defined alphabet limit.

`cube cipher malespin 'bueno Amigo á'` returns `tuani Epofi á`. The written
Malespín profile exchanges `a/e`, `i/o`, `b/t`, `f/g` and `p/m`, retaining ASCII
case. Accented vowels and all other scalars are preserved. The ten-letter
mechanism follows Sorbet's [original study, section 5](https://doi.org/10.4000/bulletinhispanique.16749).
This exact orthographic profile does not apply regional phonetic changes,
accent exchange, silent-letter deletion or contemporary slang reconstruction.
Input and output are at most 1 MiB.

Vatsyayana uses required disjoint scalar pairs. Supply each pair once, with two
different scalars; every scalar may belong to only one pair. There is no default
historical alphabet. The same operation decodes its output. Unmapped scalars
reject by default; `--unmapped preserve` retains them without folding or
normalization. At most 2,048 pairs use the existing 4,096-symbol and 64 KiB
combined-alphabet limits; input is at most 1 MiB and output at most 4 MiB.

```sh
cube cipher vatsyayana QOEU --pairs '[["Q","W"],["O","I"],["E","S"],["U","H"]]'
```

The result is `WISH`, using the pairs in the [Waterloo 2020 challenge](https://cemc.uwaterloo.ca/sites/default/files/documents/2020/2020BCCContestSolutions7_8.pdf).
This is an explicit modern pairing convention, without attributing a fixed
Latin key to the ancient text.

## Words, syllables and historical codebooks

Nomenclator accepts an explicit table whose entries contain `alternatives` and
`codes` arrays, plus optional `null_codes` and `cancel_codes` arrays. Units and
codes are exact nonempty strings. Encoding takes a JSON array of already
segmented units and chooses the first listed matching code. It never infers
word boundaries or a longest match. For example, save this as `codebook.json`:

```json
{"entries":[{"alternatives":["OR"],"codes":["10"]},{"alternatives":["D"],"codes":["05"]}],"null_codes":["99"],"cancel_codes":["27"]}
```

```sh
cube cipher nomenclator '["OR","D"]' --definition codebook.json
cube cipher nomenclator '["10","05","27","99"]' --definition codebook.json --decode
```

The first command returns `["10","05"]`. The second returns the `OR` reading
at token index 0, null index 3, canceled index 1 and cancellation index 2.
Cancellation removes the last surviving whole group, including a null or an
unknown group. A cancel with no preceding surviving group fails. Unknown
surviving codes fail. Codes are globally unique across table entries, nulls
and cancellation instructions. Alternative readings remain explicit, with
original zero-based token indices; no joined plaintext is guessed.

Syllabary uses the same segmented-unit behavior with an explicit ACA 10×10
keysquare: `cells` contains exactly 100 row-major strings; `row_labels` and
`column_labels` are permutations of `0123456789`. Cells may contain letters,
syllables or words. Coordinates are exact two-character tokens, including
leading zeros. Repeated plaintext units choose their first row-major code.
Use `cube cipher syllabary --definition square.json` and pass a JSON unit
array, or add `--decode` for code tokens. The [ACA worked example](https://www.cryptogram.org/downloads/aca.info/ciphers/Syllabary.pdf)
has multiple legal segmentations; the API preserves the caller's selection.

### Great Cipher of 1691

The fixed profile uses the second deciphering table in Burgaud and Bazeries,
[*Le Masque de fer* (1893), pp. 280–289](https://gallica.bnf.fr/ark:/12148/bpt6k1523532j).
It exposes the 587 numbered positions exactly through a documented ASCII
transcription: accents are omitted, capitalization retained, and `Geneve?`
retains the printed question mark. Empty table entries are unknown and reject;
they are never treated as nulls. Group `27` cancels a preceding whole group.
Only explicitly marked nulls are omitted. Printed I/J, U/V and other alternatives
remain separate readings. No word segmentation or historical identity claim is
inferred from a decipherment.

```sh
cube cipher great-cipher '["Les","En","Ne","Mi","S"]'
cube cipher great-cipher '["55","502"]' --decode
```

Encoding returns `["124","22","125","46","120"]`; decoding retains `I/J`
at index 0 and `Bien` at index 1. This profile uses the deciphering table as
printed, including group 502, rather than silently substituting an inferred
historical correction. See [data provenance](../THIRD_PARTY_NOTICES).

### Codebook limits

All direct Rust and SDK callers enforce these ceilings:

| Resource | Maximum |
| --- | --- |
| Table, input array or output report, compact UTF-8 JSON | 1 MiB each |
| Input units/code tokens | 65,536 |
| Nomenclator entries and total owned codes, including null/cancel codes | 4,096 each |
| Individual unit/code | 256 UTF-8 bytes |
| Alternatives per entry | 16 |
| Codes per entry | 256 |

Overlapping plaintext units are legal because segmentation is explicit.
Duplicate code ownership, empty units, unknown fields and duplicate JSON keys
reject. Reports check their exact escaped serialized size before copying table
strings into output. Processing uses bounded lookup and a stack of token indices;
it performs no exponential segmentation search. Native Play additionally enforces
its own input, output and literal budgets. The operations expose pure named-port
execution; they do not enable a linear Core pipeline adapter.

Python exports `rot8000`, `malespin`, `vatsyayana`, `nomenclator_encode/decode`,
`syllabary_encode/decode` and `great_cipher_encode/decode`. JavaScript/TypeScript
uses the same version-1 operation IDs and named ports in `cube.execute`; its
types and runtime availability derive from the actual packaged Wasm.
