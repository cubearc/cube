# Unicode observations and scalar differences

Inspect character properties or compare text by stored scalar position.
The results describe code points, not rendered visibility or a security verdict.

## Character inspection

`text.unicode.inspect` returns original scalars, code points, offsets, case labels,
and flags for non-ASCII, control, whitespace, format and zero-width candidates.
It does not fold case.

Property sources are versioned separately:

| Properties | Data source |
| --- | --- |
| Uppercase, Lowercase and White_Space | Rust's reported Unicode version |
| Titlecase and General_Category, including Cf Format | `unicode-general-category` 1.1.0, Unicode 16.0 |
| Zero-width candidates | Profile 1: U+200B–200D, U+2060, U+FEFF and variation selectors U+FE00–FE0F/U+E0100–E01EF |
| Bidirectional controls | ALM, LRM/RLM, U+202A–202E and U+2066–2069 |

The category dataset is pinned; updates require a compatibility review.
See [Rust character properties](https://doc.rust-lang.org/std/primitive.char.html)
and the [category definitions](https://docs.rs/unicode-general-category/1.1.0/unicode_general_category/).

## Scalar comparison

`text.unicode.compare` aligns scalars by index, without edit-distance realignment.
Each original difference retains both sides' own characters, code points and
scalar/byte offsets. A missing side is null.

| Normalization | Equality result | Difference positions |
| --- | --- | --- |
| None, the default | `normalized_equal=null` | Original input offsets |
| Explicit NFC, NFD, NFKC or NFKD | Adds `normalized_equal` | Adds `normalized_scalar_index` positions in normalized buffers |

Normalized offsets are never attached to the original strings. Original
differences remain available.

## Listings and limits

Counts and equality remain exact when the listing is truncated.
`complete=false` identifies omitted records; exhausting a processing budget
instead fails the operation.

| Resource | Default or maximum |
| --- | --- |
| Listing | Default 1,000; maximum 65,536; zero means counts only |
| Comparison listing order | Original differences before normalized differences, within one shared budget |
| Original input | 1 MiB per input |
| Normalized output | 8 MiB per input; 16 MiB combined |
| Decomposition preflight | 2,097,152 scalars per input |
| Scratch memory | 128 MiB, including iterator queues and capacity slack |
| Serialized report | 8 MiB |
| Work | 134,217,728 units |

Work charges input bytes, scalar visits and decomposition preflight, plus 1,536
units per retained record and 2,048 units for report headers. Options may lower
the limits. These reports have no pipeline adapters.

See [Inspect text and Unicode](../../guides/text-evidence.md).
Automatic numeric-grid remapping remains a [research proposal](https://github.com/cubearc/cube/issues/127).
