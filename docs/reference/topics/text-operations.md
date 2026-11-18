# Text operations

Normalize, reverse, select, count or arrange Unicode text. Positions count
[Unicode scalars](../../concepts/text.md#bytes-scalars-and-visible-characters).

## Reversal and positional extraction

| Operation | Behavior |
| --- | --- |
| Reverse | Reverse scalar order |
| Extract by index | Preserve requested order and repeated positions |
| Extract by stride | Use an exclusive stop; reject out-of-range bounds |

No text is implicitly trimmed, clipped or normalized.

For ordered operands and an explicit separator, see
[ordered composition](../composition.md).

```sh
cube text reverse --text 'Cube 🧊'
# 🧊 ebuC
cube text extract --start 1 --stride 2 --text abcdef
# bdf
```

## Word and line selection

`text.select` extracts the first or last scalar of each word or line.

| Unit | Boundary and selection rules |
| --- | --- |
| Word | Split on Unicode whitespace; retain punctuation |
| Line | Split on LF/CRLF; preserve bare CR, boundary spaces and punctuation |
| Empty line | Add no character |
| `line_first` | An acrostic without trimming |

The tokenizer exposes `WORD_UNICODE_VERSION` independently of normalization
data. Input and output are each limited to 1 MiB. Selection has a text pipeline
adapter.

## Normalization and UTF-8

NFC, NFD, NFKC and NFKD normalization are explicit and may lose distinctions.
The SDK exposes the data version as `cube.UNICODE_VERSION`.

| Operation | Input limit | Output or error rule |
| --- | --- | --- |
| Normalize | 1 MiB UTF-8 | At most 8 MiB |
| `text.utf8.encode` / `text.utf8.decode` | Pipeline or direct-operation limits | Convert types without normalization or replacement; invalid UTF-8 fails |

### ASCII and UTF-16

`cube text ascii encode` and `cube text ascii decode` use Unicode scalars and
byte values 0 through 127. UTF-8 retains strict validation through
`cube text to-bytes` and `cube text from-bytes`. All three formats report
invalid input at its original byte offset.

Use `cube text utf16 encode` and `cube text utf16 decode` with
`--byte-order le` or `--byte-order be` and an explicit `--bom` policy.

| Policy | Encode | Decode |
| --- | --- | --- |
| `require` | Prepend a matching marker | Require and remove the matching initial marker |
| `forbid` | Reject leading U+FEFF or U+FFFE | Reject either marker at the start |
| `strip` | Encode the given scalars | Remove a matching initial marker; reject an opposite marker |
| `preserve` | Encode the given scalars | Interpret every code unit as data |

UTF-16 decode rejects odd lengths and unpaired surrogates. Encoders accept
at most 1 MiB of UTF-8. UTF-16 binary input and output and decoded UTF-8
are capped at 4 MiB. ASCII and UTF-8 cap direct input at 1 MiB.

### JSON and HTML character escapes

`cube text json-escape encode` writes RFC 8259 string content without outer
quotes. Its default escapes non-ASCII scalars. Use `--allow-unicode` to keep
those scalars literal. The decoder accepts valid JSON string escapes and
pairs of surrogate escapes. It rejects raw controls, unescaped quotes,
malformed escapes and unpaired surrogates.

`cube text html-entity encode` emits hexadecimal character references for
ampersand, angle brackets, double quote and apostrophe. The decoder follows
WHATWG text-context matching, including the longest named match and numeric
replacements for zero, surrogates, out-of-range values and applicable C1
controls. Unknown references remain literal. The operation preserves markup
and line endings as text and decodes references once.

Both encoders accept at most 1 MiB of UTF-8 and produce at most 12 MiB.
Decoders accept up to 12 MiB and return up to 1 MiB of UTF-8.

```sh
cube text json-escape encode --text '🧊'
# \uD83E\uDDCA
cube text html-entity decode --text '&lt;&amp;&gt;'
# <&>
```

## Text statistics

Statistics count scalars and overlapping n-grams, with `n` from 1 to 16.
Preparation uses the [shared text profile](../text-preparation.md): defaults
preserve case and every scalar without normalization.

| Result | Definition |
| --- | --- |
| Counts | Descending frequency, then lexical text order |
| Index of coincidence | `sum(c*(c-1)) / (N*(N-1))`; undefined for fewer than two scalars |
| Shannon entropy | `-sum(p*log2(p))` bits/scalar; undefined for empty input, zero for a constant distribution |

IC and entropy always use single-scalar counts, even when n-grams are requested.
Floating results use `f64` and need numerical tolerance. These measurements do
not identify a cipher.

### Statistics limits

| Resource | Maximum |
| --- | --- |
| UTF-8 input | 1 MiB |
| Normalized text | 8 MiB |
| Retained sample | 1,048,576 scalars |
| Work | 8,388,608 scalar-window units |
| Distinct character values / n-grams | 10,000 per map |

Exceeding a resource cap fails. `max_results` bounds only the sorted listing;
`complete` reports omitted rows while total window and distinct counts stay exact.
Statistics return structured results through direct APIs/JSON, without a pipeline
adapter.

## Text combinations

`text.combinations` computes a Cartesian product of ordered string groups.
The rightmost index varies fastest. Duplicates, empty strings, Unicode and
control characters are preserved.

- Zero groups yield one empty string.
- An empty group yields no candidates.
- Rust and Python yield one result at a time; the CLI streams JSONL.
- Each result can enter ordinary transforms through explicit caller conversion.
  There is no pipeline iterator or candidate-search engine.

### Combination limits

| Resource | Maximum |
| --- | --- |
| Groups / fragments | 16 / 4,096 |
| Total input | 1 MiB |
| One fragment / result | 64 KiB |
| Separator | 4,096 bytes |
| Results | 1,000 by default; hard limit 10,000 |
| Aggregate raw and escaped JSONL output | 16 MiB |
| Work | 33,554,432 units |

Preflight checks count overflow, exact lengths and JSON escaping costs.
Work includes validation, output bytes and fragment/concatenation units.
Excessive plans fail before yielding a prefix. JSON transport limits still apply.
See [resource limits](../../concepts/resource-limits.md) for units and reservations.

## Text grids

Grid factors and rendering retain Unicode scalars. Optional Rust `White_Space`
removal reports removed and retained counts using `WORD_UNICODE_VERSION`.

| Setting or output | Rule |
| --- | --- |
| Input | At most 1 MiB and 65,536 retained scalars |
| Factors | Ascending width; both orientations; at most 16,384 per side |
| Candidate count | Default and hard cap 128; excess fails |
| Explicit width | Must divide the scalar count |
| Placement | Uses existing traversal coordinates |
| Plain text | JSON-quoted cells, tabs and LF rows; less than 1 MiB |

### Color rendering

Provide one RGBA tuple per distinct scalar, with at most 256 mappings. Missing,
unused and duplicate mappings fail. The CLI takes a strict JSON array of
`[scalar, eight Hex digits]`, limited to 8,192 bytes; Python takes a dictionary.

Rendering constructs a PNG from the supplied colors. It does not use fonts,
infer colors or introduce a pipeline grid type.
