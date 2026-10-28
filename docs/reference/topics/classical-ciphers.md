# Classical ciphers

Use classical ciphers for letter substitutions with a known key or alphabet.
They do not provide secure modern encryption.

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

## Modern encrypted files

The current API has no modern encrypted-file decryption operation. Proposed
formats and interoperability requirements are recorded in the
[format evaluation](https://github.com/cubearc/cube/issues/78).
