# Classical ciphers

Use classical ciphers for letter substitutions with a known key or alphabet.
They do not provide secure modern encryption.

[Enigma I and M3](enigma.md) covers configured rotor-machine messages and state.

## Letter substitutions

| Cipher | Key or parameters | Behavior |
| --- | --- | --- |
| Caesar | Signed 32-bit shift | Shift ASCII letters; preserve case and other Unicode |
| ROT13 | None | Rotate ASCII letters by 13; preserve case and other Unicode |
| Atbash | None | Reverse the ASCII alphabet; preserve case and other Unicode |
| Vigenère | Nonempty ASCII-letter key | Advance the key only for ASCII input letters |
| Beaufort | Nonempty ASCII-letter key | Standard, self-inverse Beaufort; not Variant Beaufort |
| Affine | `a,b` in 0–25; `a` coprime to 26 | `C = (aP + b) mod 26`, with A=0; decode with the same parameters |

Vigenère, Beaufort and affine inputs are limited to 1 MiB. Vigenère and Beaufort
keys are limited to 64 KiB.

[Explicit key schedules](../../ciphers.md#letter-preserving-schedules) cover
Interrupted Key, Progressive Key and the specified Vigenère–Multiplicative
composition, including reset offsets, progression and inverse rules.

For example, encrypt with Vigenère or decode an affine substitution:

```sh
cube cipher vigenere --key LEMON --text ATTACKATDAWN
# LXFOPVEFRNHR
cube cipher affine --decode --a 5 --b 8 --text IHHWVCSWFRCP
# AFFINECIPHER
```

[Monoalphabetic profiles](../../monoalphabetic.md) cover exact and keyword
alphabets, ASCII rotations, deterministic homophones, digit grammars and Grandpré.

## Book coordinates

Book coordinates select characters from an exact, caller-supplied UTF-8 source.
Record its edition and byte-level SHA-256 to reproduce a result.

### Coordinate formats

Separate records with commas. The coordinate base defaults to 1; base 0 is
available explicitly.

| Mode | Record |
| --- | --- |
| `word_character` | `word:scalar` |
| `line_character` | `line:scalar` |
| `line_word_character` | `line:word:scalar` |

Leading decimal zeros and surrounding ASCII whitespace are accepted. Signs,
empty records and invalid coordinates fail with a record number; no partial
result or substitute character is returned.

### Word and line rules

- Words split on Unicode whitespace and retain punctuation, as in `text.select`.
- LF and CRLF are line endings. Blank lines count; a final newline adds no line.
- Bare CR and spaces inside a line remain data.
- Character positions count Unicode scalars, including separate combining marks.
  No normalization is applied.

### Limits and support

| Resource | Maximum |
| --- | --- |
| Source text | 1 MiB |
| Coordinate text | 1 MiB |
| Coordinates | 65,536 |
| Indexed units | 65,536; lines plus words for triple coordinates |
| Output | 256 KiB |

No book is bundled, fetched or stored. Book coordinates have no pipeline adapter;
their text result can be passed to ordinary text operations.

## Whole words from caller pages

`cipher.arnold.encode` and `.decode` provide the explicit
`arnold_paged_words_v1` profile. The
[Friedman lecture](https://www.nsa.gov/Portals/75/documents/news-features/declassified-documents/friedman-documents/lectures-speeches/FOLDER_019/41700339073966.pdf)
describes dictionary/book word codes and page, line and word coordinates used in
that historical setting. This profile takes the caller's exact pages and complete
words; it does not reproduce an unspecified Blackstone or dictionary edition,
conventional code names, or a letter fallback.

Supply named `pages`, a JSON array of page strings, and either `words` or
`coordinates`. A coordinate is `{"page":1,"line":2,"word":1}`. Parameters require
`profile="arnold_paged_words_v1"`, an exact `source_id` identifying the caller's
edition, `index_base` 0 or 1, and `choice="first"` or `"round_robin"`.
Round robin cycles independently for each exact word and resets each invocation.
Word and line rules are shared with book coordinates above; page boundaries are
the supplied array boundaries. Case, punctuation and combining marks stay exact.

For pages `["THE ROAD\nTHE CUBE","CUBE 🐈\r\nROAD"]`, base 1 and round robin,
words `["THE","THE","CUBE","CUBE","🐈"]` select
`(1,1,1), (1,2,1), (1,2,2), (2,1,1), (2,1,2)`.
Decode those coordinates to recover the same presegmented words. Message spacing
is not reconstructed. Unknown words and unavailable coordinates fail.

Reports include `words`, selected `coordinates`, `source_id`, `source_sha256` and
`definition_sha256`. The source hash covers the compact JSON array of exact page
strings, preserving page boundaries and original line endings. The definition
hash also binds the canonical parameters. An edition ID is a caller assertion,
never a remote lookup or proof of authenticity.

CLI `cube cipher arnold --parameters FILE` reads a JSON input object containing
`pages` and `words`; use `--decode` with `pages` and `coordinates`. Native Python
functions are `arnold_encode(pages, words, parameters)` and
`arnold_decode(pages, coordinates, parameters)`. Rust uses
`ciphers::book::words::{encode,decode}`; JavaScript/TypeScript uses the canonical
IDs and corresponding named inputs. Native Play uses structured page/word/
coordinate values and inspectable named results.

| Resource | Maximum |
| --- | --- |
| Source pages | 1 MiB compact JSON |
| Indexed source | 65,536 combined pages, lines and words |
| Source word | 256 UTF-8 bytes |
| Message words or coordinate input | 65,536 items and 1 MiB compact JSON |
| Edition ID | 1024 UTF-8 bytes, nonempty |
| Serialized report | 4 MiB, with bounded expansion preflight |

Interfaces may apply lower aggregate input/output budgets. Neither books nor
source retrieval are bundled; the original character-coordinate operations
retain their formats and behavior.

## Modern encrypted files

The current API has no modern encrypted-file decryption operation. Proposed
formats and interoperability requirements are recorded in the
[format evaluation](https://github.com/cubearc/cube/issues/78).
