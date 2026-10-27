# Compute exact modular arithmetic

Choose the exact integer operation and its explicit bounds. Large exact results use the documented decimal representation; no approximate answer is substituted.

## CLI

### Exact modular arithmetic

```sh
cube number gcd --a 48 --b 18
cube number extended-gcd --a 240 --b 46 --json
cube number mod-inverse --a 3 --modulus 11
cube number mod-pow --base 4 --exponent 13 --modulus 497 --json
```


### Decimal digital root

```sh
cube number digital-root --text=-38
cube number digital-root --text +000999 --radix 10 --json
```


### Select mathematical constant digits

```sh
cube number digits pi --count 8
cube number digits pi --start 256 --count 16 --max-digits 512 --json
cube number digits sqrt2 --indices 2,0,2 --json
```


## Python

### Exact modular arithmetic

```python
import cube

assert cube.gcd(48, 18) == 6
assert cube.extended_gcd(240, 46) == {"gcd": 2, "x": -9, "y": 47}
assert cube.mod_inverse(3, 11) == 4
assert cube.mod_pow(4, 13, 497) == 445
assert cube.mod_pow(0, 0, 1) == 0
```


### Decimal digital root

```python
assert cube.digital_root(-38) == 2
assert cube.digital_root(999, radix=10) == 9
assert cube.digital_root(-(2**128 - 1)) == 3
```


### Select mathematical constant digits

```python
from cube import constant_digits

assert constant_digits("pi", count=8)["digits"] == "14159265"
more = constant_digits("pi", start=256, count=16, max_digits=512)
assert more["integer_part"] == "3"
assert len(more["digits"]) == 16
assert constant_digits("sqrt2", indices=[2, 0, 2])["digits"] == "444"
```


## Reference

- [Exact modular arithmetic](../reference/topics/exact-modular-arithmetic.md)
- [Decimal digital root](../reference/topics/decimal-digital-root.md)
- [Numeric conversions](../reference/topics/numeric-conversions.md)
