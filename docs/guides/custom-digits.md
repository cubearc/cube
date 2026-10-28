# Convert a custom digit alphabet

Use a chosen set of symbols as positional digits.
Here, `abcdefghij` stands for decimal digits 0 through 9:

```sh
cube number alphabet --source-alphabet 01 --target-alphabet abcdefghij \
  --from-base 2 --to-base 10 --text 1010
# ba
```

Binary `1010` is decimal 10. In the target alphabet, `b` means 1 and `a` means 0,
so the result is `ba`.

See [custom numeric alphabets](../reference/topics/custom-numeric-alphabets.md)
for signs, token rules and valid bases.

<a id="cli"></a>
<a id="custom-positional-digits"></a>
<a id="custom-positional-digits-1"></a>
<a id="python"></a>
<a id="reference"></a>
