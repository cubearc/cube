# Decimal digital root

`number.digital_root` returns 0 for zero and 1+((magnitude-1) mod 9) otherwise.
The sign is ignored, including negative magnitudes beyond the signed i128 range;
all magnitudes through 2^128-1 are exact. For example, 38 and -38 yield 2,
999 yields 9 and 10 yields 1. This is repeated decimal digit summation, not the
ordinary digit sum or n mod 9, and it has no inverse or checksum guarantee.

CLI input is strict decimal text with an optional leading ASCII +/- and <=4,096
digits (<=4,097 bytes including a sign). Leading zeros count; empty/sign-only,
whitespace (including a final newline), fractions, prefixes, separators and
non-ASCII digits fail. Magnitude overflow errors even if its digital root could
be computed without full parsing. One pass parses at most 4,096 digits using the
existing sign/digit and checked 128-bit magnitude primitives; root arithmetic
then uses constant storage and no heap allocation. Output is one digit, or the
single numeric field `{"value":2}` under `--json`. Native Python accepts builtin
signed integers with the same magnitude bound and rejects bool/float/string
coercion. Rust accepts a u128 magnitude or the bounded decimal parser.

Radix defaults to 10 and must equal 10 in every interface. Generalized radices
remain deferred: no concrete workflow justifies another numeric interpretation
or supported-range contract. Existing base conversion and its grammar remain
unchanged. The root operation is discoverable directly and has no pipeline
adapter; it does not change engine/recipe versions.
