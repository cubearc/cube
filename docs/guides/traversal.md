# Traverse text in an explicit order

Choose the traversal and its parameters explicitly. These operations visit scalar positions; they do not guess a cipher or normalize the text.

## CLI

### Explicit text traversal

```sh
cube text keyed-skip --steps 2 --count 5 --text ABCDE
cube text josephus --steps 2 --text ABCDE
```


## Python

### Explicit text traversal

```python
import cube

assert cube.keyed_skip("ABCDE", [2], 5)["extracted"] == "BDACE"
assert cube.josephus_extract("ABCDE", [2])["extracted"] == "BDAEC"
assert cube.keyed_skip("ABCDE", [1], 3, options={"start": 2, "index_base": 1})["extracted"] == "BCD"
```


## Reference

- [Keyed skip and Josephus traversal](../reference/topics/keyed-skip-and-josephus-traversal.md)
