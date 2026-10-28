# Raw stream ciphers

Transform exact bytes using an explicit key, nonce and initial block counter.
Applying the same profile and state to the result restores the input. These raw
transforms are unauthenticated and malleable; they do not detect a wrong key,
wrong nonce or modified ciphertext. The caller must prevent reuse of a
key/nonce/position combination across messages.

## Profiles

Every stream key is exactly 32 bytes. Nonce and counter are required; neither
has a default. Counters count 64-byte blocks, starting at the supplied index.

| Operation | Rounds | Nonce | Block counter |
| --- | --- | --- | --- |
| `cipher.chacha8` | 8 | 12 bytes, IETF layout | 0–2³²−1 |
| `cipher.chacha12` | 12 | 12 bytes, IETF layout | 0–2³²−1 |
| `cipher.chacha20` | 20 | 12 bytes, RFC 8439 | 0–2³²−1 |
| `cipher.xchacha20` | 20 | 24 bytes, HChaCha20 then IETF layout | 0–2³²−1 |
| `cipher.salsa20_12` | 12 | 8 bytes | 0–2⁶⁴−1 |
| `cipher.salsa20` | 20 | 8 bytes | 0–2⁶⁴−1 |

XChaCha20 follows the RustCrypto layout described in the expired
[XChaCha draft](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03):
derive a subkey from the first 16 nonce bytes and use a zero word followed by
the last eight bytes as the IETF nonce. Its counter is 32 bits. The legacy
64-bit-nonce ChaCha variant, XChaCha reduced rounds, XSalsa and 16-byte Salsa
keys are unsupported. Reduced-round profiles are experimental interoperability
choices and provide a smaller security margin.

## Transform a file

Prepare a raw 32-byte key file through your existing private-key workflow.
Supply a unique nonce for that key and an explicit initial counter. For a local
interoperability task, run:

```sh
cube cipher chacha20 --key-file key.bin --nonce-hex 000000000000000000000001 \
  --counter 0 --input message.bin --output transformed.bin
```

Run the same command with `transformed.bin` as input and another new output path
to restore `message.bin`. Existing output paths are rejected. `--stdout` explicitly
emits raw result bytes instead of creating a file. Omit `--input` to read exact
stdin. Keys and pads are read from regular files; no inline key argument is
accepted. `--lab-capture` is unavailable for these private operations.

Rust uses `ciphers::stream::transform(Profile, input, key, nonce, counter)`.
Native Python uses `cube.stream_transform(profile, data, key, nonce, counter)`;
profile names are the operation suffixes shown above. Both return exact bytes.
Play accepts named `input` and `key` byte ports, with `nonce` as a byte array and
`counter` as a parameter. Wide JSON counters use the canonical decimal-string
transport. Inputs stay private and result retention requires a per-run choice.
The byte viewer supports preview and explicit export. No pipeline or recipe
adapter is enabled.

## Educational Vernam

`cipher.vernam` and `cube.vernam(data, pad)` reuse exact-length XOR. Both operands
must have the same length; empty/empty is identity. CLI uses `cube cipher vernam
--key-file pad.bin --input message.bin --output transformed.bin`, without nonce
or counter options. Existing repeating-key `binary.xor` retains its semantics.

A real one-time pad requires a uniform, independent pad at least as long as the
message, delivered securely and used once. This educational API does not generate,
distribute, store or track pads and cannot guarantee those prerequisites. Pad
reuse compromises confidentiality; exact XOR alone provides no authentication.

## Limits and failures

| Resource or rule | Bound |
| --- | --- |
| Input, output and Vernam pad | 16 MiB each |
| Stream key | Exactly 32 bytes |
| Work | Linear in input length; at most 262,144 blocks |
| Counter exhaustion | Reject before allocation if any requested full or partial block exceeds the profile |
| Empty input | Validate key, nonce and counter, then return empty bytes |

Wrong lengths and out-of-range/exhausted counters return typed errors without
key contents. Backend state and temporary keystream are zeroized. Caller-owned
Rust/Python buffers, returned bytes, upstream stack temporaries, OS caches and
interface copies are outside that guarantee. There is no network access, implicit randomness or nonce storage.

## Backend qualification

The implementation pins RustCrypto `chacha20` 0.10.2 and `salsa20` 0.11.0 with
zeroization enabled (MIT OR Apache-2.0, Rust 1.85 minimum). ChaCha's upstream
[security notes](https://docs.rs/chacha20/0.10.2/chacha20/#security) report a prior
NCC Group audit and a constant-time design; that is not an audit of this exact
Cube integration. Salsa's [security notes](https://docs.rs/salsa20/0.11.0/salsa20/#%EF%B8%8F-security-warning-hazmat)
state that it has no independent audit or thorough constant-time assessment.
It is exposed for offline interoperability without a modern-security guarantee.

A [minimal vendored Salsa counter patch](../../../vendor/salsa20/CUBE-PATCH.md)
changes only the post-block counter increment to wrapping arithmetic, matching
release behavior at the last valid block. Core preflight and one-call state
ownership prevent generation after exhaustion. Recognized vectors, final-block
boundaries and original upstream tests verify the patch. No primitive is
implemented in the CLI, SDK or Play.
