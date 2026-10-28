# Select letters from words

Extract the first and third character from each word:

```sh
cube text word-extract --positions '[1,3]' --text 'Cube Uses Binary Evidence'
# CbUeBnEi
```

Positive positions count from the start of a word; negative positions count from
its end. To use a different selection for each word:

```sh
cube text word-extract --per-word '[[2,1,2],[-1]]' --text 'ab cd'
# babd
```

The first selection repeats position 2. The second takes the last character of
`cd`.

## Handle short words

```sh
cube text word-extract --positions '[2]' --short-word skip --text 'A bé'
# é
```

`A` has no second character, so `skip` omits it. The report retains the omission
and source positions. See [word and position rules](../reference/topics/signed-positions-inside-words.md)
for word boundaries and other short-word policies.

<a id="cli"></a>
<a id="positions-inside-every-word"></a>
<a id="positions-inside-every-word-1"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="select-positions-inside-words"></a>
