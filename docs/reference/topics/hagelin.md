# Hagelin pin-and-lug profiles

Encipher or decipher prepared A–Z text with a complete, explicit C-36, C-38,
M-209, C-52 or CX-52 configuration. `cipher.hagelin.encode` and
`cipher.hagelin.decode` return text, final contact indices and an optional trace.
These historical mechanisms provide no modern confidentiality or authentication.

Pass named inputs `text` and JSON `settings`, with no operation parameters.
No key is chosen automatically. Spaces, lowercase, punctuation and non-ASCII
letters fail; the caller prepares text and handles any historical space symbols,
message indicators, grouping or padding separately. Empty input preserves state.

## Choose the actual machine profile

A machine name alone does not identify its internal mechanism. These five
profiles deliberately specify their hardware arrangement:

| Required `profile` | Wheel lengths, left to right | Printing bars and advancement |
| --- | --- | --- |
| `c36_fixed25_bouch_v1` | 25, 23, 21, 19, 17 | 25 fixed, single-lug bars; weights 1, 2, 3, 7, 12; all wheels step once |
| `c38_standard27_v1` | 26, 25, 23, 21, 19, 17 | 27 bars with at most two effective lugs each; all wheels step once; required slide |
| `m209_standard_v1` | 26, 25, 23, 21, 19, 17 | 27 bars with at most two effective lugs each; all wheels step once; fixed slide 25 |
| `c52_c2_27_v1` | 26, 25, 46, 42, 38, 34 | 27 configured printing bars; remaining five bars unluggable; wheel steps 1, 1, 2, 2, 2, 2 |
| `cx52_m27_late_v1` | 47, 47, 47, 47, 47, 47 | Later M model: 27 printing bars and five fixed, nonprinting advancement bars |

The fixed C-36 profile follows [Bouch's firsthand compatibility table](https://www.jfbouch.fr/crypto/m209/compatibility.html).
Its weights differ from the C-35's 1, 2, 4, 8, 10. C-36 machines with adjustable
lugs, later 27/29-bar variants, and C-38 machines with 29 bars or rearranged
alphabets are different configurations, outside these specific profiles.

C-52 C2 uses the explicit 26, 25, 46, 42, 38, 34 order in the manufacturer's
[Description 3027, printed page 34](https://www.cryptomuseum.com/crypto/hagelin/cx52/files/CX52_Desc_3027.pdf),
with the 27-bar arrangement documented in [Wessel's investigation of actual
machines, pages 11 and 17–19](https://www.cryptomuseum.com/pub/files/BW_C52_CX52.pdf).
The fixed M-27 motion follows the manufacturer's [1964 Usage of the Hagelin
Cryptographer CX-52](https://www.cryptomuseum.com/crypto/hagelin/cx52/files/19641125_CX52_Usage.pdf).
It is distinct from standard A/B/C, early M models whose motion bars also
contributed to printing, random-tape machines, and F/V typewheel displacement.

## Supply pins, lugs and contact phase

Every profile requires `pins`, an array containing one complete binary string
per wheel, and `initial_state`, an array with the same wheel count. A `1` denotes
an effective pin, regardless of the physical side used by that model. Wrong
lengths and nonbinary characters fail instead of being repeated or truncated.

Indices are zero-based **logical reading contacts in the supplied strings**.
The current contacts generate the first output; advancement happens afterward.
They are not front-window letters or engraved face numbers. For example, the
M-209's window AAAAAA presents contact letters PONMLK. The qualification vector
uses their explicit indices 15, 14, 13, 12, 11, 10 in A-first pin strings; entering
six zero indices would describe a different starting contact state.

All profiles except fixed C-36 require `lugs`, exactly 27 rows of six booleans.
Columns denote wheels 1–6; a true cell means a lug opposite that wheel. An all-false
row is inactive. Physical left/right lug order and duplicate contacts have no
cipher effect and are represented by the same effective mask. The C-38/M-209
profiles reject rows with more than two effective lugs. C-52/CX-52 permit up to
six. The five additional motion bars in the selected C-52/CX-52 arrangements
are fixed by the profile and are not part of this printing-bar matrix.

With A=0 and Z=25, each output is `(slide - input + displacement) mod 26`.
Every printing bar with **any** effective pin/lug match contributes one to the
displacement, even when multiple lugs match. M-209 fixes `slide` at 25 and rejects
a supplied `slide` field. The other profiles require a numeric `slide` in 0–25.
Only standard opposed A–Z alphabets are supported, so encoding and decoding use
the same transformation and original starting configuration.

CX-52M samples and latches all six pin states once for the complete cycle.
Wheel 1 always steps; wheel 2 steps when pin 1 is effective; wheel 3 when either
pin 1 or 2 is effective; and so on, through wheel 6 and the first five pins.
Moving a wheel during the cycle does not resample its pin. The five motion bars
never increase the printing displacement in this later M-27 profile.

## Run and continue a message

Create a deliberately all-inactive educational M-209 configuration:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "m209_standard_v1",
    "pins": ["0" * n for n in [26, 25, 23, 21, 19, 17]],
    "lugs": [[False] * 6 for _ in range(27)],
    "initial_state": [0] * 6
}
with open("hagelin.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher hagelin encode --parameters hagelin.json --text ABC
```

This yields `"text":"ZYX"`, final state `[3,3,3,3,3,3]`, and an empty trace.
It is an explicit test configuration, not a historical daily key. The CLI also
accepts its normal file and standard-input options.

Native Python exposes `cube.hagelin_encode(text, settings)` and
`cube.hagelin_decode(text, settings)`. JavaScript/TypeScript uses
`cube.execute({operation: 'cipher.hagelin.encode', version: 1,
inputs: {text, settings}})` over the same Core algorithm in WebAssembly.

To continue, replace `initial_state` with the preceding result's `final_state`.
To decode, start from the sender's original configuration. `trace: true` adds
per-symbol offset, post-cycle state, latched `active_mask` (wheel 1 is bit 0)
and printing-bar `displacement`. Trace defaults to false; key material, input,
output and trace remain private.

## Qualification and limits

The complete 26-letter A-input check in the [US Army TM 11-380, 27 April 1942,
printed pages 16–18](https://www.cryptomuseum.com/crypto/hagelin/m209/files/m209_tm11-380_19420427.pdf)
qualifies the M-209 against its actual pin and lug tables. C-52 C2 additionally
executes that check with explicitly expanded logical contact patterns.
Hand-laid tests cover all 32 C-36 selections, all 64 CX-52M motor selections,
overlapping lugs, all slide values, wheel wraps and every split of a two-alphabet
message. These test configurations are not intercepted traffic or recovered keys.

Text is limited to 1 MiB and serialized settings to 512 KiB. A trace permits at
most 65,536 symbols; the complete serialized report independently permits 4 MiB.
A large trace can reach the report ceiling first. Limits fail instead of
truncating. CLI, native Python and JavaScript also retain their own transport,
structure, worker and output budgets; use bounded chunks and final state when a
host's ceiling is lower. Work per symbol is bounded by 27 printing bars and six
wheels. The operations run locally without automatic capture or a linear recipe
adapter; native Play uses named text/settings inputs with explicit private use.
