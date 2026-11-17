# Preserve exact numeral strings with FPE

Use `cipher.fpe.encrypt` and `cipher.fpe.decrypt` version1 to transform an array
of integer digits while preserving its length and radix, including leading zeros.
Rust exposes `cube_core::ciphers::fpe::{Settings, encrypt, decrypt}`; native Python
exposes `fpe_encrypt(digits, parameters_json)` and `fpe_decrypt`.
JavaScript/TypeScript uses `cube.execute` with numeric `input` and `output` arrays.
All interfaces execute the same Core algorithms.

## Choose an exact profile

| Profile | Tweak | Status |
| --- | --- | --- |
| `ff1_sp800_38g_integer` |0..65536 bytes | SP800-38G FF1 with integer-only length arithmetic |
| `ff3_sp800_38g_historical` |8 bytes | Original SP800-38G historical compatibility |
| `ff3_1_2019_draft_historical` |7 bytes |2019 first revision draft historical compatibility |

All settings are required: `profile`, `radix`, `key_hex`, `tweak_hex`.
Unknown fields are rejected. AES keys are exactly16,24 or32 bytes, represented
as32,48 or64 hexadecimal digits. Tweaks are exact even-length hexadecimal;
an empty FF1 tweak is the explicit empty string. There is no alphabet inference,
text normalization, passphrase derivation or profile fallback.

The radix is2..65536 and every digit is an integer0..radix-1. Input length is
2..4096 digits and the domain `radix^length` must be at least1,000,000.
FF3/FF3-1 additionally require `radix^ceil(length/2) <= 2^96`. These limits are
checked using integers before rounds run. Settings JSON is capped at132096
UTF-8 bytes. The CLI accepts at most24578 bytes of serialized digit-array JSON,
including whitespace. FF1 uses10 rounds; FF3 variants use8. Native Play can impose lower
transport limits. These private operations require the existing explicit private-retention policy in Play.

As checked on2026-11-17, the [NIST2025 second public draft](https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd)
removes FF3 and FF3-1 in response to cryptanalysis and disallows floating-point
FF1 arithmetic. It is a draft, so this does not claim a final withdrawal.
FF3/FF3-1 are exposed only under explicit historical names. None of these profiles
authenticates data, detects a wrong key/tweak or hides the format/domain size.

## Run the NIST decimal example

Create a private `settings.json` file with this synthetic published test key:

```json
{"profile":"ff1_sp800_38g_integer","radix":10,"key_hex":"2b7e151628aed2a6abf7158809cf4f3c","tweak_hex":""}
```

```sh
printf '[0,1,2,3,4,5,6,7,8,9]' | cube cipher fpe encrypt --parameters-file settings.json
```

The output is `[2,4,3,3,4,7,7,4,8,4]`. Decryption returns the original array.
Keep real keys out of shell arguments, logs, recipes and sessions.

```python
import cube
settings = '{"profile":"ff1_sp800_38g_integer","radix":10,"key_hex":"2b7e151628aed2a6abf7158809cf4f3c","tweak_hex":""}'
assert cube.fpe_encrypt(list(range(10)), settings) == [2,4,3,3,4,7,7,4,8,4]
```

## Verification and implementation

Core implements the specification's Feistel constructions over maintained
RustCrypto AES0.9.3 and num-bigint0.5.1. It computes FF1 bit lengths from exact
integer powers. The reviewed fpe0.7.0 implementation uses floating-point length
calculations for arbitrary radices, so it was not adopted for this profile.
Expanded AES keys, decoded keys/tweaks and owned byte/digit scratch buffers are
wiped on drop. Big-integer, caller, JSON parser, OS and interpreter copies are
not guaranteed erased. Big-integer operations are not claimed constant-time.
No independent audit, CAVP/FIPS validation or modern security endorsement is claimed.

The checked corpus records source-file SHA256 and sample identifiers for the
[NIST FF1 examples](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/FF1samples.pdf),
[NIST FF3 examples](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/FF3samples.pdf)
and [Bouncy Castle FF3-1/wide-radix tests](https://github.com/bcgit/bc-java/blob/main/core/src/test/java/org/bouncycastle/crypto/test/SP80038GTest.java).
Known answers execute through Core, CLI, installed native Python, actual Wasm
and private native Play graphs. Round trips alone are not the conformance oracle.
