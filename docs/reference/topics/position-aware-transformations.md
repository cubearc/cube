# Position-aware transformations

`text.transform.masked` selects scalar positions and applies existing Atbash,
Caesar, ROT13 or identity independently to that set and its complement. It
preserves character ordering and ASCII case; non-ASCII scalars remain unchanged
by the ciphers. There is no implicit normalization, trimming or letter filtering.
All scalars count, including whitespace, punctuation, NUL and combining marks;
a grapheme can span multiple positions. Try [transforming selected positions](../../guides/positions.md).

- Explicit indices form a sorted unique set. Duplicate positions transform once;
  this does not change `text.extract`, which preserves requested order/duplicates.
- Every Nth selects the Nth, 2Nth, ... scalar with positive N, regardless of base.
  For N=2 the reported labels are 1,3,5 in base 0 or 2,4,6 in base 1.
- Fibonacci uses numeric values 0,1,2,3,5,8,... as position labels, deduplicating
  the repeated 1. Base 0 includes position 0; base 1 omits it. Generation stops
  at the last input position and uses checked arithmetic. Exclusions remove
  Fibonacci **values**, not sequence ordinals: excluding 3 removes label 3,
  not the third sequence term. Only Fibonacci values are valid exclusions;
  valid values beyond the input length have no effect. Exclusion duplicates
  have no additional effect. Exclusion of 13 and 21 sends those positions to
  the `otherwise` transformation; it does not remove characters from the text.

Base 0 is the default; explicit base 1 applies to indices, reported positions
and overrides alike. Out-of-range explicit indices fail rather than being
ignored. Empty input/selection is valid. Each selector accepts only its own
parameters; Caesar requires an explicit signed i32 shift (normalized modulo 26),
and other transformations reject a shift.

Optional manual overrides specify exactly one replacement scalar at an exact
position. They run after deterministic transformation, in caller order, with
repeated positions recorded separately. The report preserves original input,
configuration, selected positions, deterministic text, every edit's previous
and replacement scalar, and final text. An override expresses a caller choice;
it is not cryptographic evidence. No automatic guesses or substitutions occur.

Input is limited to 1 MiB UTF-8. Each explicit index/exclusion/override list and
the selected position set is limited to 65,536 entries, including duplicates in
caller lists. Over-budget selections fail in full, without silent truncation.
One-for-one scalar replacement bounds final output by 4 MiB, within the existing
8 MiB output ceiling. Selector validation is reusable independently of ciphers.

A version-1 text-to-text pipeline adapter calls this same operation. Tagged list
parameters and mandatory per-step audits let it compose without losing explicit
edits; see [composition contracts](compose-operations.md#compose-operations). The engine API and recipe
format remain 1; existing adapters retain their contracts.
