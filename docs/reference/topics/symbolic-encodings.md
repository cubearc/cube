# Symbolic encodings

Encode text as Morse, Braille masks, A1Z26 or Semaphore, or interpret a
caller-supplied clock direction. A finite Solresol profile maps exact French
gloss tokens and relative syllables. These operations do not detect symbols in
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

## Solresol monosyllables

Select `profile="gajewski_1902_monosyllables_v1"` explicitly. The profile contains
the complete seven-entry monosyllable table from PDF page 4 of
[Gajewski's grammar facsimile](https://upload.wikimedia.org/wikipedia/commons/b/b9/Grammaire_du_Solresol.pdf),
independently transcribed from the original French. The
[source record](https://fr.wikisource.org/wiki/Fichier:Grammaire_du_Solresol.pdf)
identifies the 1902 original and its public-domain status; the index identifies
the facsimile's 1997 republication. No scan, font, score or modern English
translation is bundled.

| Canonical word | Accepted abbreviation | All French gloss tokens |
| --- | --- | --- |
| `do` | `d` | `non`, `pas`, `point`, `ne`, `ni` |
| `ré` | `r` | `et`, `ainsi que` |
| `mi` | `m` | `ou`, `ou bien` |
| `fa` | `f` | `à`, `au` |
| `sol` | `so` | `si (condition)` |
| `la` | `l` | `le` |
| `si` | `s` | `oui`, `soit`, `volontiers`, `d'accord` |

Encode exact presegmented French `terms` with `symbol.solresol.encode`. Each term
selects the canonical full word; synonyms share it. Decode `tokens` with
`symbol.solresol.decode` to retain every source-listed gloss in `readings`, with
its zero-based `token_index`. The report also retains the original forms and
abbreviations in `tokens`. It never guesses which synonym was intended.
Internal spaces in a gloss, such as `ainsi que`, belong to one array entry.

Save `{"profile":"gajewski_1902_monosyllables_v1"}` as `solresol.json`:

```sh
cube encode solresol --parameters solresol.json --text '["non","oui","à"]'
cube decode solresol --parameters solresol.json --text '["d","si","ré"]'
```

The first report's tokens are `["do","si","fa"]`. The second preserves the
abbreviation `d` and all five negative glosses, all four affirmative glosses,
and both glosses for `ré`. The conditional gloss `si (condition)` encodes to
`sol`; affirmative `oui` encodes to `si`.

Spelling, accents and word boundaries are exact. Full forms are lowercase;
`ré` uses NFC spelling, while `re` and decomposed `re\u0301` reject. Repeated array
entries remain separate words. Joined or inflected forms and extra grammatical
marks reject rather than being stripped. This finite profile does not implement
other Solresol vocabulary, grammar, absolute pitch, notation or transcription.

`provenance` contains source/profile identifiers, the source PDF's SHA-256 and
the exact effective definition SHA-256. Encoding and decoding use the same
definition identity. Input permits at most 65,536 tokens and 1 MiB compact JSON,
with 256 UTF-8 bytes per token; the fixed definition is below 64 KiB. Complete
reports are capped at 1 MiB without truncation. Large synonym expansions can
reach that report cap before the token cap. Empty input returns empty tokens
and readings with provenance.

Native Python exposes `solresol_encode(terms, parameters)` and
`solresol_decode(tokens, parameters)`. Rust uses `symbols::solresol::{encode,decode}`;
JavaScript/TypeScript executes the canonical IDs with named `terms` or `tokens`
input and the same required profile. Native Play uses structured word arrays
and inspectable readings/provenance. For other caller-owned lexicons, use the
[explicit symbolic token table](../../ciphers.md#symbol-token-tables) with its own source and ambiguity rules.

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

## Dated symbolic alphabets

Use `symbol.alphabet.encode` with exact presegmented `units`, or
`symbol.alphabet.decode` with exact `tokens`. A required `profile` selects one
published table; display names alone are not accepted.

| Profile | Units and representation | Source |
| --- | --- | --- |
| `aurebesh_ucsur_2016_v1` | A–Z at U+E890–U+E8A9; CH, AE, EO, KH, NG, OO, SH, TH at U+E8AA–U+E8B1; published punctuation at U+E8C0–U+E8CE | [Doug Ewell's proposal, 2016-06-22](https://www.kreativekorp.com/ucsur/charts/aurebesh.htm) |
| `standard_galactic_ucsur_2019_v1` | A–Z at U+EB40–U+EB59; left/right quotation marks and full stop at U+EB5D–U+EB5F | [Rebecca Bettencourt's proposal, 2019-09-14](https://www.kreativekorp.com/ucsur/charts/sga.html) |

These are **private-use proposals**, not Unicode-standard encodings of the
scripts. Viewing their glyphs requires a font implementing the selected proposal;
Cube bundles no fonts or source artwork. Ordinary U+0020 spaces pass through as
explicit units. Neither profile defines digits; unsupported units, reserved
positions, lowercase and joined words reject. Case folding, font detection, OCR
and language translation are not inferred.

Aurebesh punctuation uses the literal units `, . ? ! : ; - “ ” ‘ ’ ( ) /` and
the semantic unit `CREDITS_SIGN`, in proposal order. Standard Galactic punctuation
uses the literal units `“ ” .`. ASCII straight quotes are not aliases for the
proposal's left/right marks.

Save `{"profile":"aurebesh_ucsur_2016_v1"}` as `alphabet.json`:

```sh
cube encode alphabet --parameters alphabet.json --text '["A","CH","C","H"]'
```

The JSON report's `tokens` are U+E890, U+E8AA, U+E892, U+E897, in that order.
Supplying `CH` as one unit selects the digraph; separate `C` and `H` select two
letters. Decode returns the original tokens and `readings`, each with its
zero-based `token_index` and every listed `alternative`. Repeated symbols remain
separate. Empty input returns empty arrays. Both reports include the dated
`profile_id`, source URL, SHA-256 of the reviewed source snapshot (HTML, PDF or chart bytes) and SHA-256 of the
canonical effective definition. Source hashes identify the reviewed snapshots;
they do not claim a font's identity or licensing terms.

### Interfaces and limits

Native Python exposes `alphabet_encode(units, parameters)` and
`alphabet_decode(tokens, parameters)`. Rust uses
`symbols::alphabet::{encode,decode}`. The JavaScript/TypeScript SDK executes the
same operation IDs through Core Wasm. Play supports the typed JSON ports and
named results; its JSON viewer preserves the arrays and provenance.

Input is limited to 65,536 items and 1 MiB compact JSON, with at most 256 UTF-8
bytes per unit/token. Each fixed definition is at most 64 KiB; the complete
serialized report is at most 1 MiB. Either direction can reach its serialized-token/report limit before the
item limit. Exceeding either fails with `resource_limit`, without truncating the
message. SDK transport structure and worker limits can reject an invocation
before it reaches these Core ceilings. Unknown units/tokens fail with `invalid_input`; a missing, unknown or
extra profile parameter fails with `invalid_parameter`. These puzzle mappings
provide no confidentiality or authentication.

### Geometric profile selection

These profiles use exact textual transcriptions of shapes or clock positions.
They perform no graphical recognition. Choose the table that matches the clue;
profiles with similar names can assign different letters to the same shape.

| Profile | Exact source and supported repertoire |
| --- | --- |
| `pigpen_haverhill_pairs_v1` | [Haverhill Family History Group, page1](https://haverhill-tc.gov.uk/wp-content/uploads/Pigpen-Code.pdf): A–Z in paired cells; called Pigpen/Freemason/Rosicrucian by this source |
| `tic_tac_toe_dcode_v1` | [dCode single-grid description](https://www.dcode.fr/tic-tac-toe-cipher): A–I plain, J–R cross, S–Z circle; last circled cell selected for space |
| `templars_bothwell_gosse_v1` | [Bothwell-Gosse/Dickinson, pp89–90](https://freemasonry.bcy.ca/texts/templars_cipher.html): 25 letters, no J, plus explicit space gaps |
| `betamaze_cliff_2006_v1` | [Terrana Cliff's chart in the Conlangs DE-Cal reader](https://library.conlang.org/education/DECAL_%20reader_part2.pdf), printed p289/PDF p111: lowercase a–z and the space shape |
| `clock_quarter_hour_zero_v1` | [dCode's quarter-hour example](https://www.dcode.fr/clock-cipher): A=`00:00`, B=`00:15`, …, Z=`06:15`; no space or punctuation token |
| `birds_nrtech_four_features_v1` | [NR Tech Studio's four-feature reference](https://nrtechstudio.com/tools/birds-on-a-wire-cipher/): A–Z and explicit space gaps; its authored variant, without an assertion that other bird drawings match |

The dCode descriptions are attributed under the publisher's stated CC-BY terms;
their software is not reused. Glyph correspondences and geometric rules are
independently transcribed. The source snapshots, rather than source fonts or
artwork, identify the exact reviewed variants.

### Grid walls and markers

For Pigpen and Tic-Tac-Toe, `grid:` is followed by the cardinal walls surrounding
the cell, in **N,E,S,W order**, omitting absent walls. The top-left cell is
`grid:ES`; the center is `grid:NESW`. No outer border is added to either grid.

Haverhill's nine cells contain `AB CD EF / GH IJ KL / MN OP QR`. The first
letter has no marker; the second adds `:dot`. The X sectors contain `ST` north,
`UV` west, `WX` east and `YZ` south. Use `wedge:north`, `wedge:west`,
`wedge:east`, `wedge:south`, again adding `:dot` for the second letter.
A wedge names the sector occupied by the letter, not the direction of its apex.
Thus `A` encodes as `grid:ES`, `B` as `grid:ES:dot`, and `S` as `wedge:north`.
An ordinary space unit/token represents the source example's word gap.

Tic-Tac-Toe fills the nine cells row by row three times. The first bank has no
marker, the second adds `:cross`, and the third adds `:circle`. Consequently,
`J` is `grid:ES:cross`, while `S` is `grid:ES:circle`. Space uses the remaining
bottom-right circled cell, `grid:NW:circle`. Dots and X-grid wedges reject in
this profile. Marker spellings and wall order are exact; no mirrored or rotated
variant is inferred.

### Templars shape tokens

Open-angle tokens name their **apex**: A, B, C, D use `angle:south`,
`angle:west`, `angle:north`, `angle:east`. O, P, Q, R add `:dot` respectively.
Closed triangles E, F, G, H use `triangle:east`, `triangle:west`,
`triangle:north`, `triangle:south`; their dotted counterparts are V, T, U, S
in that source order.

The lozenges refer to the arms of **diagram III** in the cited source:
I, K, L, M use `lozenge:north-arm`, `lozenge:east-arm`,
`lozenge:south-arm`, `lozenge:west-arm`. The dotted forms read X, Y, W, Z.
N is `saltire`. These arm labels avoid inventing a generic diamond rotation
convention. J rejects; Cube does not silently merge it with I. Ordinary spaces
are explicit gaps. The profile identifies the published table, without claiming
its supposed medieval attribution is established.

### Betamaze cells and reading order

A token contains three rows separated by `/`, each with three cells. Use `.` for
a white cell, `#` for a full black cell, and A/B/C/D for black triangles at the
NW/NE/SW/SE corner of a cell. These uppercase cell labels are geometry notation;
the plaintext units are lowercase.

For example, `a` is `#CB/..C/C.#`, `m` is `ADB/B.C/CAD`, `z` is
`D.C/.../B.A`, and space is `A.B/.../C.D`. Encoding selects the orientation
printed in the cited chart. Decoding accepts all source-permitted rotations and
reflections: 148 distinct tokens across the 27 shapes. It retains each token's
received orientation in the report. Other shapes, changed padding and uppercase
plaintext reject.

Supply tokens in the intended reading order. Cube does not find the entrance,
choose a branch, solve a maze or infer word order from adjoining tiles.

### Clock times and bird features

The clock profile advances one quarter-hour per uppercase letter, starting at
A=`00:00`. Times require two hour digits, a colon and two minute digits. This is
one explicit published example, not a universal clock alphabet. `12:00`,
`00:0`, times after `06:15`, unsupported minutes and spaces reject. No timezone,
clock-hand direction or brute-force key selection is involved.

Bird tokens contain four colon-separated features in this exact order:
`body:head:tail:wing`. Body is `back`, `upright` or `forward`; head is `left`,
`front` or `right`; tail is `up`, `down` or `flat`; wing is `folded`, `raised` or
`spread`. Only the source's 26 combinations are accepted. For example,
A=`back:left:up:folded`, H=`upright:left:flat:raised`, and
L=`forward:left:up:raised`. Encoding emits features; decoding also accepts the
publisher's `B01`–`B26` aliases and preserves those received codes. An ordinary
space is a gap. Unknown codes, other feature orders and punctuation reject.
The website's uppercasing and punctuation deletion are not applied to these
exact, presegmented inputs.

### Historical source profiles

Historical profiles transcribe the exact selected repertoire. Their neutral glyph
IDs refer to the cited source chart or illustration; they are not Unicode script
encodings and do not render the source artwork. Match a glyph manually, then
supply its exact ID. All array positions and explicit alternatives survive
conversion. No normalization, transliteration, language translation or reading
order is inferred.

| Profile | Supported source repertoire |
| --- | --- |
| `theban_trithemius_1518_v1` | [Trithemius's 1518 plate](https://esotericarchives.com/tritheim/polygraphia/poly579.jpg): 23 Roman labels and the omega mark |
| `celestial_agrippa_ch30_v1` | [Agrippa, Book III, chapter 30, Celestial plate](https://esotericarchives.com/agrippa/celestis.gif): 22 Hebrew base letters |
| `malachim_agrippa_ch30_v1` | [Malachim plate in the same chapter](https://esotericarchives.com/agrippa/malachim.gif): 22 Hebrew base letters, including two Samech forms |
| `magi_ambrosius_1539_v1` | [Peterson's Magi chart](https://esotericarchives.com/fonts/magi.jpg), attributed to Theseus Ambrosius (1539): 22 Hebrew base letters |
| `enochian_peterson_2022_v1` | [Peterson's Enochian-3188 chart](https://esotericarchives.com/fonts/enochian-3188.jpg), associated with font version 1.00, 2022-08-27: actual nonblank letter cells in both chart styles |
| `moon_english_1877_v1` | [Moon's 1877 alphabet, Figure 1/PDF p.4 of N4128R](https://www.unicode.org/wg2/docs/n4128.pdf): uncontracted English A–Z, ampersand and explicit gaps |
| `dancing_men_doyle_1905_v1` | [Original Dancing Men illustrations](https://www.gutenberg.org/files/108/108-h/108-h.htm#chap03): 18 attested poses and their word-end flags |
| `alphabetum_kaldeorum_hahn_2004_v1` | [Heiko Hahn's 2004 chart](https://commons.wikimedia.org/wiki/File:Alphabetum_Kaldeorum.jpg), after Cod.4°810, fol.41v: 23 labelled groups, 30 forms and explicit U/V alternatives |

Except for Dancing Men, IDs use a script prefix and **one-based source row and
column**, for example `celestial:r1c9`. Count glyphs left to right within each
row of the cited chart; do not count headings or caption lines. These are legend
coordinates, not the physical positions or reading order of a message. IDs are
exact: changed prefixes, padding or leading zeros reject.

#### Theban labels and omega

The source's three rows, written as canonical units, are:

```text
r1: A B C D E F G H
r2: I K L M N O P Q
r3: R S T V X Y Z OMEGA
```

Use `theban:r1c1` for A and `theban:r3c8` for the omega-labelled mark.
`OMEGA` identifies that source mark; Cube does not reinterpret it as W or
insert sentence punctuation. J, U, W, lowercase and spaces are absent in this
profile. There is no implicit I/J, U/V or doubled-V modernization.

#### Hebrew units and the three source charts

Celestial, Malachim and Magi use the following actual Hebrew base letters as
units. They do not map a guessed modern Latin alphabet.

| Source name | Unit | Source name | Unit |
| --- | --- | --- | --- |
| Aleph | א | Lamed | ל |
| Beth | ב | Mem | מ |
| Gimel | ג | Nun | נ |
| Daleth | ד | Samech | ס |
| He | ה | Ain | ע |
| Vau | ו | Pe | פ |
| Zain | ז | Zade | צ |
| Cheth | ח | Kuff | ק |
| Theth | ט | Res | ר |
| Iod | י | Shin | ש |
| Caph | כ | Tau | ת |

The row lists below give the source names from **left to right**, so mixed-script
text direction cannot reverse the coordinate convention:

```text
Celestial
r1: Theth Cheth Zain Vau He Daleth Gimel Beth Aleph
r2: Zade Pe Ain Samech Nun Mem Lamed Caph Iod
r3: Tau Shin Res Kuff

Malachim
r1: Zain Vau He Daleth Gimel Beth Aleph
r2: Nun Mem Lamed Caph Iod Theth Cheth
r3: Res Kuff Zade Pe Ain Samech Samech Shin Tau

Magi
r1: Lamed Caph Iod Theth Cheth Zain Vau He Daleth Gimel Beth Aleph
r2: Tau Shin Res Kuff Zade Pe Ain Samech Nun Mem
```

Malachim encodes Samech with `malachim:r3c6`. Both that form and
`malachim:r3c7` decode to the same unit, retaining the received token. All other
listed glyphs are distinct. Final forms ך/ם/ן/ף/ץ, vowel marks, Latin
transliteration, spaces and punctuation reject. Input arrays retain their given
order; Cube does not reverse them because the units are Hebrew.

#### Enochian chart styles and alternative readings

The first three chart rows are the formal glyph style and rows 4–6 are the
handwritten style. Each row has ten labelled cells. They follow A–J, K–T and
U–Z; cells K/W and the displayed punctuation cells are blank and reject. Encoding
uses the first formal source form; decoding also accepts its handwritten alias.

I/J share GON: `enochian:r1c9`, `enochian:r1c10`, `enochian:r4c9` and
`enochian:r4c10` all return alternatives `["I","J"]`. U/V share the chart's
VAN form: columns 1/2 of rows 3/6 return `["U","V"]`. Y is the explicitly
marked GON+ extension at `enochian:r3c5` / `enochian:r6c5`.

This profile preserves the modern chart's actual distinctions and gaps. It does
not claim that Dee's original script had 26 letters, infer missing K/W, recover
a unique I/J or U/V spelling, translate the Enochian language, or infer page
direction. Plaintext units are exact uppercase labels even for the handwritten
style. Spaces are not defined by this letter-table profile.

#### Moon alphabet and supplied reading order

The four rows of Figure1 are `A B C D E F G`, `H I J K L M N`,
`O P Q R S T U`, and `V W X Y Z &`. Thus A is `moon:r1c1`, V is
`moon:r4c1`, and ampersand is `moon:r4c6`. An ordinary space unit/token is an
explicit gap.

Supply the glyphs in reading order. The profile does not expand contractions,
choose a numeral system, interpret tactile page layout or reverse alternate
lines. It does not mirror glyphs. The proposal's suggested Unicode positions
are not used as actual Unicode characters by Cube.

#### Dancing Men poses and flags

The following neutral IDs identify the body pose in the named original
illustration, ignoring its optional flag. `f01` means Gutenberg's `img01.jpg`;
`p01` means the first figure within that illustration.

| Unit | Pose ID | Unit | Pose ID |
| --- | --- | --- | --- |
| A | `doyle:f01p01` | M | `doyle:f01p02` |
| H | `doyle:f01p03` | E | `doyle:f01p04` |
| R | `doyle:f01p05` | B | `doyle:f01p08` |
| S | `doyle:f01p10` | L | `doyle:f01p11` |
| N | `doyle:f01p13` | Y | `doyle:f01p15` |
| T | `doyle:f02p02` | I | `doyle:f02p06` |
| G | `doyle:f02p07` | C | `doyle:f03p01` |
| O | `doyle:f03p02` | V | `doyle:f04p03` |
| P | `doyle:f05p06` | D | `doyle:f05p24` |

Append `:flag` for the same pose carrying a word-end flag. Its exact plaintext
unit includes **one trailing U+0020 space**: `"M "` encodes as
`doyle:f01p02:flag`, while `"M"` encodes the unflagged pose. A flag remains part
of that one token/reading; Cube neither inserts nor strips it automatically.
For example, encode `CODE` with an explicit word end using units
`["C","O","D","E "]`.

F, J, K, Q, U, W, X and Z have no attested pose in this profile and reject.
P and V remain distinct: the source's V has a slightly sloping foot. Do not
collapse that detail while transcribing. The profile does not repair historical
B/R errors by guessing a word, or add later completed alphabets. Its source hash
covers the Gutenberg HTML/image archive; all six relevant original illustration
byte streams were checked against that archive.

#### Alphabetum Kaldeorum forms and merged U/V

Use `kaldeorum:rRcCvV`, where R/C are source row/column numbers and V is the
form's one-based position from left to right inside its cell. For example,
`kaldeorum:r1c7v2` is G's second form. The source rows are:

```text
r1: A B C D E F G H
r2: I K L M N O P Q
r3: R S T U/V X Y Z
```

G has two forms, H three, and L/O/P/Q two each; all other cells have one form.
Encoding chooses form 1. Decoding any listed form retains that exact ID.
The U/V cell always returns both alternatives; it does not guess a unique
spelling. J/W, implicit I/J modernization, W-as-VV expansion and unspecified
null glyphs are not part of this chart profile.

The correspondence-data component adapted from Hahn's chart is distributed
with its [CC-BY-SA 3.0 attribution and share-alike terms](https://creativecommons.org/licenses/by-sa/3.0/).
No chart artwork is included; the independent software keeps its stated license.
