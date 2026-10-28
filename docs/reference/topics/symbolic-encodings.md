# Symbolic encodings

Use `symbols` for textual International Morse or reversible Unicode Braille cells.
These operations do not detect symbols from audio or images.

```sh
cube encode morse --text 'SOS 2'
# ... --- ... / ..---
cube decode braille --text '⠀⠁⣿'
# 0001ff
```

Morse supports A–Z/0–9; encoding accepts lowercase and canonicalizes ASCII spaces.
Decoding returns uppercase, with one space between tokens and ` / ` between words.
Unsupported punctuation, noncanonical spacing and trailing newlines fail.
Braille represents raw eight-dot masks, not natural-language Braille transcription:
bit `n-1` is dot `n`, and a byte maps to U+2800 plus its mask. Every byte round trips.
Both accept at most 1 MiB input; canonicalized Morse case/spacing does not round-trip.

A1Z26 maps ASCII A–Z to canonical decimal 1–26 tokens joined by hyphens within
each word: `Hi Cube` becomes `8-9 3-21-2-5`. Space and ASCII control whitespace
TAB/LF/VT/FF/CR are preserved exactly, including repeated or boundary whitespace.
Decoding returns uppercase. Punctuation, non-ASCII, signs, leading zeros, empty
hyphen tokens and numbers outside 1–26 fail at their byte offset. Encoding accepts
1 MiB and produces at most 3 MiB; decoding accepts 3 MiB. Uppercase supported
inputs round trip; lowercase case is deliberately lost. Both have bounded text
pipeline adapters, subject to the existing smaller pipeline value/work budgets.

## Semaphore and directional clocks

`symbol.clock.directions` is a separate caller-selected convention, not official
Semaphore or clock recognition. Select H:MM (ASCII hour 0..12, 0=12, minute 0..59;
no seconds/fractional minutes) or two signed i64 millidegree angles, never both.
Hours move 500 millidegrees/minute, minutes 6000. Bearings normalize modulo 360000
before arithmetic, so signed extremes are safe. Zero axis defaults N, positive
angles clockwise; E/S/W and reversed sense are explicit. Eight sector centers are
45000 apart; within +/-1 millidegree of a 22500-offset boundary, return both
directions as ambiguous rather than breaking a tie. Integer arithmetic makes
these boundaries reproducible. Each call is one reading with fixed work/storage
and output below 1 KiB. Optional decoding uses the separate Semaphore decoder with
hour=sender LH and minute=sender RH, rejecting ambiguity/unsupported pairs. This
structured result has no pipeline adapter.

Letter-only flag Semaphore follows the anatomical LH/RH positions in the
[Marine Corps Museum chart](https://www.usmcmuseum.com/uploads/6/0/3/6/60364049/nmmc_semaphore_flag_booklet_final_1.pdf)
and [Scout Association chart](https://prod-cms.scouts.org.uk/media/5508/generic-resources-semaphore-signals.pdf).
The [ANBG text](https://www.cpbr.gov.au/flags/semaphore.html) differs for I (RH up);
both charts show RH high, so I is `SW,NW`, distinct from K `N,SW`.
No artwork is bundled. Coordinates are receiver-facing: sender LH points toward
screen-right when extended outward. `N,NE,E,SE,S,SW,W,NW` denote eight directions.
Tokens are `LH,RH`, separated by exactly one ASCII space; `/` represents each
literal ASCII space. A-Z letters canonicalize case; digits, other whitespace and
procedural/numeric modes are excluded. J is literal J; D encodes canonically as
`S,N` and also decodes `N,S`. Reversed arms are otherwise not inferred.
Empty input is empty output. Limits are 1 MiB transport and 16384 symbols/tokens;
encoding expands to less than 96 KiB. Two version-1 text pipeline adapters preflight
16384 input bytes for encoding, 98304 for decoding and a 6x/1x output bound.
Existing engine/adapter versions retain their behavior. Clock interpretation is
a separate operation, not part of this alphabet.
