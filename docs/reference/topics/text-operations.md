# Text operations

Normalize, reverse, select, count or arrange Unicode text. Positions count Unicode
scalars, not UTF-8 bytes or grapheme clusters. Combining marks and emoji sequences
can span multiple positions.

## Reversal and positional extraction

| Operation | Behavior |
| --- | --- |
| Reverse | Reverse scalar order |
| Extract by index | Preserve requested order and repeated positions |
| Extract by stride | Use an exclusive stop; reject out-of-range bounds |

No text is implicitly trimmed, clipped or normalized.

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

## Text statistics

Statistics count scalars and overlapping n-grams, with `n` from 1 to 16.
Defaults preserve case and all scalars without normalization. ASCII lowercasing
and letter filtering are explicit options.

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
| Retained sample | 1 Mi scalars |
| Work | 8 Mi scalar-window units |
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
| Work | 32 Mi units |

Preflight checks count overflow, exact lengths and JSON escaping costs.
Work includes validation, output bytes and fragment/concatenation units.
Excessive plans fail before yielding a prefix. JSON transport limits still apply.

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
