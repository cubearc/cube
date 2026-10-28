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

## Progress the Vigenère key

To increase the shift after each repetition of the key, save the key and
increment in a parameter file:

```sh
printf '%s' '{"key":"BC","increment":1}' > progressive.json
cube cipher progressive-key --parameters progressive.json --text AAAAA
# BCCDD
cube cipher progressive-key --parameters progressive.json --decode --text BCCDD
# AAAAA
```

The first two letters use `BC`, the next two use one additional shift, and the
last letter uses two. Case and non-ASCII characters are preserved. See
[key schedule rules](../ciphers.md#letter-preserving-schedules) for explicit
interruptions and the qualified multiplicative composition.

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
