### Numeric conversions

Use `numbers` to change an integer's textual base, separately from byte encodings
and byte order.

```sh
cube number base --from-base 16 --to-base 2 --text FF
# 11111111
cube number base --from-base 36 --to-base 10 --text=-Z
# -35
```

Bases are explicit 2..36. Digits use ASCII 0–9/A–Z; a leading sign is allowed,
while whitespace, radix prefixes, separators, fractions and floats are rejected.
Magnitudes up to `2^128-1` are exact, with at most 4096 input digits. Output
canonicalizes case, sign and leading zeros; the integer value is preserved.

The v0.11.0 numeric evaluation is recorded in [issue #77](https://github.com/cubearc/cube/issues/77).
Its prime/Fibonacci proposals remain deferred; the 0.18.0 scope extends its
fixed-width approach with the [modular utilities](exact-modular-arithmetic.md#exact-modular-arithmetic) and
[decimal digital root](decimal-digital-root.md#decimal-digital-root). Research decisions in
[#129](https://github.com/cubearc/cube/issues/129) and
[#131](https://github.com/cubearc/cube/issues/131) retain explicit prime/candidate
engine deferrals and reuse existing composition. Proposed operations are not
advertised through discovery.

`number.constant_digits` selects exact fractional decimal digits of `pi`, `e`,
`phi` (the golden ratio) or `sqrt2`. Index 0 excludes the integer part/decimal point;
results return the integer part separately, exact selected text and the entire
prefix's SHA-256. Range defaults are start 0/count 16; an explicit index list
preserves order/repeats and may be empty. Ranges and lists are exclusive. A
zero-count range may start at the prefix's end; other out-of-range positions fail.

The explicit `max_digits` budget defaults to 256 and accepts 1..4096. It controls
the complete generated/certified prefix, even when a small selection is requested.
Increasing it permits additional positions while retaining the hard cap. Range
addition is checked; lists have at most 1024 entries and CLI selector text is
capped at 8 KiB. Inputs are integers, not fractional positions or precision guesses.
Structured output is below 16 KiB; no partial prefix or silent truncation of an
invalid selection is returned.

The [original research](https://github.com/cubearc/cube/issues/107#issuecomment-6398938938)
proposed a fixed offline reference. The [approved override follow-up](https://github.com/cubearc/cube/issues/134)
instead uses bounded runtime generation in the Rust core, with maintained pure
Rust `num-bigint` arithmetic and no native backend/reference dataset. pi uses
`16 atan(1/5)-4 atan(1/239)` with alternating-series intervals; e uses factorial
terms and a strict tail bound; sqrt2/phi use exact integer-root intervals. Sixteen
fixed guard places support these bounds, but only agreement of both endpoints'
truncated integers certifies a result. Uncertainty fails explicitly. No binary
floating approximation or caller-controlled guard precision is used.

Validated prefix lengths bound scale/root construction before allocation;
integers are capped at 32768 bits and series work at 8192 steps. One integer root
operates on a bounded radicand. These are deterministic work/size limits, not a
wall-clock timeout. All four maximum prefixes match independent Fraction/Decimal
checks. Rust, CLI and native Python share the implementation. No pipeline adapter
is advertised for this structured parameter-only operation.
