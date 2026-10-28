# Numeric conversions

Convert an integer between textual bases or select exact digits of a mathematical
constant. Integer bases are separate from byte encodings and byte order.

## Integer bases

| Rule | Behavior |
| --- | --- |
| Bases | Explicit source and destination, each from 2 to 36 |
| Digits | ASCII 0–9/A–Z |
| Sign | One optional leading `+` or `-` |
| Magnitude | At most `2^128 - 1`, with exact arithmetic |
| Input length | At most 4,096 digits |
| Invalid input | Whitespace, radix prefixes, separators, fractions and floats |
| Output | Canonical case, sign and leading zeros; value preserved |

For example, convert hexadecimal to binary or a signed base-36 integer to decimal:

```sh
cube number base --from-base 16 --to-base 2 --text FF
# 11111111
cube number base --from-base 36 --to-base 10 --text=-Z
# -35
```

For other integer operations, see [Modular arithmetic](exact-modular-arithmetic.md)
and [Digital root](decimal-digital-root.md).

## Constant digits

`number.constant_digits` selects exact fractional decimal digits of `pi`, `e`,
`phi` (the golden ratio) or `sqrt2`. Index 0 is the first digit after the decimal
point. Results include the integer part separately, selected text and a SHA-256
of the complete generated prefix.

### Selection and budgets

| Setting | Default | Rule |
| --- | --- | --- |
| Range start | 0 | Zero-based fractional position |
| Range count | 16 | Zero permits a start at the prefix's end |
| Index list | None | Preserves order and repeats; an empty list is valid |
| `max_digits` | 256 | 1–4,096; controls the full generated prefix |
| Index-list length | — | At most 1,024 |
| CLI selector text | — | At most 8 KiB |
| Structured output | — | Less than 16 KiB |

Ranges and index lists are mutually exclusive. Out-of-range positions and range
overflow fail; there is no partial prefix or silent truncation. A small selection
still generates and certifies the full `max_digits` prefix.

### Exactness and computation

Generation uses pure Rust `num-bigint` arithmetic, without a reference dataset or
native backend.

| Constant | Certification method |
| --- | --- |
| `pi` | Alternating-series intervals for `16 atan(1/5) - 4 atan(1/239)` |
| `e` | Factorial terms with a strict tail bound |
| `sqrt2`, `phi` | Exact integer-root intervals |

Sixteen fixed guard places support the bounds. A result is returned only when
both interval endpoints truncate to the same integer. Uncertainty fails
explicitly; binary floating approximations and caller-selected guard precision
are not used.

Computation is limited to 32,768-bit integers and 8,192 series steps. Scale and
root sizes are checked before allocation. These are work and size limits, not a
wall-clock timeout. All four maximum prefixes are checked against independent
Fraction/Decimal calculations.

This structured, parameter-only operation has no pipeline adapter.

### Design background

Runtime generation is documented in [#134](https://github.com/cubearc/cube/issues/134),
following the [original evaluation](https://github.com/cubearc/cube/issues/107#issuecomment-6398938938).
Prime and Fibonacci proposals remain deferred; see
[#77](https://github.com/cubearc/cube/issues/77),
[#129](https://github.com/cubearc/cube/issues/129) and
[#131](https://github.com/cubearc/cube/issues/131).
