# Fialka M-125 cipher

Encipher or decipher unpacked thirty-contact values with Fialka M-125's 6K
PROTON I rotor set. Supply all ten rotors, ring settings, punched-card contacts
and rotor positions. Core performs the complete electrical path and opposing
pin-controlled motion. This historical cipher provides no modern confidentiality
or authentication.

`cipher.fialka.encode` and `cipher.fialka.decode` take named `codes` and JSON
`settings` inputs, with no operation parameters. Results contain `codes`,
`final_state` and `trace`. Empty input preserves state.

## Configure the thirty-contact mechanism

| Setting | Required value |
| --- | --- |
| `profile` | `fialka_m125_6k_proton_i_v1` |
| `rotor_order` | Each reference rotor ID 1–10 once, reflector to entry disk |
| `ring_offsets` | Ten raw offsets 0–29 in physical slot order |
| `punch_card` | Complete permutation of contacts 0–29 |
| `initial_state.rotor_positions` | Ten raw offsets 0–29 in physical slot order |
| `trace` | Optional boolean; defaults to `false` |

The profile fixes the 6K contact and blocking-pin tables, keyboard permutation,
entry disk and directional reflector. PROTON I cores stay in their own rotor,
unflipped, without independently rotated cores. Other rotor sets, PROTON II,
NumLock10 feedback and automatic language/printing modes are not inferred.

Input values are electrical keyboard contact labels 0–29. For the published
Czechoslovak Latin letter layout, their order is
`FADULT5PBRKVZJGHCNE7QWXIOSM8Y2`. Prepare letters, space contacts or shifts
explicitly; returned values are contact labels, not automatically printed text.

## Interpret the electrical path and motion

Input crosses keyboard, punched card and entry disk, then enters the ten rotors
from right to left with the fixed three-contact frame shift. The reflector
contains a three-cycle as well as paired contacts, so deciphering uses its
**inverse**. The signal returns through inverse rotors, entry disk, punched card
and keyboard. Each rotor's electrical displacement is its raw position plus ring
offset, modulo 30.

After the symbol, two independent chains move. Slots 2, 4, 6, 8, 10 decrement
from left to right; slots 9, 7, 5, 3, 1 increment from right to left. A rotor moves
once before its old blocking pin can stop the remaining rotors in that chain.
The respective old pin-sensing offsets are 17 and 20, including the ring offset.
Every symbol therefore moves at least slots 2 and 9. Trace states are post-step.
Use `final_state` as the next block's `initial_state` to continue.

## Run the independent reference

Save `fialka.json`:

```json
{"profile":"fialka_m125_6k_proton_i_v1","rotor_order":[1,2,3,4,5,6,7,8,9,10],"ring_offsets":[0,0,0,0,0,0,0,0,0,0],"punch_card":[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29],"initial_state":{"rotor_positions":[0,1,2,3,4,5,6,7,8,9]}}
```

```sh
cube cipher fialka encode --parameters fialka.json --text '[5,18,25,5,5,18,25,5,5,18,25,5,5,18,25,5]'
```

These contacts represent `TESTTESTTESTTEST`. The result corresponds to
`PH7FR2EMWW8B8XRA` in the same layout. The starting raw offsets are explicitly
0 through 9, rather than ten zeroes. This independent Chernov simulator check is
preserved by [CrypTool's Fialka tests](https://github.com/CrypToolProject/CrypTool-2/blob/4c8f2bb2794e2263a0fd11bf1425f4badd7fef45/UnitTests/FialkaTest.cs).

## Privacy and limits

Contacts, settings and outputs are private. Native Play requires explicit private
use with named inputs. Core permits 1,048,576 unpacked contacts, settings 512 KiB,
optional traces 65,536 contacts, and the complete serialized report 4 MiB. The
independent report ceiling can be reached first. Exhaustion fails without partial
output. SDK document and native worker transport limits remain independent;
continue smaller blocks with the returned state when necessary.
