# Use a classical cipher

Choose the cipher and provide its shift or key:

```sh
cube cipher caesar --shift 3 'Attack at dawn!'
# Dwwdfn dw gdzq!
cube cipher vigenere 'ATTACKATDAWN' -k LEMON
# LXFOPVEFRNHR
```

Caesar moves each letter by a fixed shift. Vigenère uses the supplied key to vary
the shift. Classical ciphers are useful for challenges and education, but do not
provide secure encryption.

Check the selected profile before interpreting punctuation, letter case or padding:

- [Classical cipher rules](../reference/topics/classical-ciphers.md)
- [Score classical candidates](../reference/topics/cryptanalysis.md)
- [Monoalphabetic profiles](../monoalphabetic.md)
- [Historical ciphers](../ciphers.md)
- [Transposition and fractionation](../cipher-permutations.md)

<a id="cli"></a>
<a id="python"></a>
<a id="use-an-explicit-classical-cipher"></a>
