# Signed positions inside words

Extract selected Unicode scalars from each word with `text.word.extract`.
Words split on Unicode whitespace; punctuation remains part of the word.
Combining marks count separately from the characters they modify.

## Position rules

Choose one ordered list for all words or a separate list for every word.
Requests preserve order and duplicates; lists do not cycle between words.

| Setting | Default | Behavior |
| --- | --- | --- |
| Positive origin | 1 | Position 0 is invalid; origin 0 is available explicitly |
| Negative positions | — | `-1` is last, `-2` is penultimate |
| `short_word` | `error` | Missing positions fail |
| `short_word=skip` | — | Keeps a null selection and increments `omitted_count`; inserts no placeholder |

## Results

Reports retain word boundaries, global scalar indexes and UTF-8 byte offsets.
Empty text with shared positions, or an empty per-word sequence, produces an
empty report.

## Limits

| Resource | Maximum |
| --- | --- |
| Input | 1 MiB |
| Words and total coordinates | 65,536 |
| Source index | Less than 8 MiB |
| Serialized report | 8 MiB |
| CLI position JSON | 2 MiB |

The index uses shared word slices and scalar-to-byte offsets. Each requested
coordinate resolves in constant time without rescanning a word. Verbose reports
can reach the output bound before the coordinate bound.

This operation has no pipeline adapter.
See [Select letters from words](../../guides/word-positions.md).
