# Decimal digital root

Calculate the repeated decimal digit sum with `number.digital_root`.
The sign does not affect the result: 38 and −38 produce 2, 999 produces 9, and
10 produces 1.

## Calculation

The result is `0` for zero and `1 + ((magnitude − 1) mod 9)` otherwise.
This differs from a single digit sum and from `n mod 9`.
It has no inverse or checksum guarantee.

## Input

Every interface uses radix 10 and accepts magnitudes through `2^128 − 1`,
including negative magnitudes beyond the signed 128-bit range.

| Interface | Accepted input | Rejected input |
| --- | --- | --- |
| CLI | ASCII decimal digits with an optional leading `+` or `-` | Empty/sign-only text, whitespace or a trailing newline, fractions, prefixes, separators and non-ASCII digits |
| Python | Built-in signed `int` within the magnitude bound | Booleans, floats, strings and coercions |
| Rust | A `u128` magnitude or the bounded decimal parser | Magnitudes outside that range |

CLI input is limited to 4,096 digits, including leading zeros, and 4,097 bytes
with a sign. An overflowing magnitude fails even if its digital root would fit.

## Output and execution

The CLI returns one decimal digit; `--json` returns a numeric field such as
`{"value":2}`.

Parsing makes one pass over at most 4,096 digits with checked 128-bit arithmetic.
The root calculation uses constant storage and no heap allocation.
The operation has no pipeline adapter.

See [Work with numbers](../../guides/arithmetic.md#digital-roots) for examples.
