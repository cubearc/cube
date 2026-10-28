# Position-aware transformations

`text.transform.masked` applies a cipher to selected scalar positions and another
to the remaining positions. It preserves order and ASCII case; non-ASCII scalars
are unchanged by the ciphers. See [Transform selected positions](../../guides/positions.md)
for an example.

## Position rules

All scalars count, including whitespace, punctuation, NUL and combining marks.
There is no implicit normalization, trimming or letter filtering; a grapheme can
span several positions.

| Setting | Rule |
| --- | --- |
| Position base | 0 by default; explicit base 1 applies to indices, reports and overrides |
| Empty input or selection | Valid |
| Explicit out-of-range index | Fail |
| Selector parameters | Only parameters belonging to that selector are accepted |

## Selectors

| Selector | Selected positions | Duplicate behavior |
| --- | --- | --- |
| Explicit indices | Sorted unique set of valid indices | Transform each position once |
| Every Nth | Nth, 2Nth, … scalar for positive N, independent of base | Not applicable |
| Fibonacci | Numeric labels 0, 1, 2, 3, 5, 8, … through the last input position | Deduplicate the repeated 1 |

For every Nth with N=2, reported labels are 1,3,5 in base 0 or 2,4,6 in base 1.
Explicit selection differs from `text.extract`, which preserves requested order
and duplicates.

### Fibonacci exclusions

Base 0 includes label 0; base 1 omits it. Generation uses checked arithmetic.

Exclusions remove Fibonacci **values**, not sequence ordinals: excluding 3 removes
label 3. Only Fibonacci values are accepted. Repeated exclusions have no additional
effect; valid values beyond the input length have no effect.

Excluded positions use the `otherwise` transformation. Excluding 13 and 21 does
not delete those characters.

## Transformations and overrides

Choose Atbash, Caesar, ROT13 or identity for each position set. Caesar requires
an explicit signed `i32` shift, normalized modulo 26. Other transformations reject
a shift parameter.

Manual overrides replace exactly one scalar at an exact position. They run after
deterministic transformation, in caller order. Repeated positions are separate
edits; each records the previous and replacement scalar.

The report contains:

- Original input and configuration.
- Selected positions and deterministic text.
- Every manual override.
- Final text.

An override records a caller choice, not cryptographic evidence. No guesses or
automatic substitutions occur.

## Limits

| Resource | Maximum |
| --- | --- |
| UTF-8 input | 1 MiB |
| Each index, exclusion or override list | 65,536 entries, counting duplicates |
| Selected position set | 65,536 entries |
| Final output | At most 4 MiB from one-for-one replacement, within the 8 MiB output ceiling |

Over-budget selections fail in full without truncation. Selector validation is
also available independently of cipher execution.

## Pipeline support

The version-1 text-to-text adapter uses tagged list parameters and mandatory
per-step audits. See [Masked transformation audits](compose-operations.md#masked-transformation-audits)
for retention, output requirements and combined pipeline budgets.
