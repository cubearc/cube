# Enigma I and M3

Transform a configured historical Enigma message and read its final rotor windows.
`cipher.enigma` is reciprocal with identical initial settings. These machines
provide no modern confidentiality or authentication.

## Settings

The CLI reads `--parameters settings.json`; Rust uses `enigma::Parameters` and
native Python uses `cube.enigma(text, parameters)`. No machine key is inferred.

| Field | Values or default |
| --- | --- |
| `profile` | Required: `enigma_i` or `m3` |
| `rotors` | Required: three distinct names, left to right |
| `reflector` | Required: wide `B` or `C` |
| `rings` | Required: three integers 1–26, left to right; 1 means A |
| `windows` | Required: three uppercase letters, left to right |
| `plugboard` | Required: array of 0–10 disjoint uppercase letter pairs |
| `text_mode` | `strict` by default; explicit `preserve` wrapper available |
| `trace` | `false` by default |

Enigma I allows rotors I–V; M3 allows I–VIII. Entry wiring is the identity.
M4, thin reflectors, commercial models, UKW-D and message-key procedures are
unsupported. Pair order and orientation do not change the plugboard permutation.
Unknown fields, duplicate JSON keys and unsupported names fail.

## Letters and stepping

Strict mode accepts only ASCII A–Z, including an empty message. Preserve mode
retains ASCII letter case and every other Unicode scalar without normalization;
only ASCII letters step the machine. Remove group separators for strict messages
or retain them explicitly with preserve mode.

The rightmost rotor steps before each electrical traversal. The middle rotor
steps when it or the right rotor is at turnover, and the left rotor steps when
the middle is at turnover. Decisions use the pre-keypress state. This reproduces
the middle rotor's double step. Visible turnover letters are Q/E/V/J/Z for I–V
and M/Z for VI–VIII, independent of ring settings. Rings shift the electrical
wiring relative to the visible index ring.

## Results and limits

| Output | Meaning |
| --- | --- |
| `text` | Transformed text, with the same UTF-8 byte length |
| `final_windows` | Windows after the final enciphered letter |
| `trace` | Post-step windows and input UTF-8 byte offset for each letter; empty when disabled |

Use `final_windows` to continue a message with the other settings unchanged.
Reset to the original windows to decrypt. Traces contain key-state information.
Play treats input, settings and results as private; retaining a result requires
the explicit per-run choice. No pipeline adapter is advertised.

| Resource | Maximum |
| --- | --- |
| Input and transformed text | 1 MiB each |
| Settings JSON transport | 4 KiB |
| Optional trace | 65,536 letters; serialized trace below 4 MiB |
| Rotor work | Six fixed lookups per letter |

Excess input or trace returns `resource_limit` before transformation. Malformed
settings return `invalid_parameter`; strict-mode text errors return
`invalid_input` with a UTF-8 byte offset. Errors contain field names, not key values.

## Provenance

Wiring and turnover conventions follow Crypto Museum's
[measured rotor tables](https://www.cryptomuseum.com/crypto/enigma/wiring.htm).
Mechanical stepping follows David Hamer's
[double-step analysis, table 1](https://www.cryptomuseum.com/people/hamer/files/double_stepping.pdf).
Verification includes the Scharnhorst intercept of 26 December 1943: III/VI/VIII,
reflector B, rings 01/08/13, windows UZV and pairs AN EZ HK IJ LR MQ OT PV SW UX.
The message begins `YKAENZAP` and decodes to `STEUEREJ`.
The [historical-message collection](https://www.enigma.hoerenberg.com/index.php?cat=Historical%20Messages)
records the complete configured message. No manual, simulator code or archive is
bundled; the fixed wiring facts are implemented in Rust.
