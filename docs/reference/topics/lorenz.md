# Lorenz five-bit streams

Encipher or decipher unpacked teleprinter codes with a complete Lorenz wheel
configuration and continuation state. Choose SZ40, SZ42a or SZ42b explicitly.
The operations preserve all five impulses, including control codes; interpreting
ITA2 letters, figures, spacing, start/stop framing or audio is separate.
These historical ciphers provide no modern confidentiality or authentication.

| Operation | Result |
| --- | --- |
| `cipher.lorenz.encode` | Cipher codes, final state and optional trace |
| `cipher.lorenz.decode` | Recovered plain codes, final state and optional trace |

## Codes and settings

Named-port calls accept `codes`, an array of integers 0–31, and a JSON `settings`
input. There are no operation parameters. Impulse 1 is the least significant bit;
impulse 5 has value 16. Negative numbers, booleans, strings and values above 31
are rejected. Empty input returns unchanged state.

All machine settings except `trace` are required:

| Setting | Meaning |
| --- | --- |
| `profile` | `lorenz_sz40_v1`, `lorenz_sz42a_v1` or `lorenz_sz42b_v1` |
| `chi` | Five binary pin strings, lengths 41, 31, 29, 26 and 23 |
| `psi` | Five binary pin strings, lengths 43, 47, 51, 53 and 59 |
| `mu61`, `mu37` | Binary pin strings of 61 and 37 contacts |
| `plaintext_limitation` | Explicit KT switch; SZ40 requires `false` |
| `initial_state` | Twelve contact indices and delayed history, described below |
| `trace` | Optional boolean, default `false` |

Pin strings contain only `0` and `1`. The five signal wheels follow impulse
order 1–5; no default key or historical daily pin sheet is supplied. The full
configuration contains 501 pins. Malformed patterns and wrong lengths fail
instead of being repeated, padded or truncated.

## Contact phase and continuation

`initial_state` contains `chi` and `psi` arrays of five indices, scalar `mu61` and
`mu37` indices, booleans `chi2_previous` and `psi1_previous`, and
`plain5_previous`, an array of two booleans ordered **two characters back, then
one character back**. All history values are explicit, including at the beginning
of a message. The model never silently assumes zero history.

Indices are zero-based positions in the supplied pin strings. The current
contacts produce the first output; a step advances an index by one modulo its
wheel length. These are logical contact indices, rather than physical front-window
numbers. Arrange pin sequences and starting indices for the intended reading
contact and direction; no unverified window-to-contact offset is inferred.
The delayed Psi1 signal is from the previous processed character, even when its
wheel did not move.

The result's `final_state` is ready to become the next block's `initial_state`.
Use the sender's original state to decode the complete message. With KT enabled,
the decoder updates history from recovered plaintext; continuing encoding and
decoding therefore yields identical final states for the same message.

## Model-specific movement

Each output code is input XOR the current Chi code XOR the current Psi code.
After that output, all Chi wheels and M61 advance. M37 advances only when the
**old** M61 contact is 1. The old M37 contact is the basic motor signal.

| Profile | Limitation before a step |
| --- | --- |
| SZ40 | None; Psi advances only when the basic motor is 1 |
| SZ42a | Previous Chi2 impulse |
| SZ42b | Previous Chi2 XOR previous Psi1 impulse |

On SZ42a/b, enabling KT XORs the plaintext fifth impulse from two characters
back into that limitation. All five Psi wheels advance together when the basic
motor is 1 **or** the resulting limitation is 0. Otherwise all five remain still.
The basic motor overrides a stop limitation.

These profiles follow the primary [TICOM T663 device description, printed pages
8–11 and 17–19](https://cryptocellar.org/Lorenz/T663_SZ42_Vol_1.pdf), the actual
circuit timing in [Sawyers's restoration report, sections 4.3–4.4](https://www.vintage-radio.net/forum/attachment.php?attachmentid=320238&d=1760352002),
and the explicit initial-history switches described in the [General Report on
Tunny, 53C(d)](https://www.codesandciphers.org.uk/documents/newman/PAGE335.HTM).

## Run a configured stream

The CLI reads settings from `--parameters` and a JSON code array through its
usual `--text`, file or standard-input options. To create an explicit all-dot
qualification key, save settings using Python's standard library:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "lorenz_sz40_v1",
    "chi": ["0" * n for n in [41, 31, 29, 26, 23]],
    "psi": ["0" * n for n in [43, 47, 51, 53, 59]],
    "mu61": "0" * 61, "mu37": "0" * 37,
    "plaintext_limitation": False,
    "initial_state": {
        "chi": [0] * 5, "psi": [0] * 5, "mu61": 0, "mu37": 0,
        "chi2_previous": False, "psi1_previous": False,
        "plain5_previous": [False, False]
    }
}
with open("lorenz.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher lorenz encode --parameters lorenz.json --text '[0,31,5]'
```

The result contains `"codes":[0,31,5]`, because every signal pin is zero. It also
returns the advanced Chi/M61 indices and an empty trace. This deliberately weak
qualification key is explicit input, never an operation default.

A separate checked vector uses the complete Bream pin facts from [Lasry's 2020
research, appendix](https://ep.liu.se/ecp/171/013/ecp2020_171_013.pdf).
Starting at logical contact zero, the first four Chi codes are `2,18,12,13` and
the sampled Psi codes are `12,16,16,15`. Input `[0,1,31,2]` therefore produces
`[14,3,3,0]`. This is a hand-read configured transition example, not an intercepted
message or a claim that logical zero corresponds to a particular physical window.

Native Python exposes `cube.lorenz_encode(codes, settings)` and
`cube.lorenz_decode(codes, settings)`. JavaScript uses the same named inputs through
`cube.execute`; generated TypeScript requires the complete settings and history.
In native Play, connect both the codes and settings as structured JSON inputs.

## Trace, privacy and limits

When `trace` is true, each row contains the zero-based input `offset`, the
**post-step** `state`, the sampled `key_code`, `basic_motor` and `psi_step`.
Key codes and histories reveal private machine state. Inputs and every output
remain private; retention requires explicit consent. No linear recipe adapter or
automatic capture is supplied.

| Resource | Ceiling |
| --- | --- |
| Unpacked input/output codes | 1,048,576 each |
| CLI/native Python JSON code transport | 4 MiB |
| Settings JSON | 512 KiB |
| Optional trace count | 65,536 input codes |
| Complete serialized result | 4 MiB |

The report-byte ceiling can be reached before the trace-count ceiling. Exhaustion
returns `resource_limit` without truncation or a partial result. Disable tracing
or process smaller blocks with continuation state. Play and browser forms apply
smaller host and input-conversion budgets in addition to these Core limits. The
JavaScript SDK also admits at most 65,536 JSON nodes per complete invocation,
including every code and the settings. Split larger code arrays into blocks and
continue from `final_state`; the native Core code-count ceiling does not override
this independent transport limit.

Malformed settings, unsupported profiles, invalid indices and KT on SZ40 return
`invalid_parameter`; named-port interfaces can reject malformed settings against
the input schema first with `invalid_input`. Invalid codes return `invalid_input`;
native Core range errors include the code index. Errors do not echo key or message
values.
