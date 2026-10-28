# Symbolic encodings

Encode text as Morse, Braille masks, A1Z26 or Semaphore, or interpret a
caller-supplied clock direction. These operations do not detect symbols in
audio or images.

## Morse, Braille and A1Z26

| Encoding | Mapping | Text and round-trip rules |
| --- | --- | --- |
| International Morse | A–Z and 0–9 | Fold ASCII case; canonicalize ASCII spaces; decode to uppercase |
| Eight-dot Braille | Byte mask to U+2800 + mask | Bit `n-1` is dot `n`; all bytes round-trip; not language transcription |
| A1Z26 | A–Z to decimal 1–26, with hyphens within words | Fold ASCII case; preserve ASCII space/TAB/LF/VT/FF/CR exactly; decode to uppercase |

### Token rules and limits

| Encoding | Grammar | Limits |
| --- | --- | --- |
| Morse | One space between tokens, ` / ` between words; reject unsupported punctuation, noncanonical spacing and trailing newlines | 1 MiB input |
| Braille | Unicode eight-dot cells representing raw masks | 1 MiB input |
| A1Z26 | Canonical decimal tokens; reject punctuation, non-ASCII, signs, leading zeros, empty hyphen tokens and values outside 1–26 | Encode: 1 MiB input, 3 MiB output; decode: 3 MiB input |

A1Z26 errors include the byte offset. Uppercase supported inputs round-trip;
lowercase case is lost. Its text pipeline adapters use the smaller pipeline
value and work budgets.

For example, encode a Morse message or recover bytes from Braille cells:

```sh
cube encode morse --text 'SOS 2'
# ... --- ... / ..---
cube decode braille --text '⠀⠁⣿'
# 0001ff
```

A1Z26 encodes `Hi Cube` as `8-9 3-21-2-5`.

## Semaphore and directional clocks

Semaphore encodes letters as flag positions. Clock directions use a separate,
explicit convention.

### Semaphore alphabet

Directions are `N,NE,E,SE,S,SW,W,NW`. A token is `LH,RH` using the sender's
anatomical left and right arms, viewed by the receiver. The sender's left arm
points toward screen-right when extended outward.

| Rule | Behavior |
| --- | --- |
| Letters | A–Z; ASCII case canonicalized; J means literal J |
| Spaces | Each literal ASCII space becomes `/` |
| Token separator | Exactly one ASCII space |
| D | Encodes as `S,N`; also accepts `N,S` when decoding |
| I / K | `SW,NW` / `N,SW` |
| Other reversed arms | Not inferred |
| Unsupported input | Digits, other whitespace, procedural and numeric modes |
| Empty input | Empty output |

The alphabet follows the [Marine Corps Museum chart](https://www.usmcmuseum.com/uploads/6/0/3/6/60364049/nmmc_semaphore_flag_booklet_final_1.pdf)
and [Scout Association chart](https://prod-cms.scouts.org.uk/media/5508/generic-resources-semaphore-signals.pdf).
Both place I's right arm high; the [ANBG text](https://www.cpbr.gov.au/flags/semaphore.html)
differs. No chart artwork is bundled.

### Semaphore limits

| Resource | Maximum |
| --- | --- |
| Transport | 1 MiB |
| Symbols / tokens | 16,384 |
| Encoded output | Less than 96 KiB |
| Version-1 adapter encoding input / output expansion | 16,384 bytes / 6× |
| Version-1 adapter decoding input / output expansion | 98,304 bytes / 1× |

### Clock directions

`symbol.clock.directions` accepts either `H:MM` or two signed `i64` millidegree
angles. It does not recognize clock images or define official Semaphore.

| Setting or rule | Behavior |
| --- | --- |
| Time | ASCII hour 0–12, with 0=12; minute 0–59; no seconds or fractions |
| Hour movement | 500 millidegrees per minute |
| Minute movement | 6,000 millidegrees per minute |
| Angle normalization | Modulo 360,000 before arithmetic, including signed extremes |
| Zero axis / positive direction | North / clockwise; other cardinal axes and reversed sense are explicit |
| Eight sector centers | 45,000 millidegrees apart |
| Boundary ambiguity | Within ±1 millidegree of a 22,500-offset boundary, return both directions |

Each call handles one reading with fixed work/storage and output below 1 KiB.
Optional Semaphore decoding maps hour to LH and minute to RH; ambiguity and
unsupported pairs fail. This structured result has no pipeline adapter.
