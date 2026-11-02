# SIGABA CSP889

Encipher or decipher prepared A–Z contact symbols with SIGABA's cipher,
control and index banks. Select the exact `sigaba_csp889_v1` reference profile,
all fifteen wheel orientations and all physical windows. The profile uses the
qualified CSG/rmsk2 contact set; it does not infer a daily key or perform an
O reset. These historical operations provide no modern security or authentication.

`cipher.sigaba.encode` and `cipher.sigaba.decode` take named `text` and JSON
`settings` inputs, with no operation parameters. Results contain `text`,
`final_state` and `trace`. Decode reverses the cipher path while preserving the
same independent control motion. Empty text preserves state.

## Set the three banks

| Required setting | Meaning |
| --- | --- |
| `profile` | `sigaba_csp889_v1` |
| `cipher_rotors` | Five objects `{id, reversed}`, left to right |
| `control_rotors` | Five objects `{id, reversed}`, left stator, slow, fast, middle, right stator |
| `index_rotors` | Five objects `{id, reversed}`, left to right |
| `initial_state.cipher_windows` | Five uppercase window letters, left to right |
| `initial_state.control_windows` | Five uppercase window letters in control-bank order |
| `initial_state.index_windows` | Five digits, left to right |
| `trace` | Optional boolean, default `false` |

The ten large reference wheel IDs are **0–9**, shared between cipher and control
banks: use each once. The five index IDs are **0–4**: use each once. Each wheel
requires an explicit boolean `reversed`. IDs refer to this published reference
set rather than an inferred machine serial number or an arbitrary replacement
wiring. The three banks and their window strings must be complete.

A reversed wheel turns its wiring over and reverses contact numbering. Its
visible letters or digits therefore represent the additive inverse of its raw
core displacement. During backward core motion, a normal window moves A→Z,
while a reversed window moves A→B. Index wheels remain stationary.

CSP2900 controls, alternate contact sets, setup-key sequences and automatic
space/letter conversion are not controls of this precise profile. Unknown fields
and incompatible profiles fail. Text must contain only uppercase A–Z contact
labels. The historical plaintext space contact is labeled **Z**; replace spaces
explicitly during preparation if that is the intended convention.

## Follow the control motion

The current symbol crosses the five cipher wheels at their old positions.
Encoding traverses the forward bank left to right; decoding uses its inverse
right to left. There is no reflector and no assumed prohibition on a letter
enciphering to itself.

After the symbol, contacts F, G, H and I energize the old control bank from
right to left through its inverse wiring. Its 26 outputs enter the ten index
inputs in these fixed groups:

| Control outputs | Index input |
| --- | --- |
| A | 9 |
| B | 1 |
| C | 2 |
| D–E | 3 |
| F–H | 4 |
| I–K | 5 |
| L–O | 6 |
| P–T | 7 |
| U–Z | 8 |

The stationary index bank maps each energized group to a motor. Outputs 0/9
select cipher slot 1; 1/2 select slot 5; 3/4 select slot 4; 5/6 select slot 3;
7/8 select slot 2. Selections combine by OR, so repeated currents never move a
wheel twice in one cycle.

All decisions use old positions. The fast control wheel then moves backward
one core contact. The middle control wheel moves if the old fast window is O;
the slow wheel moves only if both old fast and middle windows are O. The two
outer control wheels and all index wheels stay fixed. Finally, every selected
cipher wheel moves backward once. Returned traces contain this post-step state
and `stepped_mask`, whose bit 0 refers to the leftmost cipher wheel.

The physical machine is described by the
[US Navy ECM instructions](https://maritime.org/doc/crypto/ecm/sec02.php).
The exact contact set and control equations follow
[rmsk2's pinned SIGABA implementation](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/sigaba.cpp),
with the complete 105-character independent CSG check retained in
[its source tests](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/sigaba_test.cpp).
Core also checks the 26, 676 and 17,576-symbol control-bank cycles, each individual
wheel reversal, complete input/output permutations, inverse operation and all
message splits. Source contact facts are pinned as a complete data set.

## Run a configured example

Save a complete reference configuration:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "sigaba_csp889_v1",
    "cipher_rotors": [{"id": i, "reversed": i == 2} for i in range(5)],
    "control_rotors": [{"id": i, "reversed": i == 7} for i in range(5, 10)],
    "index_rotors": [{"id": i, "reversed": i == 2} for i in range(5)],
    "initial_state": {
        "cipher_windows": "LJQMJ",
        "control_windows": "KOOON",
        "index_windows": "09000"
    }
}
with open("sigaba.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher sigaba encode --parameters sigaba.json --text DIESZISTZE
```

The ciphertext is `HHHTITJPTH`. Z explicitly represents the space in the prepared
source phrase. Decoding starts from the same original bank selection and windows.
Use `final_state` as the next block's `initial_state` to continue a stream.

Native Python exposes `cube.sigaba_encode(text, settings)` and
`cube.sigaba_decode(text, settings)`. JavaScript/TypeScript calls
`cube.execute({operation: 'cipher.sigaba.encode', version: 1,
inputs: {text, settings}})` through Core in WebAssembly.

## Privacy and limits

Text, settings and every output are private. Native Play requires explicit private
use with its named text/settings inputs; no linear recipe adapter or automatic
capture is supplied.

Text permits 1 MiB, serialized settings 512 KiB, trace cardinality 65,536 symbols
and the complete serialized result 4 MiB. The report-byte ceiling can be reached
before the trace-count ceiling. Exhaustion fails without truncation or partial
output. SDK and worker transport limits remain independent: process smaller
blocks with returned state when required. Work per symbol is a fixed five-wheel
cipher path, four five-wheel control paths and bounded index/motor selection.
