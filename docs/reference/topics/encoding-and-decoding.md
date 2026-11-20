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

## More byte encodings

| CLI format | Canonical wire representation | Byte / text limit |
| --- | --- | --- |
| `binary` | Eight binary digits per byte with one ASCII space between tokens | 1 MiB / 9,437,183 bytes |
| `octal` | Three octal digits per byte, `000` through `377`, with one ASCII space between tokens | 1 MiB / 4,194,303 bytes |
| `base36` | `0–9a–z`, unsigned big-endian magnitude | 4,096 / 8,192 bytes |
| `base62` | `0–9A–Za–z`, unsigned big-endian magnitude | 4,096 / 8,192 bytes |
| `ascii85` | Adobe digit order, `<~` and `~>` framing, `z` for four zero bytes | 1 MiB / 1,310,724 bytes |
| `ascii85-bare` | Same Adobe digit order and packing, with no framing | 1 MiB / 1,310,720 bytes |
| `base91` | Original basE91 alphabet and 13/14-bit little-endian packing | 1 MiB / 2 MiB |
| `base92` | Original Base92 alphabet and 13-bit big-endian packing | 1 MiB / 2 MiB |
| `url-component` | RFC 3986 unreserved bytes and uppercase percent triplets | 1 MiB / 3 MiB |

Each initial zero byte in Base36 and Base62 becomes one literal `0`. These
profiles use GNU MP digit order and Cube's exact leading-zero convention. The
4,096-byte cap also limits magnitude conversion work. The
[integer base converter](numeric-conversions.md) handles signed numbers.

Ascii85 uses `!` through `u` for digit values 0 through 84. The last partial
group has two to four digits. Groups with overflow, noncanonical tails or
misplaced `z` fail. Empty framed input is `<~~>`. Empty input in Base92 is
`~`. Empty input in the other profiles is empty text.

Base91 uses the following alphabet in order.

~~~text
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&()*+,./:;<=>?@[]^_`{|}~"
~~~

Base92 uses `!`, ASCII `#` through `_`, then ASCII `a` through `}` in order.
Both decoders reject characters outside the profile and require canonical
tail bits. Compare the [basE91 implementation documentation](https://docs.rs/base91/0.1.0/base91/)
and [the original Base92 specification](https://github.com/thenoviceoof/base92).

The URL component decoder accepts lowercase hex in percent triplets. Encode a
literal plus as `%2B`. The decoder returns bytes, including NUL or invalid UTF-8,
so text conversion remains an explicit operation. Percent escapes apply to one
component. The operation interprets no URL structure or form fields.

```sh
cube encode octal --text A
# 101
cube decode url-component --text '%00%ff'
# 00ff
```

## Extended-hex Base32 and URL-safe Base64

`encoding.base32hex.encode` and `encoding.base32hex.decode` use the
RFC 4648 §7 alphabet `0–9 A–V` with canonical padding. The decoder requires
uppercase letters, zero unused bits and padding in the final group only.

`encoding.base64url.encode` and `encoding.base64url.decode` use the
RFC 4648 §5 alphabet with `-` and `_`. The default `required` policy
uses canonical padding. Set `padding` to `forbidden` for profiles that omit
it. A decoder enforces the selected policy and rejects whitespace, unused
bits set to one, and standard Base64 alphabet characters `+` and `/`.

Both profiles limit bytes to 1 MiB and encoded text to 2 MiB. The native
Python entry points are `base32hex_encode`, `base32hex_decode`,
`base64url_encode`, and `base64url_decode`. All four also have version-1
bytes/text pipeline adapters.

```sh
cube binary format base64url-encode --text '{"inputs":{"input":{"base64":"+/8="}},"parameters":{"padding":"forbidden"}}'
# {"output":"-_8"}
```

The standard Base32 and Base64 operations retain their alphabets and padding
rules. The wire profiles are defined in [RFC 4648 §§5, 7, and 10][rfc4648].

[rfc4648]: https://www.rfc-editor.org/rfc/rfc4648.html
