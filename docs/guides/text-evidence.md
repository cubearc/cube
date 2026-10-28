# Inspect text and Unicode

Count characters, inspect code points, or compare text that looks the same but
has a different representation.

## Count and inspect characters

```sh
cube text word-lengths --text 'A é'
cube text codepoints --format unicode --text 'A é'
cube text word-values --sums --table '[["A",1],["B",2]]' --text AB
```

Both words have one Unicode scalar. The code points are `65, 32, 233`, including
the space. With the supplied value table, `AB` has a sum of 3.

## Compare representations

```sh
cube text unicode-inspect --text 'Aaé' --max-results 1000
cube text unicode-compare --text 'é' --other 'é' --normalization NFC
```

In the comparison, the left input is one scalar; the right input is `e` followed
by a combining acute accent. They are not exactly equal, but become equal under
NFC normalization. The report keeps both comparisons.

See [text measurements](../reference/topics/numeric-text-evidence.md) and
[Unicode comparison](../reference/topics/unicode-observations-and-scalar-differences.md)
for indexing, normalization and report limits.

<a id="cli"></a>
<a id="inspect-unicode-and-compare-scalar-views"></a>
<a id="inspect-unicode-and-compare-scalar-views-1"></a>
<a id="measure-text-and-unicode-evidence"></a>
<a id="numeric-text-evidence"></a>
<a id="numeric-text-evidence-1"></a>
<a id="python"></a>
<a id="reference"></a>
