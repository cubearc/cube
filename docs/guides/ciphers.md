# Use an explicit classical cipher

Choose the documented cipher and provide its key or shift. Classical ciphers are
useful for challenges and education; they do not provide secure encryption.

## CLI

```sh
cube cipher caesar --shift 3 'Attack at dawn!'
# Dwwdfn dw gdzq!
cube cipher vigenere 'ATTACKATDAWN' -k LEMON
# LXFOPVEFRNHR
```

## Python

```python
import cube

assert cube.caesar_encode("Attack at dawn!", 3) == "Dwwdfn dw gdzq!"
assert cube.vigenere_decode("LXFOPVEFRNHR", "LEMON") == "ATTACKATDAWN"
```

Use [cipher semantics](../reference/topics/classical-ciphers.md),
[monoalphabetic profiles](../monoalphabetic.md),
[keyed historical profiles](../ciphers.md) and
[transposition/fractionation profiles](../cipher-permutations.md)
for exact alphabets, canonicalization, schedules and visible padding.
