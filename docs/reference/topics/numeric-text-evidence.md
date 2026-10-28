# Numeric text evidence

`text.word.lengths` reports Unicode-scalar lengths and original word boundaries.
`text.codepoints` includes every source scalar, including whitespace, as an exact
u32 value with scalar/UTF-8 byte offsets. Its decimal/hex/unicode presentation
field changes only display (unicode means U+XXXX), never the numeric value.
`text.word.values` retains per-character records inside each whitespace-delimited
word, defaulting to Unicode code points. Punctuation remains data; there is no
normalization, implicit A1Z26, case folding or concatenation of decimal values.

Custom value tables contain <=4,096 unique exact scalar/u64 pairs and <=64 KiB
serialized JSON. Unmapped scalars default to errors; explicit skip retains null
values and omitted counts. Optional checked u64 sums include mapped values only,
retain all character provenance and obey an inclusive max_sum (default u64::MAX).
Overflow is an error; non-default max_sum requires sums. JSON writes exact integer
digits and the native SDK returns exact Python ints, including u64::MAX.
Input <=1 MiB, word/code-point/total-value records <=65,536, source index <8 MiB,
and serialized report <=8 MiB. Report bounds can reject requests within record
bounds. All three are direct structured APIs without pipeline adapters.
