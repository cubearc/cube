# Unicode text profiles

Choose a text unit explicitly when case folding, segmenting, searching, arranging
or selecting text. Coordinates in reports are zero-based and half-open in the
original UTF-8 bytes and Unicode scalars. See [text units](../../concepts/text.md).

The CLI accepts one JSON request through `cube text profile <name>`. Pass the
same request object to the [Python SDK](../../interfaces/python.md#text-profiles)
or use the `request` input and `report` output ports of the version 1 direct API.

## Case transforms

| Transform | Rule |
| --- | --- |
| `ascii_upper` / `ascii_lower` | Change ASCII letters and preserve all other scalars |
| `casefold` | Use Unicode 17.0 default full folding. Choose `simple` for C+S or `turkic` for full folding with T overrides |
| `category_filter` | List unique Unicode 16.0 two-letter categories such as `Lu` and `Mn`. Default `include` keeps those categories, while `exclude` removes them |

Case folding can expand text. Category filtering preserves the order of the
selected scalars. Apply [normalization](text-operations.md#normalization-and-utf-8)
explicitly when needed. The transform reports its data version.

## Extended graphemes

Choose `segments` for source spans or `reverse` to concatenate source clusters
in reverse order. Both use [UAX #29 revision 47](https://www.unicode.org/reports/tr29/tr29-47.html)
and Unicode 17.0 extended clusters. Segmenting empty text returns an empty list.
Resegmenting the reversed output can produce different boundaries.

```sh
cube text profile graphemes --text '{"action":"reverse","text":"á🧑‍💻b"}'
```

The `text` field is `b🧑‍💻á`. Each cluster retains its original scalars.

## Regular expressions and tokens

Use `regex` with an explicit `action` and `pattern`. The profile uses
[regex-syntax 0.8.11](https://docs.rs/regex-syntax/0.8.11/regex_syntax/) and
[regex-automata 0.4.18 PikeVM](https://docs.rs/regex-automata/0.4.18/regex_automata/nfa/thompson/pikevm/).
The default is case-sensitive and Unicode-aware. The grammar lacks backreferences
and lookaround. An empty match advances to the next scalar boundary and can
also occur immediately after a nonempty match. A zero-width match at the end
terminates iteration.

| Action | Report |
| --- | --- |
| `exists` | Boolean result from one search |
| `matches` | Ordered nonoverlapping whole matches |
| `captures` | Whole matches and indexed groups. An unmatched group has `matched: null`. An empty captured group has a present zero-length span |
| `split` | Fragments between delimiters. Set `keep_empty: true` to keep empty fragments |
| `tokenize` (separate operation) | Existing `unicode_whitespace` tokens and spans, with the tokenizer's Unicode version |

Listing defaults to at most 1,000 results. An extra match or token above
`max_results` returns a resource error. Regex admission accounts for
`8 × NFA states × (UTF-8 bytes + 1) × search calls × (capture groups + 1)`,
with one call for existence and `max_results + 1` for listing. The final
call checks for an extra match. See [resource limits](../../concepts/resource-limits.md)
for preflight behavior.

## Scalar shape and lexical order

| Request | Rule |
| --- | --- |
| `scalar-weave`, `pad` | Supply scalar `width`, from 0 through 65,536. Defaults are the right side and one ASCII space. Text longer than the requested width fails |
| `scalar-weave`, `interleave` | Supply 2 to 16 ordered lanes. Default `equal_lengths` requires matching lengths. `ragged` visits lanes in order, skipping exhausted lanes |
| `scalar-weave`, `deinterleave` | Use the returned lane lengths. Their sum must equal the exact source scalar count |
| `select-words` | Request signed indexes in order, including duplicates. Default `index_base` is 1 and `-1` selects the final word. `out_of_range: "skip"` records omitted indexes |
| `sort` | Supply `unit: "scalars"` or `unit: "tokens"`. Default order is ascending. Equal items preserve source order in either direction |

The default sort compares exact scalar values lexicographically. The
`ascii_letters` collation compares folded ASCII letters while retaining original
bytes. That profile rejects nonletters and accepts empty tokens. The result
includes the original zero-based indexes. Lexical order is independent of locale.

## Limits

| Operation | Limits |
| --- | --- |
| Case transforms | 1 MiB source, 8 MiB report, 8,388,608 work units, 16 MiB scratch |
| Graphemes | 1 MiB source, 65,536 segment records, 8 MiB report, 4,194,304 scan units |
| Regex | 1 MiB source, 4,096-byte pattern, nesting 64, 64 explicit capture groups, 65,536 records, 8 MiB report |
| Regex engine | 1 MiB NFA, 8 MiB PikeVM cache, 32 MiB scratch, 67,108,864 preflight units |
| Scalar shape | 1 MiB aggregate source or output, 65,536 scalars, 2 MiB report, 1,048,576 visits, 8 MiB scratch |
| Words and lexical order | 1 MiB source, 65,536 records or requested indexes, 8 MiB report, 8,388,608 comparison units, 16 MiB scratch |

The regex compiler limits parser storage, repetition expansion and NFA
construction before allocating its search cache. Token comparisons charge
shared-prefix bytes as they are visited. These reports use direct execution. Hosts apply their own invocation document,
input and report budgets before the operation-specific ceilings. The JavaScript
SDK exposes the installed host limits as `cube.limits.direct`.
