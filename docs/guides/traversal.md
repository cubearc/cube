# Read text in a different order

Choose a step size to visit positions in a circular string:

```sh
cube text keyed-skip --steps 2 --count 5 --text ABCDE
# BDACE
cube text josephus --steps 2 --text ABCDE
# BDAEC
```

Both start by selecting `B`. Keyed skip keeps every character in the circle;
Josephus removes each selected character before continuing. That difference
changes the later selections.

Positions count Unicode scalars. The operations keep the original text order
and do not normalize it. See [traversal rules](../reference/topics/keyed-skip-and-josephus-traversal.md)
for start positions, direction and multiple step sizes.

<a id="cli"></a>
<a id="explicit-text-traversal"></a>
<a id="explicit-text-traversal-1"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="traverse-text-in-an-explicit-order"></a>
