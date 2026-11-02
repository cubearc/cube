# Swiss NEMA Model45

Encipher or decipher prepared uppercase A–Z with the operational or training
NEMA Model45. The operations use the measured A–F contact wiring, moving
reflector and model-specific drive wheels. Supply the complete wheel selection
and ten physical lower-row window letters; only trace defaults to false.
This historical cipher provides no modern confidentiality or authentication.

`cipher.nema.encode` and `cipher.nema.decode` take named `text` and JSON
`settings` inputs, with no operation parameters. Results contain `text`,
`final_state: {windows: ...}` and `trace`. Empty text preserves state. Spaces,
lowercase, digits, punctuation and non-ASCII letters fail. Historical number
shifts, spacing, grouping and message-key procedures are separate preparation.

## Choose the exact machine and key

| Required setting | Meaning |
| --- | --- |
| `profile` | `nema_model45_operational_v1` or `nema_model45_training_v1` |
| `rotors` | Four distinct contact wheel identities, physically left to right |
| `drive_wheels` | Four distinct drive wheel numbers, paired with those contact wheels in the same order |
| `initial_state` | Object with exactly ten uppercase `windows`, read left to right from the lower row under the opened wheel cover |
| `trace` | Optional boolean, default `false` |

| Profile | Contact wheels available | Drive wheels available | Fixed red wheel |
| --- | --- | --- | --- |
| Operational | A, B, C, D, E, F | 12, 13, 14, 15, 17, 18 | 22/1 |
| Training | A, B, C, D | 16, 19, 20, 21 | 23/2 |

The two Army blocking arms are fixed at combined positions C2 and C4. Notch
rings use their prescribed mounting alignment: two-digit identification at U,
and the right red ring at L. Wheel choices from the other model, duplicated
components and unrecognized controls are refused. No daily key, random wheel
order or starting windows are inferred.

These profiles follow [Sullivan and Weierud's firsthand study, Figures 2–12](https://cryptocellar.org/pubs/nema.pdf)
and the [direct contact measurements and machine inspection at Crypto Museum](https://www.cryptomuseum.com/crypto/nema/index.htm).
The contact B table uses the independently measured J→O connection, correcting
the duplicate 17 at that position in the study's Figure 3. All contact tables
are checked as complete permutations; the reflector has thirteen reciprocal
pairs and no fixed points.

## Physical ordering and timing

Numbering from the right, physical positions 1, 3, 5, 7 and 9 are drive wheels;
2, 4, 6 and 8 are contact wheels; position 10 is the reflector. Position 1 is
the red drive wheel, whose electric entry plate is stationary. The ten window
letters are supplied in the opposite direction, as seen on the actual machine:
reflector at the left, red drive at the right.

All stepping decisions sample the old positions. Ordinary notch fingers sense
at window minus 10 letters; the red right probing lever senses at window minus
9. A `1` is an active low notch, permitting movement. Drive positions 1, 5 and
9 always move. The right red cam controls drive positions 3/7 and blocks their
paired contact wheels 4/8; each permitted contact wheel additionally requires
an active notch on its own drive wheel. Contacts 2, 6 and the reflector 10
follow their own right-hand drive cams. Every selected window moves backward
one letter, with A wrapping to Z.

Current flows only after this simultaneous movement. The static keyboard entry
uses the study's clockwise QWERTZU sequence starting at contact 1, labeled I.
The electrical coordinate convention uses that I reference and the two-row
window benchmark; in an A-numbered array, a contact wheel's displacement is
`window - 10`. Add the displacement before its measured permutation and subtract
it afterward. The path crosses the four contact wheels from right to left,
passes through the moving reflector, then returns through the inverse contact
permutations and keyboard entry. This reflected path makes the transformation
reciprocal and prevents a letter from enciphering to itself.

The coordinate convention and complete reference are independently checked
against [Sullivan's simulator reference retained in rmsk2](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/nema_test.cpp).
The source study's full 26-step sequence and the reference ciphertext/final
windows are verification data, rather than fitted offsets or an inferred key.

## Run the source reference

Save this complete explicit configuration:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "nema_model45_operational_v1",
    "rotors": ["A", "B", "C", "D"],
    "drive_wheels": [12, 13, 14, 15],
    "initial_state": {"windows": "MQQPZQSOEV"}
}
with open("nema.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher nema encode --parameters nema.json --text AAAAAAAAAAAAAAAAAAAAAAAAAAA
```

The 27 A letters yield `HRSBVOOTZUCRWLMGRMGVWYWOVNF`, and the final state is
`{"windows":"QPMKHPPJPU"}`. Decoding starts from the original `MQQPZQSOEV`,
with the same selected wheels. The CLI also accepts its usual file and standard
input options.

Native Python exposes `cube.nema_encode(text, settings)` and
`cube.nema_decode(text, settings)`. JavaScript/TypeScript executes
`cube.execute({operation: 'cipher.nema.encode', version: 1,
inputs: {text, settings}})` through the same Core algorithm in WebAssembly.

To continue in blocks, copy `final_state` into the next block's `initial_state`.
Explicit `trace: true` returns the post-step window state for every input offset.
The text, settings and trace are private; native Play requires explicit private
use and uses the named text/settings inputs without a linear recipe adapter.

## Qualification and limits

Checks include the complete 27-letter cipher reference and final windows, every
published state of the 26-step example, the 17,576-step state period of that
configuration, both models, all six contact wheels, exact inverse behavior,
empty input and every split of a two-alphabet message. Invalid wheel sets,
windows, extra fields and private malformed text are refused.

Text permits 1 MiB, settings JSON 512 KiB, optional trace 65,536 symbols and the
complete serialized report 4 MiB. Exact limits and one-over failures are checked;
results are never truncated. SDK and worker transport/structure limits remain
independent and can be lower, so use bounded chunks with returned state.
Per-character work is fixed ten-wheel stepping plus the five-wheel reflected
contact path. Execution is local, without automatic capture or network access.
