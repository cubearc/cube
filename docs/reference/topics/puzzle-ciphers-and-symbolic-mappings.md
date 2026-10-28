# Puzzle ciphers and symbolic mappings

Apply explicit alphabets, coordinate squares, keyboard layouts or character
classes. Text input is limited to 1 MiB; errors return no partial result.
These operations have no pipeline adapters.

## Bacon and coordinate squares

### Bacon alphabets

Select an alphabet explicitly. Each letter's zero-based index becomes a
five-bit code, most significant bit first.

| Alphabet | Letters and aliases | Z |
| --- | --- | --- |
| `classical24` | `ABCDEFGHIKLMNOPQRSTUWXYZ`; J→I, V→U | `BABBB` |
| `extended26` | `ABCDEFGHIJKLMNOPQRSTUVWXYZ` | `BBAAB` |

A is `AAAAA` and B is `AAAAB`. Classical endpoints follow the
[Folger exhibition](https://folgerpedia.folger.edu/Decoding_the_Renaissance_exhibition_material#Bacon.27s_Biliteral_Cipher_.28case_9.29).

| Notation | Mapping |
| --- | --- |
| `ab` (default) | A and B |
| `binary` | A=0, B=1 |
| `binary_inverted` | A=1, B=0 |

Encoding folds ASCII case, collapses ASCII space runs and removes edge spaces.
The grammar uses one space between groups and ` / ` between nonempty words.
Decoding accepts only that grammar.

Unsupported letters, punctuation, whitespace, incomplete codes and unused codes
fail with byte offsets. Output is at most six times input bytes, up to 6 MiB.

### Coordinate-square presets

| Preset | Rows | Alias |
| --- | --- | --- |
| `polybius` | ABCDE / FGHIK / LMNOP / QRSTU / VWXYZ | J→I |
| `tap` | ABCDE / FGHIJ / LMNOP / QRSTU / VWXYZ | K→C |

Presets fold ASCII case. The Tap convention follows the
[NSA material](https://www.nsa.gov/Puzzles/View/Article/3860720/no-such-puzzle-tap-codes/).
Both use the same coordinate operation and grammar; Tap detection and timing are
not included.

### Custom squares and coordinates

| Setting | Rule |
| --- | --- |
| Side | 1–64 |
| Alphabet | Exactly side² unique Unicode scalars; ASCII space reserved |
| Aliases | Point directly to canonical cells, never other aliases |
| Definition size | At most 64 KiB: alphabet bytes plus eight bytes per alias |
| Aliases | At most 4,096 |
| Coordinates | Row, then column; base 1 by default, optionally 0 |
| Tokens | At most 65,536, including spaces |

Every literal ASCII space becomes a separate `null` token in structured results.
CLI tokens are `ROW,COLUMN` or `/`, separated by one ASCII space. Repeated and
edge spaces survive. Alias-free custom squares round-trip exactly; presets and
aliases return canonical cells.

## Keyboard and keypad

### Keyboard remapping

Select source and target layouts. Mapping uses the same physical key and layer
in each layout; it does not infer an intersection or fold case.

| Preset | Coverage |
| --- | --- |
| `qwerty`, `dvorak` | 47 US ANSI printable positions, base and shift layers |
| `qwertz` | Y/Z swap on that geometry; not a complete German layout |

Presets follow [libxkbcommon's XKB fixtures](https://github.com/xkbcommon/libxkbcommon/blob/master/test/data/symbols/us).
Spaces, control keys and other modifier layers are outside the presets.

| Custom layout rule | Requirement |
| --- | --- |
| Structure | `geometry` and `entries` with `key`, `layer` and one-scalar `symbol` |
| Compatibility | Exact same geometry and key/layer set |
| Uniqueness | No duplicate positions or symbols, including cross-layer ambiguity |
| Unmapped input | `preserve` by default; `error` reports the UTF-8 byte offset |
| Limits | 512 entries, 64 KiB serialized per layout; 128-byte geometry/key/layer IDs |
| Output | At most 8 MiB |

Use an explicitly smaller common geometry for partial mappings.

### Keypad modes

The standard [ITU E.161](https://www.itu.int/rec/T-REC-E.161-200102-I/en) groups are
ABC=2, DEF=3, GHI=4, JKL=5, MNO=6, PQRS=7, TUV=8 and WXYZ=9.
ASCII letters fold to uppercase; spaces and digits require a custom mapping.

| Mode | Encoding | Decoding |
| --- | --- | --- |
| `telephone` | Contiguous digits | `text: null`, ordered per-digit `alternatives`, `reversible: false`; no combination expansion |
| `multi_tap` | One `\|`-delimited repeated-digit group per character | Canonical `text`, empty `alternatives`, `reversible: true` for token inversion |

Multi-tap encodes HELLO as `44|33|555|555|666`. Neither mode restores original
letter case.

Custom mappings contain `ascii_fold` and `keys`, each with a unique `digit` and
ordered `symbols`. Limits are ten keys, 256 disjoint scalars, 64 KiB serialized
definitions, 65,536 groups/results, and 8 MiB output or serialized report.

## Character classes

The default maps ASCII vowels AEIOU to `0` and remaining ASCII letters, excluding
Y, to `1`. ASCII case is folded; no Unicode normalization or linguistic
classification is implied.

| Setting | Rule |
| --- | --- |
| Y | If present, explicitly choose `vowel`, `consonant` or `unmapped` |
| Unmapped input | `error` by default; `preserve` and `filter` are explicit alternatives |
| Custom definition | `ascii_fold` and `classes` with unique nonempty `name`, disjoint `characters` and distinct nonempty, prefix-free output `symbol` |
| Conflicts | Reject duplicate scalars after folding and preset/custom option conflicts |

### Results and limits

| Result field | Meaning |
| --- | --- |
| `mapped_text` | Class symbols plus preserved input |
| `records` | Class and emitted symbol, with original scalar index and UTF-8 byte offset |
| `preserved` | Unmapped scalars retained as text, not class symbols |
| `filtered`, `filtered_count` | Removed positions and their count |

Definitions allow 4,096 scalars and 64 KiB serialized data. Input positions are
limited to 65,536; text and reports to 8 MiB. Retention accounting can reject an
expansion before the final serialized limit. Results are never silently
truncated. This mapping is lossy and has no inverse.

## Related formats

[Bounded Brainfuck](../../brainfuck.md) provides execution and diagnostics.
Ook notation conversion and bundled language-specific tables remain deferred;
see research [#120](https://github.com/cubearc/cube/issues/120) and
[#121](https://github.com/cubearc/cube/issues/121).
