# ML-KEM key encapsulation

ML-KEM lets two parties derive the same 32-byte secret. The recipient generates
a key pair and shares the public key. The sender encapsulates a new secret to
that key and sends the resulting ciphertext. The recipient decapsulates it with
the expanded private key. Use a separate protocol to authenticate the parties
and encrypt messages.

## Profiles and byte lengths

Pass one required `profile` in a JSON object of at most 256 bytes. The three
profiles use FIPS 203 raw bytes, including the expanded private key in section
7.1. Key files contain the raw bytes without Base64, PEM or ASN.1 framing.

| Profile | Public key | Expanded private key | Ciphertext |
| --- | ---: | ---: | ---: |
| `ml_kem_512_fips203` | 800 | 1,632 | 768 |
| `ml_kem_768_fips203` | 1,184 | 2,400 | 1,088 |
| `ml_kem_1024_fips203` | 1,568 | 3,168 | 1,568 |

| Operation | Input ports | Output ports |
| --- | --- | --- |
| `crypto.ml_kem.keygen` | None | `public_key`, `private_key` |
| `crypto.ml_kem.encapsulate` | `public_key` | `ciphertext`, `shared_secret` |
| `crypto.ml_kem.decapsulate` | `ciphertext`, `private_key` | `shared_secret` |

Generation and encapsulation use 64 and 32 fresh bytes of system entropy,
respectively. Decapsulation is deterministic. Incorrect lengths, malformed
encapsulation keys and inconsistent expanded-key hashes fail before returning a
result. For correct-width ciphertexts and a valid key, implicit rejection
returns 32 bytes even when the ciphertext fails the internal comparison. The
result has no validity flag. The profile must match at both ends.

Each call processes one key pair or ciphertext. The direct executor reserves one
work unit and 1 MiB for the portable call, with a separate host process limit.
Play requires explicit consent to retain private results. Python byte strings,
JavaScript buffers, CLI output and backend temporaries can retain copies. Keep
private keys and shared secrets out of logs and shared files.

## Use the native Python SDK

```python
from cube import ml_kem_decapsulate, ml_kem_encapsulate, ml_kem_keygen

parameters = '{"profile":"ml_kem_768_fips203"}'
public_key, private_key = ml_kem_keygen(parameters)
ciphertext, sender_secret = ml_kem_encapsulate(public_key, parameters)
recipient_secret = ml_kem_decapsulate(ciphertext, private_key, parameters)
assert recipient_secret == sender_secret
```

The CLI uses `cube cipher ml-kem keygen`, `encapsulate` or `decapsulate` with
`--parameters-file settings.json`. Encapsulation and decapsulation read the
appropriate raw `--key-file`. Decapsulation reads ciphertext from `--file` or
stdin. CLI output is JSON with the named output ports, each containing a
`base64` string. The JavaScript SDK uses the named input and output ports above
with `Uint8Array` values.

See [FIPS 203](https://csrc.nist.gov/pubs/fips/203/final) and the
[backend qualification](../../contributing/asymmetric-backends.md#ml-kem-qualification).

## Recover a Kyber Round 3 shared secret

`crypto.kyber.decapsulate` reads raw SHAKE ciphertext and expanded private-key
bytes from a CRYSTALS-Kyber 3.0 exchange. Match the profile recorded by the
sender. The byte widths match the corresponding ML-KEM sizes in the table, so
the encoding cannot identify which algorithm produced the ciphertext.

| Profile | Private key | Ciphertext |
| --- | ---: | ---: |
| `kyber512_round3_shake` | 1,632 | 768 |
| `kyber768_round3_shake` | 2,400 | 1,088 |
| `kyber1024_round3_shake` | 3,168 | 1,568 |

Use `cube cipher kyber decapsulate --parameters-file settings.json --key-file
legacy.bin --file ciphertext.bin` or `kyber_decapsulate(ciphertext, private_key,
parameters_json)` in Python. JavaScript uses the `ciphertext` and `private_key`
byte ports and returns `shared_secret`. The 32-byte output uses historical
implicit rejection for a correct-width ciphertext. Wrong lengths, an invalid
expanded-key hash or an unknown profile return an error. Play retention and
resource bounds match the raw ML-KEM decapsulation interface.
