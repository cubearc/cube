# Numeric text evidence

Measure words and code points without normalizing, folding case or interpreting
the text as a numeric encoding.

## Operations

The operations retain source positions:

| Operation | Measurement | Position information |
| --- | --- | --- |
| `text.word.lengths` | Unicode-scalar count per word | Original word boundaries |
| `text.codepoints` | Exact `u32` value of every scalar, including whitespace | Scalar indexes and UTF-8 byte offsets |
| `text.word.values` | Value records inside whitespace-delimited words; defaults to code points | Per-character source positions |

Punctuation remains data. Decimal, hex and Unicode (`U+XXXX`) presentation changes
only the display, not the numeric code-point value. Cube does not infer A1Z26 or
concatenate decimal values.

## Custom values and sums

Custom tables contain unique, exact scalar/`u64` pairs.

- Unmapped scalars fail by default. Explicit skip retains null values and omitted counts.
- Optional sums include mapped values only and retain character provenance.
- `max_sum` is inclusive and defaults to `u64::MAX`. A non-default value requires sums.
- Overflow fails. CLI JSON writes exact integer digits; Python returns exact integers.

## Limits

| Resource | Maximum |
| --- | --- |
| Input | 1 MiB |
| Custom table | 4,096 pairs and 64 KiB of JSON |
| Word, code-point or total-value records | 65,536 |
| Source index | Less than 8 MiB |
| Serialized report | 8 MiB |

A report can reach its size limit before its record limit.
All three operations are direct APIs without pipeline adapters.

See [Inspect text and Unicode](../../guides/text-evidence.md).
