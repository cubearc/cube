# Work with numbers

Calculate with integers or select digits from supported mathematical constants.

## Modular arithmetic

```sh
cube number gcd --a 48 --b 18
# 6
cube number mod-inverse --a 3 --modulus 11
# 4
cube number mod-pow --base 4 --exponent 13 --modulus 497 --json
```

The inverse is 4 because `3 × 4` leaves remainder 1 modulo 11.
The modular power computes `4^13 mod 497`, which is 445.

## Digital roots

```sh
cube number digital-root --text=-38
# 2
cube number digital-root --text +000999 --radix 10 --json
```

The sign does not affect the digital root. For 38, `3 + 8 = 11` and `1 + 1 = 2`.
For 999, the result is 9.

## Digits of a constant

```sh
cube number digits pi --count 8
# 14159265
cube number digits sqrt2 --indices 2,0,2 --json
```

Positions refer to digits after the decimal point and start at zero.
The second example selects digits 2, 0 and 2 from √2, producing `444`.

See [integer arithmetic](../reference/topics/exact-modular-arithmetic.md),
[digital roots](../reference/topics/decimal-digital-root.md) and
[numeric conversions](../reference/topics/numeric-conversions.md) for accepted
ranges and exact result formats.

<a id="cli"></a>
<a id="compute-exact-modular-arithmetic"></a>
<a id="decimal-digital-root"></a>
<a id="decimal-digital-root-1"></a>
<a id="exact-modular-arithmetic"></a>
<a id="exact-modular-arithmetic-1"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="select-mathematical-constant-digits"></a>
<a id="select-mathematical-constant-digits-1"></a>
