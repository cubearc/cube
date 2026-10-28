# Exact modular arithmetic

`number.gcd`, `number.extended_gcd`, `number.mod_inverse` and `number.mod_pow`
operate on nonnegative u64 values. CLI operands are required flags with 1..20
ASCII decimal digits, including leading zeros, without signs, whitespace or
prefixes. Python accepts exact builtin `int` values in 0..2^64-1; booleans,
floats and coercions are rejected. GCD(0,0)=0. Extended GCD returns gcd/x/y
with a*x+b*y=gcd, using floor Euclidean quotients starting from coefficient
pairs (1,0)/(0,1); (0,0) returns all zeros, (a,0) has x=1/y=0 and (0,b)
has x=0/y=1. Coefficients are signed i128 in Rust and exact Python integers.

Inverse requires modulus>=2 and coprime operands; otherwise it returns a typed
error. Power requires modulus>0, allows a full u64 exponent and defines a^0,
including 0^0, as 1 mod modulus. Modulus 1 yields zero. Results are canonical
residues. Scalar CLI output is decimal text; `--json` returns `{"value":"445"}`.
Extended JSON contains decimal strings for gcd/x/y. This preserves exact values
for JSON consumers with floating-point numeric types; the SDK returns integers.

Euclidean routines enforce at most 256 rounds, use fixed storage and check i128
coefficient arithmetic. Alternating coefficient signs bound each generated
coefficient and quotient product by max(a,b)/gcd(a,b), including the terminal
zero-remainder pair. Modular products fit u128 and exponentiation uses at most
64 bit rounds. Successful scalar output is at most 20 decimal digits; extended
output has at most 20 digits for gcd and 21 characters per signed coefficient
(under 128 bytes including JSON framing). No input-dependent arithmetic
allocation or new dependency is required. These local mathematical tools make no
constant-time cryptographic guarantee. They are direct APIs with no pipeline
adapter; engine API 1 and recipe format 1 are unchanged.
