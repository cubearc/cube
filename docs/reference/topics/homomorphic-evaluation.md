# Homomorphic ciphertext evaluation

Add two encrypted values or negate an encrypted Boolean using fixed ciphertext
profiles. Prepare the ciphertext with the sender's cryptographic library and
keep its parameters and key identity with the data. The operations return
ciphertext bytes for the same context and key.

## Formats and operations

Each required `profile` fits in a JSON settings object of at most 256 bytes.
The format starts with a four-byte ASCII tag followed by fixed-width little-endian
words. For SEAL, export `Ciphertext::data()` in native array order and retain
the source library context when importing the result. The profile covers two
ciphertext components and one coefficient modulus. Use the same key for both
operands and match the scale, level and basis in the table.

| Operation | Profile | Raw format | Bytes per ciphertext |
| --- | --- | --- | ---: |
| `cipher.bfv.add` | `bfv_seal412_degree1024_t17_raw_v1` | `CBF1`, 2,048 `u64` residues, coefficient basis | 16,388 |
| `cipher.bgv.add` | `bgv_seal412_degree1024_t17_ntt_raw_v1` | `CBG1`, 2,048 `u64` residues, NTT basis, correction factor 1 | 16,388 |
| `cipher.ckks.add` | `ckks_seal412_degree2048_scale32_ntt_raw_v1` | `CCK1`, 4,096 `u64` residues, NTT basis, scale `2^32` | 32,772 |
| `cipher.tfhe.not` | `tfhe_rs182_boolean_default_lwe805_raw_v1` | `CTF1`, 806 `u32` Torus32 words from an encrypted Boolean | 3,228 |

The SEAL 4.1.2 BFV and BGV profiles use plaintext modulus 17 and coefficient
modulus 132,120,577. Their decrypted sums are exact modulo 17. CKKS uses
coefficient modulus 18,014,398,492,704,769 and returns an approximate sum in
the original scale. TFHE uses the small LWE ciphertext of Boolean
`DEFAULT_PARAMETERS` in TFHE-rs 1.8.2. Negation preserves the magnitude of the
noise and takes no bootstrapping key.

## Run one evaluation

Save an explicit profile in `settings.json`, then pass the matching raw files.

```sh
cube cipher bfv add --parameters-file settings.json --left a.bin --right b.bin > sum.bin
```

Use `bgv add` or `ckks add` with their matching profiles. For a Boolean use
`cipher tfhe not --parameters-file settings.json --file ciphertext.bin`.
The CLI writes only the raw output bytes. Three Python functions `bfv_add`,
`bgv_add` and `ckks_add` take `(left, right, parameters_json)`.
`tfhe_not` takes `(ciphertext, parameters_json)`. JavaScript uses named
byte ports with one `ciphertext` output.

Each invocation processes one gate or two input ciphertexts, with a 256 KiB
direct-executor reservation. Wrong sizes, tags and noncanonical residues fail.
Tags check container compatibility. The caller remains responsible for matching
keys and preparing ciphertexts within the source library's noise budget.

## Paillier and Damgård–Jurik ciphertext sums

The public operations `cipher.paillier.add` and `cipher.damgard_jurik.add`
accept `left`, `right` and `public_modulus` byte ports. Use an externally
validated shared public modulus `n` and generator `g = n + 1`. Encode `n`
as a full-width unsigned big-endian integer of 256, 384 or 512 bytes with the
most significant bit set and the least significant bit odd.

| Profile | Plaintext space | Ciphertext width |
| --- | --- | --- |
| `paillier_g_n_plus_1_be_v1` | `Z/nZ` | Twice the public modulus width |
| `damgard_jurik_s2_g_n_plus_1_be_v1` | `Z/n²Z` | Three times the public modulus width |

The inputs must use the same fixed-width canonical ciphertext encoding.
Cube checks for a positive residue below the ciphertext modulus and coprime
to `n`. The output is a fixed-width ciphertext for the sum in the
plaintext space. Generate keys, encrypt and decrypt in the originating
library. The evaluation is deliberately malleable and provides no
authentication.

```sh
cube cipher paillier add --parameters-file settings.json --modulus n.bin \
  --left a.bin --right b.bin > sum.bin
```

Python exposes `paillier_add` and `damgard_jurik_add` with parameters
`(left, right, public_modulus, parameters_json)`.
