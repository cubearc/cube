# Phonetic text profiles

Generate reproducible spelling keys from explicitly selected rules. Keys represent
a rule-based pronunciation approximation suitable for comparing candidate
spellings. Multiple unrelated names can share the same key.

Use `cube text profile phonetic --text '<request>'` for one name or an ordered
batch. A request supplies `name` or `names` and a `method`. The `keys` list
preserves the input order.

## Soundex, NYSIIS and Metaphone

| Method | Alphabet and result |
| --- | --- |
| `soundex_nara` | ASCII letters. Four-character census key with H/W bridging and vowel separation |
| `nysiis` | ASCII letters. Default six-character maximum. Set `strict: false` for the full key |
| `metaphone` | ASCII letters. One key, capped by `max_code_length` from 1 through 32, default 4 |
| `double_metaphone` | ASCII letters. Primary and alternate keys, including an identical alternate. Same length setting |

Empty names return empty keys. The census profile follows
[NARA](https://www.archives.gov/research/census/soundex). NYSIIS and the two
Metaphone variants use the Apache Commons Codec 1.15 rules.

```sh
cube text profile phonetic --text '{"method":"soundex_nara","name":"Washington"}'
```

The result contains `keys: [{"key":"W252"}]`.

## Caverphone, Cologne and Match Rating

`caverphone1` and `caverphone2` accept ASCII letters and return six or ten
characters respectively, including padding for an empty name. `cologne` adds
`ÄÖÜäöüß` to its accepted alphabet and returns a decimal key. It suppresses
adjacent equal codes and retains a vowel code at the start.

The `match-rating` operation accepts ASCII-letter names. Its key keeps the
first letter, removes subsequent vowels and collapses consonant runs. Keys
longer than six letters retain the first three and final three. The comparison
reports both keys, the larger unmatched count, the rating `6 - unmatched_count`,
the threshold and the decision. Empty keys and a length difference of at least
three return a nonmatch with null comparison metrics. Identical nonempty names,
including one-letter names, match.

| Total encoded length | Minimum rating |
| --- | --- |
| 1–4 | 5 |
| 5–7 | 4 |
| 8–11 | 3 |
| 12 | 2 |

## Branching alternatives

`daitch-mokotoff` uses the branching extension in Apache Commons Codec 1.15.
The result is a lexical list of unique six-digit codes. Empty input yields
`["000000"]`. The alphabet includes ASCII letters and the scalars mapped by the
pinned rule table. Set `folding: true` to use that table's explicit accent
folding. The default is false.

`beider-morse` selects a `name_type` (`generic`, `ashkenazi` or `sephardic`),
a `rule_type` (`exact` or `approximate`) and a `language` object. Use
`{"mode":"guess"}` for the pinned language hints,
`{"mode":"any"}` for unrestricted rules, or
`{"mode":"explicit","languages":["french"]}` to constrain all branches to a
declared language set. A singleton uses that language's table. A set of
candidate languages uses the `any` table while filtering the rule alternatives.
Language availability depends on the name type.

Default `concatenate: true` encodes the processed name together. Set it to false
for word-by-word groups. The result carries the display encoding, lexical
alternatives, table selection, group paths, a rule-corpus hash, and work/peak
counts. Generic prefixes produce separate and joined groups, using the profile's fixed
prefix order. For `de la`, it checks `de` first. Spaces, hyphens and apostrophes have explicit rules. Unmapped
scalars fail. See the [source and bounded integration record](../../contributing/phonetic-provenance.md).

## Limits and errors

| Profiles | Bounds |
| --- | --- |
| Soundex, NYSIIS and Metaphone | 256 ASCII bytes per name, 256 names and 64 KiB total per batch |
| Caverphone and Cologne | 256 scalars and 1,024 bytes per name, same batch cap |
| Match Rating | 256 ASCII bytes per operand, 1 MiB scratch and 4 KiB report |
| Daitch–Mokotoff | 256 scalars and 1,024 bytes, 4,096 alternatives, 65,536 rule-state updates, 8 MiB scratch |
| Beider–Morse | 256 scalars and 1,024 bytes, 4,096 alternatives, 1,048,576 rule-state units, 32 MiB rule data and scratch |

The batched operation also enforces its aggregate work budget and 1 MiB
report limit. Each branching profile caps the report at 1 MiB and fails on
resource exhaustion before returning any alternatives. These operations use
direct execution.
