### Puzzle ciphers and symbolic mappings

These direct operations share Rust implementations with CLI and native Python.
They do not add pipeline adapters; discovery reports `pipeline: null`. Existing
engine API 1, recipe format 1 and adapter contracts keep their behavior. Text
input is bounded at 1 MiB; errors do not return partial results.

### Bacon and coordinate squares

Bacon requires `classical24` (`ABCDEFGHIKLMNOPQRSTUWXYZ`, J→I and V→U) or
`extended26` (`ABCDEFGHIJKLMNOPQRSTUVWXYZ`). A letter's zero-based index is its
five-bit code, most significant bit first: A=`AAAAA`, B=`AAAAB`; classical Z is
`BABBB`, extended Z is `BBAAB`. This matches the classical endpoints in the
[Folger exhibition](https://folgerpedia.folger.edu/Decoding_the_Renaissance_exhibition_material#Bacon.27s_Biliteral_Cipher_.28case_9.29).
Notation `ab` is the default; `binary` explicitly means A=0/B=1 and
`binary_inverted` means A=1/B=0. Encoding folds ASCII letter case, collapses ASCII
space runs and removes edge spaces. Exactly one space separates groups and
` / ` separates nonempty words. Decode accepts only that grammar; unsupported
letters, punctuation, whitespace, incomplete and unused codes fail with byte
offsets. Output is bounded by six times input bytes (at most 6 MiB).

One `polybius::Square` implements both coordinate conventions. The 5×5
`polybius` rows are ABCDE/FGHIK/LMNOP/QRSTU/VWXYZ, with J→I; `tap` rows are
ABCDE/FGHIJ/LMNOP/QRSTU/VWXYZ, with K→C. The
[NSA Tap Code material](https://www.nsa.gov/Puzzles/View/Article/3860720/no-such-puzzle-tap-codes/)
provides the latter convention. Both presets fold ASCII case; custom alphabets
match exact unique Unicode scalars. Supply side 1..64 and exactly side² cells;
ASCII space is reserved for word tokens. Explicit aliases point directly to
canonical cells, never another alias. Definitions are limited to 64 KiB
(alphabet bytes plus eight bytes per alias) and 4,096 aliases.

Coordinates are row then column; origin is 1 by default, optionally 0. Each
literal ASCII space becomes a separate `null` token in JSON/native Python, so
repeated and edge spaces survive. CLI tokens are `ROW,COLUMN` or `/`, separated
by exactly one ASCII space. At most 65,536 tokens include spaces. Custom
alias-free squares round-trip exactly; presets/aliases return canonical cells.
There are no separate Tap operation IDs because this release uses the same
coordinate grammar with an explicit preset, without tap detection or timing.

### Keyboard and keypad

Keyboard remapping requires source and target layouts. `qwerty` and `dvorak`
cover the 47 US ANSI printable positions on base and shift layers, checked
against [libxkbcommon's XKB fixtures](https://github.com/xkbcommon/libxkbcommon/blob/master/test/data/symbols/us).
`qwertz` swaps only Y/Z on that geometry; it is not a complete German layout.
Spaces/control keys and additional modifier layers are outside these presets.
Unmapped scalars default to `preserve`; `error` includes their UTF-8 byte offset.
Custom Layout objects contain `geometry` and `entries`, each with `key`, `layer`
and one-scalar `symbol`. Both layouts need the exact same geometry and key/layer
set. Duplicate positions or symbols are rejected, including cross-layer
ambiguity. Limits are 512 entries and 64 KiB serialized per layout, 128-byte
geometry/key/layer IDs and 8 MiB output. Use an explicitly smaller common
geometry for partial mappings; no implicit intersection or case folding occurs.

Keypad requires `telephone` or `multi_tap`. The standard
[ITU E.161](https://www.itu.int/rec/T-REC-E.161-200102-I/en) groups are
ABC=2, DEF=3, GHI=4, JKL=5, MNO=6, PQRS=7, TUV=8, WXYZ=9.
ASCII letters fold to uppercase; spaces/digits require a custom disjoint mapping.
Telephone encoding emits contiguous digits. Decode returns `text: null`, ordered
per-digit `alternatives` and `reversible: false`, without expanding combinations.
Multi-tap uses one `|`-delimited repeated-digit group per character; HELLO is
`44|33|555|555|666`. Decode returns canonical `text`, empty `alternatives` and
`reversible: true` for token inversion; original letter case is not restored.
Custom Mapping objects specify `ascii_fold` and `keys` with unique `digit` and
ordered `symbols`. Limits are ten digit keys, 256 disjoint scalars, 64 KiB
serialized definitions, 65,536 groups/results, and 8 MiB output/serialized report.

### Character classes

The default maps ASCII vowels AEIOU to `0` and the remaining ASCII letters,
excluding Y, to `1`, folding ASCII case. If Y occurs, explicitly choose `vowel`,
`consonant` or `unmapped`. The unmapped policy defaults to `error`, with explicit
`preserve` and `filter` alternatives. Custom Definition objects supply
`ascii_fold` and `classes`, each with unique nonempty `name`, disjoint
`characters` and distinct nonempty, prefix-free output `symbol`. Duplicate scalars after
folding and conflicting preset/custom options fail. No Unicode normalization or
linguistic vowel classification is implied.

Reports contain `mapped_text`, mapped `records` (class/emitted symbol and original
zero-based scalar index/UTF-8 byte offset), separate `preserved` records, and
`filtered` positions with `filtered_count`. Preserved scalars appear in text but
are not class symbols. Bounds are 4,096 definition scalars/64 KiB serialized,
65,536 original positions, and 8 MiB text/report. Incremental retention accounting
may reject an expansion conservatively before the final serialized report limit;
no silent truncation occurs. Mapping is lossy and has no inverse.

Research decisions for configurable Abjad/alphabet operations and Ook!/Brainfuck
notation are recorded in [#120](https://github.com/cubearc/cube/issues/120) and
[#121](https://github.com/cubearc/cube/issues/121). These research issues added no
callable operation. The separately implemented
[bounded Brainfuck profile](../../brainfuck.md) now supplies execution and diagnostics;
Ook notation conversion and bundled language-specific tables remain deferred.
