# Educational lattice encryption

Run one encryption or decryption at a time using synthetic inputs. These small
profiles illustrate the original NTRUEncrypt and Regev PKE constructions.
Their parameters provide no cryptographic security or authentication.

| Profile | Plaintext | Public key | Private key | Ciphertext |
| --- | ---: | ---: | ---: | ---: |
| `ntru_encrypt_educational_n11_p3_q32` | 11 bytes | 11 bytes | 22 bytes | 11 bytes |
| `regev_educational_n4_m16_q97` | 1 byte | 80 bytes | 4 bytes | 5 bytes |

NTRUEncrypt works in the cyclic polynomial ring with `N=11`, `p=3` and
`q=32`. The public key contains the 11 residues of `3 f⁻¹ g` modulo 32
in ascending polynomial degree. The caller uses a ternary `f` with at most
five nonzero coefficients, invertible modulo 3 and 32, and a ternary `g`.
The private key concatenates `f` and its inverse modulo 3. Encode each
ternary coefficient as the byte 0, 1 or 2 (for −1). Encrypting a message
adds a cyclic product with a random polynomial containing one +1 and one −1
at distinct positions. The stated key shape bounds the centered recovery
coefficients in absolute value by 11.

Regev PKE works with dimension 4 and modulus 97. The public key contains
16 consecutive rows, each of four residues followed by the corresponding
dot product with the private vector plus an error in {−1, 0, 1}, reduced
modulo 97. Encrypt a single byte 0 or 1. A fresh random subset of the rows
is summed, with 48 added to the last coordinate for 1. The five ciphertext
coordinates are canonical one-byte residues modulo 97. Decryption measures
circular distance to the two centers and rejects a tie.

The `profile` is required in a JSON object of at most 256 bytes. Raw
ciphertext and plaintext can be read from stdin or a regular file. The CLI
writes only raw output bytes.

```sh
cube cipher regev encrypt --parameters-file settings.json --key-file public.bin \
  --file bit.bin > ciphertext.bin
cube cipher regev decrypt --parameters-file settings.json --key-file private.bin \
  --file ciphertext.bin > recovered.bin
```

Python exposes `regev_encrypt`, `regev_decrypt`, `ntru_encrypt_encrypt`,
and `ntru_encrypt_decrypt`. Pass `(input, key, parameters_json)` using
the matching public or private key. JavaScript uses the `input` byte port
and either `public_key` or `private_key`, returning the `output` byte
port. Play treats the input as private and requires explicit consent for
private retention.
