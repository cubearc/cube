# Custom numeric alphabets

Convert an integer between two ordered alphabets with `number.alphabet.convert`.
Digits match Unicode scalars exactly; Cube does not normalize or fold their case.

## Alphabet and input rules

The source and target alphabets follow the same rules:

| Setting | Accepted values |
| --- | --- |
| Base | 2–256, no greater than the alphabet length |
| Active digits | The first `base` entries |
| Alphabet | Unique scalars; no ASCII `+`/`-` or Unicode whitespace |
| Alphabet size | At most 1,024 UTF-8 bytes |
| Input sign | Optional ASCII `+` or `-` |
| Magnitude | At most `2^128 − 1` |
| Input size | At most 4,096 digits and 16,385 bytes |

## Output

Output removes leading zeros and a leading plus sign. Negative zero becomes the
target alphabet's zero digit.

The result is at most 128 digits plus a sign, or 513 UTF-8 bytes.
This operation has no pipeline adapter.

See [Convert a custom digit alphabet](../../guides/custom-digits.md) for an example.
