# Text operations

`text.combinations` is a bounded incremental Cartesian product of ordered string
groups, with rightmost index varying fastest. It preserves duplicates, empty
strings and Unicode/control data. Zero groups yield one empty string; any empty
group yields zero candidates. Limits: 16 groups, 4096 fragments, 1 MiB total input,
64 KiB fragments/results, 4096 separator bytes, finite result cap default 1000/hard
10000, 16 MiB aggregate raw and escaped JSONL output, and 32 MiB work units.
Preflight checks count overflow, exact aggregate lengths and JSON escaping costs;
work includes input validation, output bytes and fragment/concatenation units.
Excessive plans fail without a prefix. Rust/Python generate one result at a time;
CLI streams unambiguous JSONL. Existing text/byte transforms accept each result
through explicit caller conversion; no candidate engine or pipeline iterator type
is introduced. JSON transport retains its existing depth/node/byte limits.

Grid factors and rendering retain Unicode scalars, not grapheme clusters. Input
is capped at 1 MiB and 65536 retained scalars; optional Rust White_Space removal
reports removed/retained counts using `WORD_UNICODE_VERSION`. Factors are ascending
width, include both orientations within 16384 per side, and reject excessive
candidate counts (default/hard cap 128). Explicit widths must divide the count.
Placement reuses existing traversal coordinates. Plain text uses JSON-quoted cells,
tabs and LF rows; its bounded scalar count keeps expansion below 1 MiB.
Color rendering requires exactly one explicit RGBA tuple per distinct scalar
(<=256), with no unused/missing/duplicate mappings; PNG construction is reused.
No fonts, inference or pipeline grid value is introduced. CLI mappings use a strict
JSON array of `[scalar, eight Hex digits]`, limited to 8192 bytes; Python uses a dict.

Text statistics count Unicode scalars and overlapping n-grams (`n=1..16`). Defaults
are no normalization, preserved case and all scalars; ASCII-only lowercasing and
letter filtering are explicit choices. Counts sort by descending frequency, then
lexical text order. IC is `sum(c*(c-1))/(N*(N-1))`, undefined below two scalars;
Shannon entropy is `-sum(p*log2(p))` bits/scalar, undefined for empty input and zero
for a constant distribution. Both measures always use single-scalar counts, even
when n-grams are requested. Floating results use f64 (allow numerical tolerance).

Statistics accept 1 MiB UTF-8 input, reuse normalization's 8 MiB output cap, then
limit the retained sample to 1 Mi scalars. Work is limited to 8 Mi scalar-window
units; character and n-gram maps each allow at most 10000 distinct values. Exceeding
these caps fails explicitly. `max_results` limits the sorted listing and `complete`
discloses omitted rows; total windows/distinct counts remain exact. These are
observations, not cipher identification. Structured results use direct APIs/JSON;
no text-statistics pipeline value or lossy string adapter is introduced.

`text.select` extracts first/last Unicode scalars per word or line. Words split
on Unicode whitespace and keep punctuation. Lines follow LF/CRLF boundaries;
bare CR, leading/trailing spaces and punctuation remain data. Empty lines add no
character; `line_first` is an acrostic without trimming. Combining marks and emoji
sequences are scalars, not grapheme clusters. The word tokenizer's Unicode version
is exposed as `WORD_UNICODE_VERSION`, separately from normalization data.
Input/output are bounded to 1 MiB. Positional and every-Nth selection still use
the existing `text.extract` semantics; the new selector also has a text adapter.

Use `text` for explicit normalization, reversal and positional extraction, rather
than treating arbitrary data as UTF-8.

```sh
cube text reverse --text 'Cube 🧊'
# 🧊 ebuC
cube text extract --start 1 --stride 2 --text abcdef
# bdf
```

Positions/reversal use Unicode scalar values, not UTF-8 byte offsets or visual
characters: combining marks and emoji sequences can split. Ordered index lists
may repeat positions; stride selections have an exclusive stop and reject out-of-
range bounds. Nothing is silently trimmed, clipped or normalized.

NFC/NFD/NFKC/NFKD normalization is explicit and may lose distinctions. Check
`cube.UNICODE_VERSION` for the SDK's data version. Text input is capped at 1 MiB;
normalization output at 8 MiB. `text.utf8.encode`/`text.utf8.decode` convert types
without replacement or normalization; invalid UTF-8 fails.
