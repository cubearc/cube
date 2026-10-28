# Decode puzzle symbols

Choose a mapping that matches the clue. A result depends on that choice;
Cube does not infer the intended cipher.

## Bacon and Polybius

```sh
cube cipher bacon --variant extended26 --text AB
# AAAAA AAAAB
cube cipher bacon --variant classical24 --decode --text 'ABAAA BAABB'
# IU
cube cipher polybius --preset tap --text 'CK J'
# 1,3 1,3 / 2,5
```

The Bacon variant determines the alphabet. The tap-code square maps `C` and `K`
to the same coordinates, so that distinction cannot be recovered from the code.

## Keypad mappings

```sh
cube encode keypad --mode multi_tap --text HELLO
# 44|33|555|555|666
cube encode keypad --mode telephone --text HELLO
# 43556
cube decode keypad --mode telephone --text 4
# {"text":null,"alternatives":["GHI"],"reversible":false}
```

Multi-tap records how many times each key is pressed. Telephone mode records only
the key, so decoding `4` leaves three alternatives.

See [symbol mappings](../reference/topics/puzzle-ciphers-and-symbolic-mappings.md)
for custom alphabets, keyboard layouts and character classes. For anagrams,
probability or spatial puzzles, use the [puzzle model reference](../puzzle-analysis.md).

<a id="analyze-a-supplied-puzzle-model"></a>
<a id="bounded-lexical-probability-and-spatial-analysis"></a>
<a id="caller-supplied-puzzle-models"></a>
<a id="cli"></a>
<a id="puzzle-mappings"></a>
<a id="puzzle-mappings-and-evidence"></a>
<a id="python"></a>
<a id="reference"></a>
