# Analyze a supplied puzzle model

Choose a defined mapping, caller dictionary or explicit finite model. Results preserve ambiguity and the assumptions supplied by the caller.

## CLI

### Puzzle mappings

```sh
cube cipher bacon --variant extended26 --text AB
# AAAAA AAAAB
cube cipher bacon --variant classical24 --decode --text 'ABAAA BAABB'
# IU
cube cipher polybius --preset tap --text 'CK J'
# 1,3 1,3 / 2,5
cube cipher polybius --alphabet 'αβγδ' --side 2 --index-base 0 --text 'δ α' --json
# [[1,1],null,[0,0]]
```


```sh
cube text keyboard --source qwerty --target dvorak --text qwerty
# ',.pyf
cube encode keypad --mode telephone --text HELLO
# 43556
cube encode keypad --mode multi_tap --text HELLO
# 44|33|555|555|666
cube decode keypad --mode telephone --text 4
# {"text":null,"alternatives":["GHI"],"reversible":false}
cube text class --unmapped filter --text 'éa!B'
```


```sh
cube encode keypad --mode multi_tap --text '0 ' \
  --mapping '{"ascii_fold":false,"keys":[{"digit":"0","symbols":" 0"}]}'
# 00|0
cube text class --text 'αβ' \
  --definition '{"ascii_fold":false,"classes":[{"name":"greek","characters":"αβ","symbol":"!"}]}'
```



### Caller-supplied puzzle models

Use the linked reference for the supported model and its examples.

## Python

### Puzzle mappings and evidence

```python
import cube

assert cube.bacon_encode("AB", "extended26") == "AAAAA AAAAB"
assert cube.bacon_decode("ABAAA BAABB", "classical24") == "IU"
assert cube.polybius_encode("CK", preset="tap") == [[1, 3], [1, 3]]
assert cube.polybius_decode([[1, 3]], preset="tap") == "C"
assert cube.polybius_encode("δ α", alphabet="αβγδ", side=2, index_base=0) == [
    [1, 1], None, [0, 0]
]
assert cube.keyboard_remap("yYzZ", "qwerty", "qwertz") == "zZyY"
assert cube.keypad_encode("HELLO", "multi_tap") == "44|33|555|555|666"
assert cube.keypad_decode("4", "telephone") == {
    "text": None, "alternatives": ["GHI"], "reversible": False
}
report = cube.map_character_classes("éa!B", unmapped="filter")
assert report["mapped_text"] == "01"
assert report["records"][0]["scalar_index"] == 1
assert report["records"][0]["byte_offset"] == 2
assert report["filtered_count"] == 2
```


```python
mapping = {"ascii_fold": False, "keys": [{"digit": "0", "symbols": " 0🧊"}]}
assert cube.keypad_encode("0 🧊", "multi_tap", mapping=mapping) == "00|0|000"
definition = {
    "ascii_fold": False,
    "classes": [{"name": "greek", "characters": "αβ", "symbol": "!"}],
}
assert cube.map_character_classes("αβ", definition=definition)["mapped_text"] == "!!"
source = {"geometry": "one", "entries": [{"key": "k", "layer": "base", "symbol": "α"}]}
target = {"geometry": "one", "entries": [{"key": "k", "layer": "base", "symbol": "🧊"}]}
assert cube.keyboard_remap("α", source, target) == "🧊"
```


### Bounded lexical, probability and spatial analysis

Use the linked reference for the supported model and its examples.

## Reference

- [Puzzle ciphers and symbolic mappings](../reference/topics/puzzle-ciphers-and-symbolic-mappings.md)
- [Bounded puzzle analysis](../reference/topics/bounded-puzzle-analysis.md)

- [Dictionary, probability and spatial models](../puzzle-analysis.md)
- [Monoalphabetic profiles](../monoalphabetic.md)
- [Keyed historical profiles](../ciphers.md)
