# Measure text and Unicode evidence

Measure supplied text and inspect exact scalar views. Counts and differences are observations; explicit conversion or interpretation remains a separate step.

## CLI

### Numeric text evidence

```sh
cube text word-lengths --text 'A é'
cube text codepoints --format unicode --text 'A é'
cube text word-values --sums --text 'A é'
cube text word-values --sums --table '[["A",1],["B",2]]' --text AB
```


### Inspect Unicode and compare scalar views

```sh
printf 'A' > saved-text.txt
cube text unicode-inspect --text 'Aaé' --max-results 1000
cube text unicode-compare --text 'é' --other 'é' --normalization NFC
cube text unicode-compare --text A --other-file saved-text.txt
```


## Python

### Numeric text evidence

```python
import cube

assert [w["scalar_length"] for w in cube.word_lengths("A é")] == [1, 1]
assert [c["value"] for c in cube.text_codepoints("A é", format="unicode")] == [65, 32, 233]
assert [w["sum"] for w in cube.word_values("A é", sums=True)] == [65, 233]
assert cube.word_values("AB", table=[("A", 1), ("B", 2)], sums=True)[0]["sum"] == 3
```


### Inspect Unicode and compare scalar views

```python
report = cube.inspect_unicode("A\u200d", options={"max_results": 0})
assert report["counts"]["format"] == 1 and not report["complete"]
comparison = cube.compare_unicode("é", "e\u0301", normalization="NFC")
assert not comparison["exact_equal"] and comparison["normalized_equal"]
assert comparison["original"]["records"][1]["left"] is None
assert comparison["normalized"]["coordinate_space"] == "normalized_scalar_index"
```


## Reference

- [Numeric text evidence](../reference/topics/numeric-text-evidence.md)
- [Unicode observations and scalar differences](../reference/topics/unicode-observations-and-scalar-differences.md)
