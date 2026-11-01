# Read directional symbols and grids

Use the selected symbol system to convert a clue into text or positions.

## Semaphore

```sh
cube encode semaphore --text 'A B'
cube decode semaphore --text 'S,SW / S,W'
```

The directions describe the two arms. The slash separates words; decoding
the second example returns `A B`.

## Named alphabet tokens

Select a dated source table when the clue uses Aurebesh or Standard Galactic.
Save `{"profile":"aurebesh_ucsur_2016_v1"}` as `alphabet.json`, then encode
explicit letter units:

```sh
cube encode alphabet --parameters alphabet.json --text '["A","CH"]'
```

The `tokens` in the JSON report are the private-use characters U+E890 and U+E8AA.
Rendering them as glyphs requires a matching font. The single unit `CH` selects
the digraph; `C` and `H` would produce two tokens. See
[dated symbolic alphabets](../reference/topics/symbolic-encodings.md#dated-symbolic-alphabets)
for the supported tables, punctuation, exact decoding and limits.

## Book positions

Create a short source text, then select the first character from each word:

```sh
printf 'Cube Uses Binary Evidence' > book-source.txt
cube cipher book --source book-source.txt --text '1:1,2:1,3:1,4:1'
# CUBE
```

The source text and position convention are part of the interpretation.

## Text grids

```sh
cube text grid --text ABCDEF --width 3
```

The grid has two rows, `ABC` and `DEF`. Width and reading order determine how
positions map back to the input.

See [symbolic encodings](../reference/topics/symbolic-encodings.md) for clock
directions and semaphore, and [text operations](../reference/topics/text-operations.md)
for grids and string combinations.

<a id="arrange-text-in-scalar-grids"></a>
<a id="cli"></a>
<a id="decode-symbolic-clues"></a>
<a id="extract-from-a-caller-supplied-book"></a>
<a id="generate-string-combinations"></a>
<a id="interpret-directional-symbols"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="work-with-grids-combinations-and-directional-symbols"></a>
