# Alternate KEM profiles

Recover a shared secret from a previously encapsulated key with a named
decapsulation profile. Provide the raw `ciphertext` and raw `private_key` in
separate byte ports. The result has one `shared_secret` byte port.

| Profile | Private key bytes | Ciphertext bytes | Shared secret bytes |
| --- | ---: | ---: | ---: |
| `frodokem640_shake_salted` | 19,888 | 9,752 | 16 |
| `hqc128_20260210_submission` | 2,321 | 4,433 | 32 |
| `mceliece460896f_round4` | 13,608 | 156 | 32 |

The FrodoKEM profile expects the salted SHAKE format. The HQC profile fixes
the February 2026 submission encoding. The Classic McEliece profile uses the
Round 4 ciphertext format. Use the same peer implementation and edition as
the sender. Each settings object requires the exact `profile` and has a
256-byte limit.

## Decapsulate with a private key file

For `crypto.frodo_kem.decapsulate`, write
`{"profile":"frodokem640_shake_salted"}` to `settings.json`, then run:

```sh
cube cipher frodo-kem decapsulate --parameters-file settings.json \
  --key-file private.bin --file ciphertext.bin
```

The CLI encodes `shared_secret.base64` in the result JSON. Substitute `hqc` or
`classic-mceliece` for the corresponding profile and operation. The native
Python functions are `frodo_kem_decapsulate`, `hqc_decapsulate` and
`classic_mceliece_decapsulate`, each taking `(ciphertext, private_key,
parameters_json)`. JavaScript accepts `Uint8Array` for both named byte ports
and returns a `Uint8Array` shared secret.

## Bounds and result handling

Each call decapsulates one ciphertext. Wrong widths or a different profile
fail. Correctly sized ciphertexts go through the selected backend's implicit
rejection. The resulting bytes carry no validity flag. Use a protocol with
explicit authentication and keep the output private.

The direct executor reserves one work unit and 4 MiB per call. The host applies
its independent memory and transport limits. Play requires explicit consent
to retain private results. See the [backend qualification](../../contributing/asymmetric-backends.md#alternate-kem-qualification)
for exact source revisions and vectors.
