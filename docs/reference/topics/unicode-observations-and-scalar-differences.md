# Unicode observations and scalar differences

`text.unicode.inspect` records original scalars/code points/offsets, case labels,
non-ASCII/control/whitespace/format flags and named zero-width candidates. Derived
Uppercase/Lowercase/White_Space properties use Rust's separately reported Unicode
version; titlecase and General_Category (including Cf Format) use pinned
`unicode-general-category` 1.1.0 / Unicode 16.0 data. The dependency provides
maintained complete categories without a hand-written category table. Changes to
this pinned dataset require a compatibility review. Unicode property definitions
are described by [Rust char](https://doc.rust-lang.org/std/primitive.char.html) and
[the category crate](https://docs.rs/unicode-general-category/1.1.0/unicode_general_category/).
Zero-width profile 1 lists U+200B..200D, U+2060, U+FEFF and variation selectors
U+FE00..FE0F/U+E0100..E01EF. Bidi-control observations list ALM, LRM/RLM,
U+202A..202E and U+2066..2069. These flags observe code points, not rendered
visibility, font behavior or security verdicts. No case folding is performed.

`text.unicode.compare` aligns stored scalars by index, without edit-distance
realignment. Original differences always retain each side's own byte/scalar
offsets, characters and numeric code points; absent sides are null. Default
normalization is none, with normalized_equal=null. Explicit NFC/NFD/NFKC/NFKD
reuses the existing versioned normalizer and adds normalized_equal plus a
`normalized_scalar_index` listing. Those offsets belong to normalized buffers;
they are never attached to original strings. Original differences remain intact.

Default listing is 1,000, maximum 65,536, zero counts-only. Comparison shares one
listing budget, prioritizing original then normalized differences. Scans always
compute exact counts/equality; limited listings have complete=false. Budget
exhaustion instead returns an error, never a partial success. Originals each
allow 1 MiB; normalized output each allows 8 MiB and preflight allows 2,097,152
decomposed scalars per input. This bounds pending normalizer queues as well as
retained text: normalized strings <=16 MiB together; a conservative 128 MiB
scratch bound includes iterator queues/capacity slack. Serialized reports <=8 MiB
are independently bounded. Work <=134,217,728 units charges input bytes, scalar
visits and decomposition preflight, 1,536 conservative units per retained record
(including bounded serialization), and 2,048 report-header units. Limits may be
lowered through concrete options. These direct reports have no pipeline adapter;
existing normalization, binary comparison and recipe contracts are unchanged.

Numeric-grid research in [#127](https://github.com/cubearc/cube/issues/127) records
reuse of existing public coordinate traversal and explicit small rank mappings.
It defers an automatic grid remapper; no additional matrix engine or placeholder
operation is included in 0.17.0.
