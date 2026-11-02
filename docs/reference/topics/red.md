# RED Type A cipher

Encipher or decipher prepared A–Z contacts with the qualified
`red_type_a_vigenere_v1` circuit: sixes and twenties half-rotors, a shared
plugboard and a 47-pin breakwheel. Both operations return text and the complete
state needed to continue a message. This historical cipher provides no modern
confidentiality or authentication.

`cipher.red.encode` and `cipher.red.decode` accept named `text` and JSON
`settings` inputs, with no operation parameters. Results contain `text`,
`final_state` and `trace`. Empty text preserves state.

## Configure the circuit

| Setting | Required value |
| --- | --- |
| `profile` | `red_type_a_vigenere_v1` |
| `alphabet` | Each uppercase A–Z letter once; sixes first, twenties last |
| `pins` | 47 characters: `1` present, `0` absent; at least one present |
| `direction` | `direct` or `reverse` |
| `initial_state.breakwheel` | Zero-based physical position 0–46 |
| `initial_state.sixes` | Zero-based half-rotor position 0–5 |
| `initial_state.twenties` | Zero-based half-rotor position 0–19 |
| `trace` | Optional boolean; defaults to `false` |

The two electrical groups remain separate. Enciphering subtracts the current
half-rotor position from the input contact, modulo that group's size;
deciphering adds it. The same alphabet connects input and output. This profile
uses the direct Vigenere wiring qualified against 1937 traffic; it does not
substitute Konheim's alternative reconstruction or the later M3A commutators.

After the contact, movement starts at the **current** breakwheel pin. Skip
absent pins in the selected direction, then cross one present pin. Move both
half-rotors the same total distance in that direction. Consecutive missing pins
can produce more than two steps. The scan is bounded by 47 positions.
`trace` records the post-symbol state and `steps`, independently of electrical
encipher/decipher mode. Use `final_state` as the next block's `initial_state`.

## Run the paper example

Save `red.json`:

```json
{"profile":"red_type_a_vigenere_v1","alphabet":"TBZJNRSKYMPIHLDVWAOFQCXUEG","pins":"11111111111111111111111111111111111111111111111","direction":"direct","initial_state":{"breakwheel":0,"sixes":0,"twenties":0}}
```

```sh
cube cipher red encode --parameters red.json --text TOBEOR
```

The result is `TARCDT`. Spaces were explicitly removed; input does not silently
discard punctuation or change case. Decipher with the original settings and
`cube cipher red decode`.

The circuit, paper example and complete initial state for genuine January 1937
traffic are documented by [J.F. Bouchaudy](https://www.jfbouch.fr/crypto/red/working.html)
and [his message reconstruction](https://www.jfbouch.fr/crypto/red/messages.html).
The reference traffic uses its documented circuit mode; an intercepted ciphertext
is not automatically assigned to the API's inverse operation.

## Privacy and limits

Text, settings and outputs are private. Native Play requires explicit private
use with named inputs. Text permits 1 MiB, settings 512 KiB, optional traces
65,536 symbols, and the complete serialized report 4 MiB. The independent report
ceiling can be reached first. Exhaustion fails without partial output. SDK and
native worker transport limits remain independent; use smaller stateful blocks
when necessary.
