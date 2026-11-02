# Clock and shifting disks

Encipher prepared historical text while retaining the alignment needed to continue
another block. Wheatstone, Kryha and Alberti each use a precisely named source
profile. Supply its settings explicitly; the operations never invent a key,
prepare a message or choose a machine variant.

| Device | Operation pair | Profile |
| --- | --- | --- |
| Wheatstone clock cryptograph | `cipher.wheatstone.encode`, `cipher.wheatstone.decode` | `wheatstone_1879_v1` |
| Kryha, Callimahos's 17-sector example | `cipher.kryha.encode`, `cipher.kryha.decode` | `kryha_callimahos_1973_v1` |
| Alberti, Di Palma museum reconstruction | `cipher.alberti.encode`, `cipher.alberti.decode` | `alberti_di_palma_2009_v1` |

These are historical ciphers and provide no modern confidentiality or
authentication. For independent block rows, use [disks and strips](disks-and-strips.md).

## Inputs and state

Named-port calls take `text` and a typed JSON `settings` input, with no operation
parameters. Native Python functions take text and the same settings dictionary;
the CLI reads settings from `--parameters`. In Play, connect settings as a
structured input Asset. Settings, text and state traces are private; retaining
a result requires explicit consent. Play also applies its own smaller invocation
and storage budgets. These operations have no linear recipe adapter.

Every result contains `text` and `trace`, an empty array unless settings include
`"trace":true`. Wheatstone and Kryha return `final_state`; Alberti returns
`final_index`. To continue processing, use that value as the next call's
`initial_state` or `initial_index`. To decode the entire ciphertext, start from
the sender's original state. Empty text returns the unchanged starting state.
Trace rows contain the zero-based ASCII byte `offset` and the state **after**
processing that symbol.

## Wheatstone's original clock

Save these settings as `wheatstone.json`:

```json
{"profile":"wheatstone_1879_v1","cipher_alphabet":"FBKSYRDLTZAGMUNHOVCIPWEJQX","initial_state":{"plain_position":0,"cipher_position":0}}
```

Run `cube cipher wheatstone encode --parameters wheatstone.json --text THE+QUARTER+`.
The result has text `PZLSPQREQAJD` and final pointers
`{"plain_position":0,"cipher_position":6}`. Decode with the original settings
to recover `THE+QUARTER+`.

The outer ring is fixed `+ABCDEFGHIJKLMNOPQRSTUVWXYZ`, numbered 0–26. The cipher
ring is a required permutation of ASCII A–Z, numbered 0–25. Both initial pointer
positions are required. Plaintext uses only A–Z and `+` for word separation;
ciphertext uses only A–Z.

Move the plain pointer clockwise by 1–26 positions to the next prepared symbol,
and advance the cipher pointer by the same number. An equal consecutive plain
symbol, including the starting pointer, is rejected: an entire 27-position turn
would be indistinguishable from a one-position turn on the cipher ring. Prepare
repeated letters externally according to the intended historical procedure.
On decode, an equal cipher symbol represents 26 advances. No filler, case folding,
spacing or sentence reset is inferred.

The rules and known answer come from Wheatstone's 1879 directions, reproduced in
Friedman's [Several Machine Ciphers (1918), printed pages 7–10](https://incoherency.co.uk/interest/several-machine-ciphers.pdf).
The [surviving mechanism photographs](https://www.jproc.ca/crypto/wheatstone.html)
show its 27/26 gearing.

## Kryha's 17-sector profile

Required settings are `profile`, `plain_alphabet`, `cipher_alphabet` and
`initial_state` with `rotation` 0–25 and `selector` 1–17. Both alphabets must
contain every ASCII letter A–Z exactly once. Input and output use those exact
uppercase letters. `rotation` is the cipher-ring displacement from the plain
ring; selector identifies the sector used for the next post-letter advance.

Each letter uses the current alignment, then advances the cipher ring clockwise
by the selected effective displacement:

```text
7 6 7 5 6 7 6 8 6 10 5 6 5 7 6 5 9
```

The selector wraps after 17. This schedule advances 111 positions per revolution;
the complete state repeats after 442 letters. With plain alphabet
`ABCDEFGHIJKLMNOPQRSTUVWXYZ`, cipher alphabet `HYDRAULICBEFGJKMNOPQSTVWXZ`,
rotation 0 and selector 1, seventeen `A` characters encipher as
`HIJSZUGPHLNTYLJQX`, ending at rotation 7 and selector 1.

This exact profile follows Callimahos's primary [Q.E.D.—2 Hours, 41 Minutes
(1973), pages 20–21](https://www.nsa.gov/portals/75/documents/news-features/declassified-documents/tech-journals/qed.pdf).
It does not represent every Kryha model. The later 52-stop mechanism and other
gear reconstructions have different configurations; the explicit profile name
prevents silently substituting one for another.

## Alberti's digit feedback

Required settings are `profile` and `initial_index`: exactly one inner-ring
symbol placed under the fixed outer `A`.

| Ring | Symbols in clockwise order |
| --- | --- |
| Fixed outer | `ABCDEFGILMNOPQRSTVXZ1234` |
| Moving inner | `c&bmdgpfznxyutoskerlhaiq` |

Plaintext must use exact outer-ring symbols; ciphertext must use exact inner-ring
symbols. The shared historical U/V cell is represented by uppercase `V` and
lowercase `u`. Automatic U/V folding, case conversion, random indicator insertion,
word spacing and phrase-book expansion are separate preparation steps.

After enciphering or recovering an outer digit 1–4, realign its ciphertext symbol
under fixed `A`. Other symbols preserve the alignment. Decoding retains control
digits so that the exact prepared message and alignment history remain reversible.

With `initial_index` equal to `c`, `ALE4ISA1BEAVTI3FVL2CITI` enciphers as
`czdqpoqlhqluybeix&okapa` and ends at index `o`. The ciphertext `sfzefqqfxkmohihuy`
decodes as `SILVI4ALOVES2CATS`, including its two alignment indicators.
These examples and rings follow Di Palma's primary museum reconstruction,
[Leon Battista Alberti's cipher disk (2009), pages 11–13](https://www2.mathematik.uni-halle.de/reports/sources/2009/09-04report.pdf).

## Limits and errors

| Resource | Ceiling |
| --- | --- |
| Input and output text | 1 MiB each |
| Settings JSON | 512 KiB |
| Requested trace | 65,536 input symbols |
| Complete serialized report | 4 MiB |

Each ceiling applies independently. A detailed Wheatstone trace can reach the
report-byte ceiling before the symbol-count ceiling. Exhaustion returns
`resource_limit`, without truncation or a partial report. Disable tracing or
process smaller chunks using the returned continuation state.

Unsupported profiles, duplicate or unknown settings fields, invalid pointer
positions and malformed alphabets fail with `invalid_parameter`. Named-port
interfaces can reject malformed settings against their input schema first with
`invalid_input`. Unprepared text and forbidden repeated Wheatstone symbols fail
with `invalid_input` and a byte offset. Errors identify the field without echoing
private text or key values.
