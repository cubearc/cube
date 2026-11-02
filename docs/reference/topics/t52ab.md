# Siemens T52a/b five-bit streams

Encipher or decipher unpacked teleprinter codes with the Siemens T52a/b XOR
and pluggable double-changeover switch circuit. Supply complete fixed cam facts,
plug assignments and logical reading contacts. `cipher.t52ab.encode` and
`cipher.t52ab.decode` return codes, final state and optional trace.
This historical cipher provides no modern confidentiality or authentication.

## Codes and required configuration

Named inputs are `codes`, an array of integers 0–31, and JSON `settings`.
There are no operation parameters. Impulse 1 is the least significant bit;
impulse 5 has value 16. Codes preserve all five impulses, including control
values. ITA2 text interpretation, bit packing, framing and transmission
procedures are separate. Empty input preserves state.

The exact profile is `siemens_t52ab_configured_v1`. Every setting except
`trace` is required:

| Setting | Meaning |
| --- | --- |
| `wheel_set_id` | Caller-supplied identity of the fixed cam set; 1–128 ASCII letters/digits/dots/underscores/hyphens, starting with a letter or digit |
| `cam_patterns` | Ten complete binary strings, in ascending wheel-length order: 47, 53, 59, 61, 64, 65, 67, 69, 71, 73 |
| `wheel_order` | Each logical wheel ID 0–9 exactly once; first five feed XOR impulses 1–5, last five control the corresponding socket pairs |
| `socket_pairs` | Five two-socket arrays; every socket 1–10 occurs exactly once |
| `initial_state` | Ten zero-based logical contact indices in the supplied cam strings |
| `trace` | Boolean, default `false` |

The 629 cam contacts were fixed physical facts of a machine's manufactured wheel
set. They are supplied explicitly with an identity, rather than chosen as a
message key or inferred from a factory-set name. No historical factory pattern
is silently supplied. A `1` denotes the active cam contact; a `0` denotes inactive.
Wrong lengths, nonbinary contacts, duplicate assignments and incomplete plugs
fail instead of being filled, repeated or normalized.

The profile follows the [1937 OKM preliminary description, M.Dv.Nr.35](https://www.cryptomuseum.com/crypto/siemens/t52/files/Geheimzusatz_FSM_T52_OKM_1937.pdf)
and [Weierud's primary reconstruction, pages 3–5 and 21–23](https://cryptocellar.org/pubs/weierud_sturgeon.pdf).
The 1944 addendum in the manual describes wheel stopping as an addition made
for T52d. The selected a/b profile advances **all ten wheels once after every
code**. Other T52 variants have distinct combination and movement circuits;
unknown profile and control fields are rejected.

## Contact phase and the switch circuit

Indices identify the contacts currently read in the supplied strings. They are
not engraved front-window positions. After producing a code, every index
increases by one modulo its wheel length. The returned `final_state` is ready
to become the next block's `initial_state`; decode with the sender's original
state. No unverified window offset or rotation direction is inferred.

Encoding first XORs the five input impulses with the first five assigned cam
signals, then applies the actual switch permutation. Decoding applies the
inverse permutation **before** XOR. Each of the last five assigned wheels
controls its two plugs: active `1` passes straight; inactive `0` crosses to the
paired plug. Red/black ordering within a pair does not change this circuit.

The fixed socket layout, from Weierud's Figure 8, is:

| Impulse | First socket | Second socket |
| --- | --- | --- |
| 1 | 1 | 10 |
| 2 | 8 | 9 |
| 3 | 6 | 7 |
| 4 | 4 | 5 |
| 5 | 2 | 3 |

A first socket's right contact connects to the second socket's left contact in
its row. A second socket's right contact reaches that row's output impulse.
Crossed plugs can route a signal through several contact units. The operation
resolves this physical network for all 32 control selections before processing
the stream. Every resulting map connects the five input impulses bijectively
to the five output impulses.

There are 945 ways to partition the ten sockets into five pairs. All are accepted
as physical configurations, including degenerate weak arrangements; no key is
randomly generated or changed to improve its distribution.

## Run and continue a stream

This explicit all-active cam set is a synthetic qualification example, not an
asserted historical factory pattern:

```sh
python3 - <<'PY'
import json
settings = {
    "profile": "siemens_t52ab_configured_v1",
    "wheel_set_id": "synthetic-all-active",
    "cam_patterns": ["1" * n for n in [47, 53, 59, 61, 64, 65, 67, 69, 71, 73]],
    "wheel_order": list(range(10)),
    "socket_pairs": [[1, 3], [2, 10], [4, 5], [6, 7], [8, 9]],
    "initial_state": [0] * 10
}
with open("t52ab.json", "w") as output:
    json.dump(settings, output)
PY
cube cipher t52ab encode --parameters t52ab.json --text '[0,31,5]'
```

All five XOR impulses are 1, and every switch passes straight. The output is
`[31,0,26]`, final state contains ten 3s, and trace is empty. The CLI also accepts
its usual file and standard-input options for the JSON code array.

Native Python exposes `cube.t52ab_encode(codes, settings)` and
`cube.t52ab_decode(codes, settings)`. JavaScript/TypeScript uses
`cube.execute({operation: 'cipher.t52ab.encode', version: 1,
inputs: {codes, settings}})` through the same Core algorithm in WebAssembly.

With `trace: true`, each row includes the input `offset`, post-cycle `state`,
`xor_code` and `permutation`. That permutation maps input impulse indices 0–4
to output impulse indices 0–4, in the encoding direction. Inputs, cam facts,
plugs, results and traces are private; native Play requires explicit private use
and accepts the named structured inputs without a linear recipe adapter.

## Qualification and limits

The independent Figure 8 hand check plugs units into 1–3, 2–10, 4–5, 6–7 and
8–9. With the first unit inactive and the other four active, impulses 1 and 5
exchange. XOR code 1 therefore changes input 0 into output 16. The full check
`[0,1,2,4,8,16,31]` yields `[16,0,18,20,24,17,15]`; decoding reverses it exactly.
Its constant cam strings are labeled synthetic, rather than intercepted traffic
or a historical factory set.

An exhaustive independent pairing test reproduces the published Figure 10
statistics: 24 pairings yield 27 distinct maps, 240 yield 30, 120 yield 32, and
561 yield at most 16; exactly one pairing yields only the identity map.
Other checks cover all 32 codes, wheel wraps, every split of a 96-code stream,
invalid plugs/cams and bounded input/trace/report failures.

Core permits 1,048,576 unpacked codes. Settings permit 512 KiB, code JSON transport
4 MiB, trace 65,536 symbols and the full serialized report 4 MiB, independently.
A large trace can reach the report ceiling first. Failures never truncate data.
SDK host limits, including document nodes and worker/input/output budgets, can
be lower; process bounded blocks and reuse final state. No host budget is raised
to reach a native ceiling. Per-symbol work is bounded by ten contact reads, ten
advancements and five impulse mappings, with a fixed 32-map configuration step.
