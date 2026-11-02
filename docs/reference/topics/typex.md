# Configured Typex Mark22

Encipher or decipher prepared A–Z contact symbols with the Typex Mark22's
three moving drums, two stators and reflected electrical path. Supply every core
wiring, orientation, ring, window, notch pattern and plug connection. The
operation supplies the machine mechanism; it does not infer a historical key
or treat an Enigma rotor set as a Typex set.

`cipher.typex.encode` and `cipher.typex.decode` take named `text` and JSON
`settings` inputs, with no operation parameters. Both return `text`,
`final_state: {windows: ...}` and `trace`. The path is reciprocal, so decoding
uses the sender's original settings. Empty text preserves the state.
This historical cipher provides no modern confidentiality or authentication.

## Specify the complete configuration

| Setting | Meaning |
| --- | --- |
| `profile` | Required `typex_mk22_configured_v1` |
| `wheel_set_id` | Required caller identity for this complete wiring set |
| `rotors` | Five distinct named cores, physically left to right: slow, middle, fast, stator 2, stator 1 |
| `rotors[i].id` | Required distinct core identity |
| `rotors[i].wiring` | Complete permutation of A–Z, normal core orientation at zero displacement |
| `rotors[i].reversed` | Required boolean: turn the wiring core over independently of its casing |
| `ring_settings` | Five uppercase letters in the same physical order; A means zero offset |
| `turnover_windows` | Two nonempty strings of distinct A–Z letters: old middle and fast windows that permit turnover |
| `entry_wiring` | Complete permutation from keyboard A–Z contact labels into the stationary right entry plate |
| `plugboard` | Complete arbitrary A–Z permutation; use the explicit alphabet for an unconnected board |
| `reflector` | Complete A–Z permutation made of thirteen reciprocal pairs with no fixed points |
| `initial_state.windows` | Exactly five uppercase window letters in physical left-to-right order |
| `trace` | Optional boolean, default `false` |

Set and core identities contain 1–128 ASCII letters, digits, dots, underscores
or hyphens, starting with a letter or digit. They identify the caller's data;
they do not select or download tables. Duplicate core identities, incomplete
permutations, duplicate notch letters, invalid windows, unpaired reflector
contacts and unknown controls fail. No setting other than `trace` defaults.
The slow casing's unused turnover notches do not enter the stepping control.

Text must contain only uppercase ASCII A–Z. These are prepared contact symbols,
not automatic Typex keyboard entry. Spaces, lowercase, digits and other Unicode
characters fail. Letter/figure shifts, the space key, punctuation, grouping,
padding and message-key procedures require separate preparation.

## Advance and reverse the cores

Stepping samples the **old** middle and fast notch windows. The fast drum then
advances one letter on every symbol. The middle drum advances if either the
middle or fast notch is active; the slow drum advances if the middle notch is
active. Both stators remain at their supplied windows. A central drum can
therefore step on two consecutive symbols. Z wraps to A.

Current flows after movement, as described in the primary
[Typex maintenance manual, printed page 3, paragraph 8](https://www.cryptomuseum.com/crypto/uk/typex/files/Typex_Manual_IB_II_III_VI.pdf).
The [Crypto Museum inspection](https://www.cryptomuseum.com/crypto/uk/typex/index.htm)
distinguishes the two stationary right-hand drums, removable wiring cores and
Mark22 input and reflector plugboards.

A normal core at displacement `d = window - ring` maps contact `x` to
`W(x + d) - d`, modulo 26. Turning a core over exchanges its faces and reverses
contact numbering: the reversed map is `R(x) = -W^-1(-x)`. Merely taking the
inverse permutation is insufficient. Core reversal does not turn over the
casing or change its supplied notch windows.

The path crosses the arbitrary plugboard and entry plate, the five cores from
right to left, and the reflector. It returns through the inverse core maps,
entry plate and plugboard. An arbitrary plug permutation need not exchange
letter pairs; its return path uses its inverse. Only the reflector must consist
of pairs. The resulting complete path is reciprocal and has no self encipherment.

## Run a configured interoperability example

This SP02390 configuration reproduces an explicitly configured simulator check
from [rmsk2's independent Typex tests](https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/typex_test.cpp).
It is a simulator interoperability set, **not a claimed wartime daily key**.
Its keyboard test is represented here as fifty prepared A–Z contact symbols.

```sh
python3 - <<'PY'
import json
rotors = [
    "RFNVBKTIHXQGCWAEOLSMPYDZUJ",
    "OLTGENZUJDIBYPSAFWRQMCXKHV",
    "TWBHQDURMLNIEAKSVOYCJGXFPZ",
    "LYUMXSFBPZONKJCEQIATGWRHDV",
    "KGBTYSOAIVXCJPRQZNHLFWUEMD"
]
alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
entry = "QWERTZUIOASDFGHJKPYXCVBNML"
settings = {
    "profile": "typex_mk22_configured_v1",
    "wheel_set_id": "rmsk2-SP02390-interoperability",
    "rotors": [{"id": name, "wiring": wiring, "reversed": False}
               for name, wiring in zip("ABCDE", rotors)],
    "ring_settings": "AAAAA",
    "turnover_windows": ["FMSZ", "FMSZ"],
    "entry_wiring": "".join(alphabet[entry.index(c)] for c in alphabet),
    "plugboard": alphabet,
    "reflector": "RYUHQSLDXPNGOKMJEAFWCZTIBV",
    "initial_state": {"windows": "AAAAA"}
}
with open("typex.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher typex encode --parameters typex.json --text QWERTYUIOPASDFGHJKLXCBNMZQWERTYUIOPASDFGHJKLXCNMVA
```

The result is `HVDNGQYLGGHJOKKIOXPEQLFEMXNWIZAOMSSRMFSVVPUACYKUCN`, with
final windows `BIYAA`. A separate independent check uses the same five cores
with only the fast C core reversed and reproduces its published ciphertext
prefix. Stepping, reversed orientations, arbitrary plug permutations, all
message splits and both stationary windows are checked separately.

Native Python exposes `cube.typex_encode(text, settings)` and
`cube.typex_decode(text, settings)`. JavaScript/TypeScript calls
`cube.execute({operation: 'cipher.typex.encode', version: 1,
inputs: {text, settings}})` through the same Core algorithm in WebAssembly.

## Continue a message and interpret limits

Supply a returned `final_state` as the next block's `initial_state`. Traces, when
requested, contain each input offset and its post-step five-window state.
Text, complete settings and state traces are private. Native Play uses the named
text/settings inputs with explicit private use, without a linear recipe adapter.

Text permits 1 MiB, serialized settings 512 KiB, optional trace 65,536 symbols,
and the complete serialized report 4 MiB. Exceeding a limit fails instead of
truncating the text or trace. SDK and native worker transport/structure budgets
remain independent and may require smaller chunks. Per-symbol work is fixed
three-drum motion and a five-core reflected path; execution remains local.
