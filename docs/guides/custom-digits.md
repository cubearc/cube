# Convert a custom digit alphabet

Supply the positional alphabet and base explicitly. Review the reference for sign, token and canonical representation rules.

## CLI

### Custom positional digits

```sh
cube number alphabet --source-alphabet 01 --target-alphabet abcdefghij \
  --from-base 2 --to-base 10 --text 1010
# ba
```


## Python

### Custom positional digits

```python
import cube

assert cube.convert_alphabet("1010", "01", "abcdefghij", 2, 10) == "ba"
assert cube.convert_alphabet("-٠٠١٠", "٠١٢٣٤٥٦٧٨٩", "01", 10, 2) == "-1010"
```


## Reference

- [Custom numeric alphabets](../reference/topics/custom-numeric-alphabets.md)
