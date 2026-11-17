# Historical public-key reference profiles

Rabin, Goldwasser–Micali, Benaloh and Merkle–Hellman have executable educational profiles in Core, the CLI, native Python, JavaScript/TypeScript/WebAssembly and private Play graphs. They operate on exact integer or bit tokens. They are unsuitable for protecting live secrets: arithmetic is variable-time, private bigint heaps are not guaranteed erased, and these profiles provide no authenticated byte-message envelope.

## Exact contracts

Each family has `cipher.<family>.encrypt` and `cipher.<family>.decrypt`. Required named inputs are `tokens`, a JSON array of strings, and `key`, bytes containing a UTF-8 JSON document. Integer strings use canonical ASCII decimal notation: `"0"` or a nonzero digit followed by digits. Numbers, signs, leading decimal zeroes, floats and inferred text encodings are refused. Empty token arrays return empty arrays after key validation.

| Family | Required `profile` | Public key JSON | Private key JSON | Tokens |
| --- | --- | --- | --- | --- |
| `rabin` | `rabin_blum_reference_v1` | `{"n":"77"}` | `{"p":"7","q":"11"}` | Plaintext/ciphertext integers in 0..n−1 |
| `goldwasser_micali` | `goldwasser_micali_reference_v1` | `{"n":"272953","y":"2"}` | `{"p":"499","q":"547","y":"2"}` | Plaintext `"0"`/`"1"`; unit ciphertext residues |
| `benaloh` | `benaloh_prime_r_reference_v1` | `{"n":"91","y":"2","r":3}` | `{"p":"7","q":"13","y":"2","r":3}` | Plaintext integers in 0..r−1; unit ciphertext residues |
| `merkle_hellman` | `merkle_hellman_permuted_reference_v1` | `{"weights":["82","22"]}` | `{"weights":["2","3"],"modulus":"101","multiplier":"41","permutation":[0,1]}` | Plaintext binary strings of exactly the weight count; decimal subset-sum ciphertexts |

The table contains tiny synthetic teaching keys, not generated security parameters. Unknown settings and key fields are rejected. All results use the named `tokens` port except Rabin decryption, which returns `candidates`: one numerically sorted, deduplicated array of all roots per ciphertext. Rabin never selects a plaintext on the caller's behalf.

## Construction and validation

Rabin encryption squares modulo n. Private factors must be distinct probable primes congruent to 3 modulo 4; decryption checks roots and returns all CRT combinations, including the smaller root sets for zero or nonunits. Nonresidues fail the entire call. Raw Rabin has ambiguity and no secure padding or chosen-ciphertext protection.

Goldwasser–Micali encrypts each bit as u²·y^bit modulo n, using a fresh uniformly sampled unit u. Public validation checks an odd modulus and a nontrivial unit y with Jacobi symbol +1. Public data cannot prove that y is a quadratic nonresidue; proper key generation must establish that condition. Private validation checks both factors and requires Legendre symbol −1 modulo each. Decryption accepts only unit residues with matching Legendre symbols. There is one modulus-sized residue per bit. The scheme is malleable and has no integrity check.

Benaloh encrypts m as y^m·u^r modulo n with fresh unit u. This profile requires odd prime r≤4093, r dividing p−1, gcd((p−1)/r,r)=1 and gcd(q−1,r)=1. Private validation requires x=y^(φ(n)/r) to have exact order r. Decryption uses a complete finite table of those r powers; it never starts an unbounded discrete-log search. These restrictions address the order ambiguity in earlier descriptions. Ciphertexts remain homomorphic and unauthenticated.

Merkle–Hellman is a broken historical construction. Private weights must be positive and superincreasing, the modulus must exceed their sum, and the multiplier must be an invertible integer in 2..modulus−1. The explicit zero-based permutation must be a full bijection. Public weight i is multiplier·private_weight[permutation[i]] modulo modulus. Greedy private decoding must leave zero residue and reproduce the exact public ciphertext on re-encryption. There is no generic subset-sum search. Binary leading zeroes and weight order are significant.

Factor validation uses 64 independently sampled uniform Miller–Rabin bases through `crypto-primes` 0.7.2. This is a bounded probabilistic check, not a primality certificate. Sampling is fallible; entropy failure produces no partial result. GM/Benaloh encryption is randomized; Rabin and Merkle–Hellman transforms are deterministic. No production seed or coin override is exposed.

## Interfaces

The CLI reads a token array from a file and writes the complete compact result array followed by a newline:

```sh
cube cipher rabin decrypt --parameters-file settings.json --key-file private.json --file ciphertext.json
cube cipher goldwasser-micali encrypt --parameters-file settings.json --key-file public.json --file bits.json
cube cipher benaloh decrypt --parameters-file settings.json --key-file private.json --file ciphertext.json
cube cipher merkle-hellman encrypt --parameters-file settings.json --key-file public.json --file blocks.json
```

Native Python exposes `<family>_encrypt(data, key, parameters_json)` and `<family>_decrypt(data, key, parameters_json)`, with `list[str]` input and `bytes` keys. Results are `list[str]`, except `rabin_decrypt` returning `list[list[str]]`. For example:

```python
import cube
roots = cube.rabin_decrypt(["15"], b'{"p":"7","q":"11"}',
                          '{"profile":"rabin_blum_reference_v1"}')
assert roots == [["13", "20", "57", "64"]]
```

JavaScript/TypeScript uses `sdk.execute` with the exact operation ID, version 1, named `tokens`/`key` inputs and typed parameters. It calls the same Core implementation through WebAssembly. Play uses structured JSON arrays and separate key bytes, requires explicit private retention and offers no text pipeline adapter. Engine API 1 and existing adapter versions are unchanged.

## Combined limits and errors

Values are limited to 4096 bits, token strings to 1234 UTF-8 bytes, calls to 4096 tokens, keys to 65536 bytes and settings to 1024 bytes. Input token transport and complete output reports each have a 4 MiB ceiling. Merkle–Hellman accepts 2..256 weights with total subset sums fitting 4096 bits.

Optional `max_work` defaults to 4194304 and accepts 1..4194304 weighted integer steps, combining key validation, sampling, lookup construction and message processing. Optional `max_memory_bytes` defaults to 67108864 and accepts 1..67108864. The memory bound is a conservative reservation for owned key, token, output, bigint scratch and lookup data; caller/interpreter/process overhead is excluded. Both controls combine; hosts can impose smaller limits. Each rejection loop has at most 128 attempts and each validation/encryption phase permits at most 16 MiB of entropy requests. Exhaustion returns `resource_limit` without partial results.

Invalid profiles or keys return `invalid_parameter`; invalid tokens, root conditions or ciphertexts return `invalid_input`. Errors do not echo key or token values. Some owned transport/string buffers are wiped, but this does not establish bigint or process-wide erasure. No production-security, timing-audit, FIPS or other certification claim is made.

The [backend qualification](../../contributing/asymmetric-backends.md) records dependency and independent-vector evidence. Full named schemas and per-operation privacy/resource policies are in the [operation catalog](../../../contracts/v1/catalog.json).
