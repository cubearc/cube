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

## Reproduce an Enigma message

Save these explicit settings as `settings.json`:

```json
{"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[]}
```

Run the reciprocal transform:

```sh
cube cipher enigma --parameters settings.json --text AAAAA
```

The JSON result contains `text: "BDZGO"`, `final_windows: "AAF"` and an empty
`trace`. Running `BDZGO` with the same original settings returns `AAAAA`.
See [Enigma settings and stepping](../reference/topics/enigma.md) for naval M3,
explicit preservation of case/group separators and optional state traces.

<a id="cli"></a>
<a id="python"></a>
<a id="use-an-explicit-classical-cipher"></a>
