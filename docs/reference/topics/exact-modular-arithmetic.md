# Exact modular arithmetic

Compute greatest common divisors, modular inverses and modular powers with
nonnegative 64-bit integers. These utilities do not provide a constant-time
cryptographic guarantee.

## Operations

The operations use the following rules:

| Operation | Result | Special cases |
| --- | --- | --- |
| GCD | Greatest common divisor of `a` and `b` | `gcd(0, 0) = 0` |
| Extended GCD | `gcd`, `x` and `y` such that `a*x + b*y = gcd` | `(0,0)` returns all zeros; `(a,0)` gives `x=1,y=0`; `(0,b)` gives `x=0,y=1` |
| Modular inverse | The canonical inverse residue | Requires modulus ≥ 2 and coprime operands; otherwise fails |
| Modular power | `base^exponent mod modulus` | Requires modulus > 0; `a^0`, including `0^0`, is `1 mod modulus`; modulus 1 gives zero |

Extended GCD uses floor Euclidean quotients from coefficient pairs `(1,0)` and
`(0,1)`. Coefficients use signed 128-bit integers in Rust and exact integers in Python.

## Input and output

| Interface | Input | Output |
| --- | --- | --- |
| CLI | Required flags containing 1–20 ASCII decimal digits, including leading zeros | Decimal text; JSON uses strings, such as `{"value":"445"}` |
| Python | Exact built-in `int` values in `0..2^64−1` | Exact integers |
| Rust | `u64` operands and exponent | Integer results; `i128` coefficients |

CLI signs, whitespace and prefixes fail. Python booleans, floats and coercions
fail. Extended CLI JSON contains decimal strings for `gcd`, `x` and `y` to preserve
precision in consumers that use floating-point numbers.

## Execution bounds

| Resource | Bound |
| --- | --- |
| Euclidean iterations | 256 rounds |
| Modular exponentiation | 64 bit rounds; products fit `u128` |
| Scalar output | 20 decimal digits |
| Extended output | 20 digits for GCD; 21 characters per signed coefficient; less than 128 bytes of JSON |

The routines use fixed storage and checked coefficient arithmetic.
Alternating signs bound coefficients and quotient products by `max(a,b)/gcd(a,b)`,
including the terminal zero-remainder pair.

These operations have no pipeline adapters.
See [Work with numbers](../../guides/arithmetic.md#modular-arithmetic) for examples.
