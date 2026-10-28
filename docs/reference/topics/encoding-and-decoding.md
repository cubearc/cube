# Encoding and decoding

Encoding represents bytes as text; decoding recovers the bytes. These formats
are reversible and provide no confidentiality.

## Supported formats

Decoders require exact input and reject whitespace.

| Format | Encoding | Decoding rules |
| --- | --- | --- |
| Base32 | Uppercase with padding | Uppercase and canonical padding |
| Base58 | Bitcoin alphabet; each leading zero byte becomes `1` | Case-sensitive; no Base58Check checksum |
| Base64 | Standard alphabet with padding | Standard alphabet, padding and canonical unused bits |
| Hex | Hexadecimal byte pairs | Either letter case; no prefix |

For example, encode text as Base32 or decode Base64 to bytes:

```sh
cube encode base32 --text 'foo'
# MZXW6===
cube decode base64 --text 'AP8='
# 00ff
```

Decoded output is bytes; `00ff` is its hexadecimal display. Select raw output
when passing those bytes to another operation. See
[Encode and decode data](../../guides/encoding.md).

## Versioned Base58Check

Base58Check adds a version byte and checksum to a payload. The checksum detects
corruption; it does not authenticate a sender or identify a network or wallet.

### Frame and limits

The frame is `version || payload || checksum`. The checksum is the first four
bytes of `SHA-256(SHA-256(version || payload))`. The complete frame uses the
Bitcoin Base58 alphabet and preserves leading zeros.

| Field or limit | Rule |
| --- | --- |
| Version | One explicit byte |
| Payload | 0–1,024 bytes; empty is valid |
| Decoded frame | 5–1,029 bytes |
| Encoded input | At most 1,421 bytes |
| Validation | Reject whitespace, missing version and checksum mismatch |
| Conversion work | Quadratic in input length, bounded by the input cap |
| Working storage | Bounded by input length before conversion |

The format and test vectors follow
[Bitcoin Core](https://github.com/bitcoin/bitcoin/blob/master/src/base58.cpp).

### Results and pipeline support

Decoding returns the version, exact payload and `checksum_verified`. Rust and
Python expose payload bytes; CLI JSON uses `payload_hex`.

Encoding has a version-1 bytes-to-text adapter with an explicit `version`
parameter. Its output and work bounds are checked before execution. Decoding
returns a structured result and has no pipeline adapter.

Payloads and captures contain the caller's data. A valid checksum does not make
private-key material safe to capture; see [Offline key formats](offline-key-formats.md).
