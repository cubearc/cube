# Keyed skip and Josephus traversal

Visit positions in a circular sequence of Unicode scalars.
Input stays in stored order, without normalization.

## Traversal behavior

Both methods use a nonempty cyclic list of positive `u64` step sizes.
The first selection advances `step − 1` positions from the starting scalar.
After selection, the anchor advances to the next scalar in the chosen direction.

| Method | After a selection | Count | `ABCDE` with step 2 |
| --- | --- | --- | --- |
| Keyed skip | Keeps the selected scalar in the ring | Required | Count 5 gives `BDACE` |
| Josephus | Removes the selected scalar | Defaults to all scalars | `BDAEC` |

The default start is the first scalar, with origin 0 and forward direction.
Origin 1 and reverse direction must be selected explicitly.

Keyed skip can disable wrapping; it fails if the next selection leaves the input.
Josephus requires wrapping. An explicit Josephus count can stop early, with
unselected scalars returned in `remaining`.

## Result positions

Records retain original zero-based scalar indexes and UTF-8 byte offsets,
regardless of the input origin.

## Limits

Limits apply independently:

| Resource | Maximum |
| --- | --- |
| Input | 1 MiB |
| Step-list entries and selection count | 65,536 |
| Scratch memory, excluding caller input | 8 MiB |
| Serialized report | 8 MiB |
| Work | 16,777,216 units |

Source arrays use eight bytes per scalar; Josephus also uses four bytes per
Fenwick-tree cell. A large ASCII input can therefore fail below the byte limit.
Extraction releases these arrays before reusing the text-extraction routine.

Work includes input validation, indexing, tree setup and visits, selections,
extraction and a conservative serialization estimate. Interface options can lower
the limits. Exhausting a budget fails without a partial result.

These operations have no pipeline adapters.
See [Read text in a different order](../../guides/traversal.md).
