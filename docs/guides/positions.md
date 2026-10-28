# Transform selected text positions

Apply one transformation at selected positions and another to the remaining text.
This example selects Fibonacci positions, excluding 13 and 21:

```sh
cube text transform masked --text ABCDEFGHIJKLMNOPQRSTUVWXYZ \
  --selector fibonacci --exclude 13,21 --index-base 1 \
  --selected atbash --otherwise caesar --otherwise-shift 3
# ZYXGVIJSLMNOPQRSTUVWXYZABC
```

Positions 1, 2, 3, 5 and 8 use Atbash. The other letters use a Caesar shift of 3.

Positions count Unicode scalars, which can differ from visible characters:
an accented letter written with a combining mark occupies two positions.
See [position selection and overrides](../reference/topics/position-aware-transformations.md)
for selectors, result records and limits.

<a id="cli"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="transform-by-scalar-position"></a>
<a id="transform-by-scalar-position-1"></a>
