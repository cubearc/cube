# KL7 rotor and re-entry cipher

Encipher or decipher prepared A–Z keyboard contact symbols with the exact
`kl7_rijmenants_v1` reference rotor/notch set. Supply eight distinct rotors,
seven notch rings, their offsets and all eight raw core positions. The fourth
rotor remains stationary. Core owns the motion and electrical re-entry; this
historical cipher provides no modern confidentiality or authentication.

`cipher.kl7.encode` and `cipher.kl7.decode` take named `text` and JSON `settings`
inputs, with no operation parameters. Both return `text`, `final_state` and
`trace`. Decode reverses every rotor-bank pass and follows identical post-step
motion. Empty text preserves state.

## Specify the complete raw state

| Setting | Meaning |
| --- | --- |
| `profile` | Required `kl7_rijmenants_v1` |
| `rotors` | Eight distinct reference identities A–M, physical slots 1–8 |
| `notch_rings` | Seven distinct IDs 1–11, on slots 1, 2, 3, 5, 6, 7, 8 |
| `notch_offsets` | Seven integers 0–35, relative to the corresponding wiring core |
| `initial_state.core_positions` | Eight integers 0–35, including the fixed fourth core |
| `trace` | Optional boolean, default `false` |

The profile fixes the thirteen qualified 36-contact rotor wirings, eleven
36-position notch patterns, base plate and ten feedback contacts. It accepts
neither arbitrary substitute wirings nor guessed extra controls. Duplicate
rotors/rings, contacts outside 0–35, missing state and unknown fields fail.

Positions are **raw core displacements**, not alphabetic window indices. On a
moving wheel, convert an operator's letter-ring index and zero-based alphabet
ring offset with `core = (window_index + alphabet_offset) mod 36`. The reference
36-position circumference is `AB1CDE2FG3HIJ4KLM5NO6PQR7ST8UVW9XYZ0`; digits in
that representation identify otherwise blank positions. Supply the fourth
core's chosen displacement directly. Notch offsets remain independent of the
alphabet ring and are sensed at `core + notch_offset + 10`, modulo 36.

No preparatory turn is inserted. Supply the state after any manual preparation.
An explicitly requested dummy-contact operation can produce a preparatory turn:
discard its text and use its returned state. A zero-length operation never steps.

Text must contain only uppercase A–Z contact labels. J is the figure-shift
contact, V returns to letters when in figures mode, and Z is the space contact.
These operations do not perform keyboard mode conversion, punctuation handling,
grouping or message-key procedures, and do not interpret returned contact labels
as already typeset plaintext.

## Follow re-entry and motion

Input crosses the inverse stationary base plate, then all eight 36-contact
rotors. Ten output contacts—2, 6, 10, 13, 16, 19, 23, 26, 30 and 35 in zero-based
numbering—feed back to the same numbered input contacts. Repeat the complete
bank until the signal reaches an external output. The fixed base plate maps
that exit to an A–Z contact. Decoding reverses the bank's order and wiring.

Every complete bank is a permutation. There are only ten internal feedback
contacts, so a valid external input exits after at most eleven passes. The
implementation enforces this bound; it never uses an unbounded re-entry loop.
Traces expose the actual `feedback_passes` count.

Movement occurs after the symbol. All seven notch decisions use the old core
positions. Each selected moving rotor increments one position modulo 36; the
fourth rotor never moves. The slot-specific controls follow the qualified
[rmsk2 KL7 mechanism](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/kl7.cpp).
The independent Rijmenants check is retained in
[its configured source test](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/kl7_test.cpp).
Core checks its full prepared-contact prefix, explicit preparatory state,
stationary fourth core, every reference rotor/ring, complete permutations,
inversion and every message split. Complete source contact/notch facts are pinned.

## Run the independent reference prefix

The source operator settings include alphabet offsets 27/1/1/17/1/1/1/1,
notch offsets K/A/A/A/A/A/A, initial windows F/A/A/M/A/A/A/A and one preparatory
cycle. Their resulting raw starting state is explicit here:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "kl7_rijmenants_v1",
    "rotors": ["A", "B", "C", "L", "D", "E", "F", "G"],
    "notch_rings": [1, 2, 3, 4, 5, 6, 7],
    "notch_offsets": [14, 0, 0, 0, 0, 0, 0],
    "initial_state": {"core_positions": [34, 0, 1, 16, 1, 1, 1, 1]}
}
with open("kl7.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher kl7 encode --parameters kl7.json --text HALLOZDIESZISTZWIEDERZEINZTESTZVVVZ
```

The result is `LPZOCRFYBRJMWHZRTSIYGTXHUODGYYIUOGP`, with final raw core
positions `[22,27,25,16,33,21,26,19]`. This is the source message's 35-contact
prefix before figure shifts, with spaces explicitly prepared as Z.

Native Python exposes `cube.kl7_encode(text, settings)` and
`cube.kl7_decode(text, settings)`. JavaScript/TypeScript calls
`cube.execute({operation: 'cipher.kl7.encode', version: 1,
inputs: {text, settings}})` through the same Core algorithm in WebAssembly.
Use returned `final_state` as the next block's `initial_state` for continuation.

## Privacy and limits

Text, key state and traces are private. Native Play requires explicit private use
and preserves the named text/settings inputs and result outputs; no linear
recipe adapter or automatic capture is supplied.

Text permits 1 MiB, serialized settings 512 KiB, trace cardinality 65,536 symbols
and the complete serialized report 4 MiB. The byte ceiling is independent and can
be reached before the trace count. Exhaustion fails without truncated or partial
output. SDK and worker transport limits remain independent, so use smaller
stateful blocks when required. Per-symbol work is at most eleven eight-rotor
passes, followed by fixed seven-ring motion decisions.
