# Select positions inside words

Select signed positions under the documented word rules. Keep source-position evidence when interpreting a clue.

## CLI

### Positions inside every word

```sh
cube text word-extract --positions '[1,3]' --text 'Cube Uses Binary Evidence'
cube text word-extract --per-word '[[2,1,2],[-1]]' --text 'ab cd'
cube text word-extract --positions '[2]' --short-word skip --text 'A bé'
```


## Python

### Positions inside every word

```python
import cube

assert cube.extract_word_characters("Cube Uses Binary Evidence", positions=[1, 3])["extracted"] == "CbUeBnEi"
assert cube.extract_word_characters("ab cd", per_word=[[2, 1, 2], [-1]])["extracted"] == "babd"
report = cube.extract_word_characters("A bé", positions=[2], short_word="skip")
assert report["extracted"] == "é" and report["omitted_count"] == 1
```


## Reference

- [Signed positions inside words](../reference/topics/signed-positions-inside-words.md)
