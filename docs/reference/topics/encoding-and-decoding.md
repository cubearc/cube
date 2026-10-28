# Encoding and decoding

Use encodings to represent bytes as text, or decoding to recover the original
bytes. Encoding is reversible and provides no confidentiality. Base32, Bitcoin
Base58, standard Base64 and hex are available through the `encoding` category.

```sh
cube encode base32 --text 'foo'
# MZXW6===
cube decode base64 --text 'AP8='
# 00ff
```

Decoders reject whitespace rather than trim it. Base32 requires uppercase and
canonical padding; Base64 requires the standard alphabet, padding and canonical
unused bits. Base58 preserves leading zero bytes as `1`, is case-sensitive and
does not add/verify Base58Check. Hex accepts either letter case but no prefix.
Use `encoding.base64.decode` to inspect the maintained descriptor. Decoding outputs
bytes; explicitly select raw output when passing them to another operation.

## Versioned Base58Check

`encoding.base58check.encode` accepts one explicit version byte and 0..1024 payload
bytes, appends the first four raw bytes of SHA-256(SHA-256(version || payload)),
and reuses Bitcoin-alphabet Base58. Empty payloads are valid; leading zeros remain
exact. The checksum detects corruption and does not authenticate a sender.
`encoding.base58check.decode` strictly verifies at most 1421 encoded bytes and
returns the exact version and payload. It rejects whitespace, absent versions,
frames below five/above 1029 bytes and bad checksums. Working decoded storage is
bounded by input length before conversion; conversion work is quadratic but capped.
These generic codecs infer no network or wallet structure and leave plain Base58
behavior unchanged. Rules/vectors follow [Bitcoin Core](https://github.com/bitcoin/bitcoin/blob/master/src/base58.cpp).

Rust returns `Decoded { version, payload, checksum_verified }`; native Python adds
raw `payload` bytes, and CLI JSON uses `version`, exact `payload_hex` and
`checksum_verified`. Encoding has a version-1 bytes-to-text adapter with an explicit
`version` data parameter, conservative output bounds and quadratic work charging.
Decoding's structured result has no adapter; no new pipeline value is introduced.
Generic codec output/capture contains caller-supplied data: this does not establish
safe automatic capture of private-key formats.
