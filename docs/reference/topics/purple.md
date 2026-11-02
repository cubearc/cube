# PURPLE Type B cipher

Encipher or decipher prepared A–Z contacts with PURPLE's sixes switch and three
twenties switches. Supply the complete daily alphabet, speed assignment and all
four switch positions. The qualified `purple_type_b_v1` profile fixes the
published switch contact tables. This historical cipher provides no modern
confidentiality or authentication.

`cipher.purple.encode` and `cipher.purple.decode` accept named `text` and JSON
`settings` inputs, with no operation parameters. Results contain `text`,
`final_state` and `trace`. Empty text preserves state.

## Set the switches

| Setting | Required value |
| --- | --- |
| `profile` | `purple_type_b_v1` |
| `alphabet` | Each uppercase A–Z letter once; sixes first, twenties last |
| `fast_switch` | Physical twenties switch 1, 2 or 3 |
| `middle_switch` | A different physical twenties switch 1, 2 or 3 |
| `initial_state.switch_positions` | Four integers 0–24: sixes, twenties 1, 2, 3 |
| `trace` | Optional boolean; defaults to `false` |

The remaining twenties switch is slow. Positions are **zero-based**: subtract
one from each position in a historical key sheet. The plugboard is one complete
alphabet used at both ends, not two independently chosen permutations.

At the current positions, a sixes input crosses the sixes switch; a twenties
input crosses switches 1→2→3. Deciphering reverses both contact mappings and
order. The groups never mix. No garble marker, spaces, case conversion or
message-key derivation is implicit.

## Interpret advancement and state

All motion uses the old sixes and middle positions. After each symbol, sixes
advances once modulo 25, and exactly one twenties switch advances:

| Old state | Twenties switch that advances |
| --- | --- |
| Sixes 23 and middle 24 | Slow |
| Sixes 24 | Middle |
| Otherwise | Fast |

The returned trace contains the post-symbol `state` and physical
`advanced_switch` number. Use `final_state` as a subsequent block's
`initial_state`; enciphering and deciphering produce identical motion.

## Run a historical message prefix

Save `purple.json`. The source key-sheet notation `9-1,24,6-23` becomes:

```json
{"profile":"purple_type_b_v1","alphabet":"NOKTYUXEQLHBRMPDICJASVWGZF","fast_switch":2,"middle_switch":3,"initial_state":{"switch_positions":[8,0,23,5]}}
```

```sh
cube cipher purple encode --parameters purple.json --text FOVTATAKIDASINIMUIMINOMOXIWOIRUBESIFYXXFCKZZR
```

The result is `ZTXODNWKCCMAVNZXYWEETUQTCIMNVEUVIWBLUAXRRTLVA`, with final
positions `[3,0,16,7]`. This is the first 45 letters of the fourteen-part message,
before its first garble. The reference contact tables and source message are
retained by [Brian Neal's PURPLE implementation](https://github.com/gremmie/purple/tree/0647dfe89719bf8ab1c0badbb99936d04eaa4499),
based on Freeman, Sullivan and Weierud's *PURPLE Revealed* (2003).

## Privacy and limits

Text, settings and outputs are private. Native Play requires explicit private
use with named inputs. Text permits 1 MiB, settings 512 KiB, optional traces
65,536 symbols, and the complete serialized report 4 MiB. The independent report
ceiling can be reached first. Exhaustion fails without partial output. SDK and
native worker transport limits remain independent; continue smaller blocks with
the returned state when necessary.
