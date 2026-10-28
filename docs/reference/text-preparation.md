# Shared text preparation profile

`text.statistics`, `text.multiset.compare`, `text.multiset.subtract` and
`text.anagram.search` apply the same preparation steps before counting.
See [Unicode and text preparation](../concepts/text.md) for the underlying units.

## Order and defaults

| Step | Default |
| --- | --- |
| Normalization | `none` |
| Case handling | `preserve` |
| Filtering | `all` |

Normalization accepts `none`, `NFC`, `NFD`, `NFKC` or `NFKD`. Case handling accepts
`preserve` or `ascii_lower`. Filtering accepts `all` or `ascii_letters`.
These steps run in the order shown. ASCII handling does not fold accented letters,
segment graphemes or consult a dictionary. Reports retain the normalization data
version.

For example, NFD turns `é` into `e` plus a combining accent; `ascii_letters` then
keeps only `e`. With no normalization, that filter removes the single scalar `é`.

## Preparation limits

These ceilings apply to each prepared input; individual operations can impose
additional or smaller limits.

| Stage | Maximum |
| --- | --- |
| Original UTF-8 text | 1 MiB |
| Normalized UTF-8 text | 8 MiB |
| Retained prepared text | 1,048,576 scalars |
| Distinct scalars in a counting inventory | 10,000 |

Per-word, aggregate dictionary, result and work limits belong to the operation
reference. Filtering still visits discarded scalars; the owning operation defines
how those visits consume its work budget.
