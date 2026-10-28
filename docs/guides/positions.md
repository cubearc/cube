# Transform selected text positions

Select explicit, periodic or Fibonacci scalar positions, then apply a documented transform. Positions refer to Unicode scalars; review the reference before using combining characters or overrides.

## CLI

### Transform by scalar position

```sh
cube text transform masked --text ABCDEFGHIJKLMNOPQRSTUVWXYZ \
  --selector fibonacci --exclude 13,21 --index-base 1 \
  --selected atbash --otherwise caesar --otherwise-shift 3
# ZYXGVIJSLMNOPQRSTUVWXYZABC
```


## Python

### Transform by scalar position

```python
import cube

result = cube.transform_masked(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    selector="fibonacci", exclude=[13, 21], index_base=1,
    selected="atbash", otherwise="caesar", otherwise_shift=3,
)
assert result["selected_positions"] == [1, 2, 3, 5, 8]
assert result["deterministic_text"] == "ZYXGVIJSLMNOPQRSTUVWXYZABC"
assert result["final_text"] == result["deterministic_text"]
```


## Reference

- [Position-aware transformations](../reference/topics/position-aware-transformations.md)
