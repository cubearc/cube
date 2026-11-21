# Binary transformations

Apply bitwise masks, reorder bytes, or calculate with fixed-width unsigned integers.
These operations return bytes and do not interpret text. For a walkthrough, see
[Transform bytes](../../guides/binary.md).

## Bitwise operations

Choose from these bitwise operations:

| Operation | CLI command |
| --- | --- |
| AND | `binary and` |
| OR | `binary or` |
| Exact XOR | `binary xor-exact` |
| Repeating-key XOR | `binary xor` |
| NOT | `binary not` |

- AND, OR and exact XOR combine equal-length operands, including two empty inputs.
  They do not repeat or pad an operand.
- Repeating-key XOR cycles a nonempty key across the input. An empty key fails
  even when the input is empty.
- NOT flips every bit of its single input.

## Byte order

The two reorder operations differ in scope:

| Operation | CLI command | Result for `41 42 43 44` |
| --- | --- | --- |
| Reverse the buffer | `binary reverse` | `44 43 42 41` |
| Reverse bits within each two-byte unit | `binary reverse-bits --unit-bytes 2` | `42 82 22 c2` |
| Swap two-byte units | `binary swap-endian --unit-bytes 2` | `42 41 44 43` |

Both operations restore the input when applied twice. Swapping byte order keeps
the units in their original order.

## Fixed-width arithmetic

Shifts, rotations and wrapping arithmetic treat each unit as an independent,
unsigned big-endian integer. No carry or borrow crosses a unit boundary.

| Setting | Default | Accepted values |
| --- | --- | --- |
| `--unit-bytes` | `1` | `1`, `2`, `4` or `8` bytes |
| `--bits` for shifts | Required | `0` through the unit's bit width |
| `--count` for rotations | Required | Unsigned 64-bit integer; reduced modulo the bit width |
| `--key-hex` for addition/subtraction | Required | An equally long operand containing complete units |

The operations handle overflow as follows:

| Operation | CLI commands | Behavior |
| --- | --- | --- |
| Logical shift | `shift-left`, `shift-right` | Fill with zeros and discard bits outside the unit. |
| Rotation | `rotate-left`, `rotate-right` | Wrap shifted bits around within the unit. |
| Wrapping arithmetic | `add-wrapping`, `sub-wrapping` | Calculate modulo `2^(unit_bytes × 8)`, including 64-bit overflow and underflow. |

A shift of zero leaves the input unchanged. A shift equal to the unit's bit width
produces zeros; a larger shift fails. Partial units fail without padding.
Unit widths and operand lengths are validated even for empty input.

Bit reversal requires an explicit unit width of 1, 2, 4 or 8 bytes. It
reverses bits independently within each complete big-endian unit. Applying
the same command twice restores the bytes.

## Compression formats

Use `binary compress` and `binary decompress` with an explicit `--profile`.
Selected profiles read one complete stream. Trailing data, concatenated
streams and external dictionaries fail.

| Profile | Wire format | Encoder |
| --- | --- | --- |
| `deflate` | Raw RFC 1951 stream | Levels 0–9, default 6 |
| `zlib` | RFC 1950 stream with Adler-32 | Levels 0–9, default 6 |
| `gzip` | RFC 1952 single member with CRC-32 and length | Levels 0–9, default 6, mtime 0, no filename |
| `brotli` | RFC 7932 standard window | Quality 4, window 1 MiB, block setting 64 KiB |
| `zstd` | RFC 8878 standard frame | Fixed fastest setting with a content checksum |
| `lzma_alone` | LZMA1 in a .lzma header, with end marker when size is unknown | Fixed fast preset with 1 MiB dictionary |

Input and output are capped at 16 MiB. Decoder windows and dictionaries
are bounded at 16 MiB. Decoding stops at the smaller of 16 MiB and 256 times
the encoded byte count. Encoding also checks this ratio so its returned
stream can be read under the same rules. Declared lengths are checked when
the format carries them. Any included checksum detects corruption. It
provides no authentication.

```sh
cube binary compress --profile zlib --text abc --output raw > abc.zlib
cube binary decompress --profile zlib --file abc.zlib
# 616263
```

## Limits

Limits depend on the operation and execution interface:

| Scope | Limit |
| --- | --- |
| Bitwise, byte-order and arithmetic operations, except repeating-key XOR | 16 MiB per operand |
| Repeating-key XOR through direct Rust/Python calls | No fixed input or allocation budget; callers control buffer sizes |
| Binary operand parameter in a pipeline | 64 KiB |

Pipelines validate types, parameters and budgets before execution. Operand lengths
that depend on an earlier result are checked when that step runs. The shared
[pipeline limits](compose-operations.md#limits) also apply.

## Repeating-key XOR example

Use `--key` for UTF-8 key text or `--key-hex` for arbitrary key bytes.
Provide exactly one:

```sh
cube binary xor --text 'Hello' --key-hex 6b6579
# 230015070a
```

Applying the same key twice restores the original bytes. Repeating-key XOR is
useful for reversible masks, but it does not provide secure encryption.

Use `--output raw` when passing binary output to another operation.
For the complete schemas, see the [operation reference](../operations.md).

## Bit packing and counts

Use `binary.pack_bits` to pack bytes whose values are exactly 0 or 1. Set
`bit_order` to `msb` or `lsb` to choose the first position within each byte.
The default `reject` tail policy requires complete groups of eight. Set
`tail` to `pad_zero` to fill an incomplete final byte with zeros.
`binary.unpack_bits` uses the supplied `valid_bits` and returns that many bytes
valued 0 or 1. Both operations return the valid bit count. Each accepts up to
1 MiB of input and unpacking can produce up to 8 MiB.

`binary.mask` repeats a single unsigned AND mask over complete big-endian
units. `binary.count_bits` reports `population`, `leading_zeros`, or
`trailing_zeros` in input order. The default unit is one byte. Other choices
are 2, 4 and 8 bytes. A zero unit has as many leading and trailing zeros as bits
in the unit. The count report holds up to 65,536 units and the byte input for
either operation is limited to 16 MiB.

## Exact numbers and decimal digits

| Family | Required format | Limits and canonical behavior |
| --- | --- | --- |
| `binary.fixed_integer.*` | 8, 16, 32 or 64 bits, `big` or `little` byte order, `signed` or `unsigned` | Signed values use two's complement. Narrowing and partial units fail. Up to 65,536 values and 1 MiB each for input and output. |
| `binary.bcd.*` | `unsigned_packed`, `unsigned_unpacked`, or `signed_packed`, with exact digit width | Up to 4,096 ASCII digits. Packed values take a leading zero nibble when needed. Unpacked values use a zero high nibble. Signed values use C for positive and D for negative. Decode accepts sign F only with `allow_unsigned_sign`. Zero has a positive sign and all declared digits. |
| `binary.varint.*` | `uleb128`, `sleb128`, `protobuf_uint64`, or `zigzag64` | Each uses at most ten bytes. Decode requires `prefix`, `full` or `batch` and reports consumption per integer. Full mode requires one complete value and rejects trailing bytes. The default requires shortest form. `allow_noncanonical` admits redundant groups within the 64-bit domain. Up to 65,536 values and 1 MiB of bytes or reports. |
| `binary.ieee754.inspect` | `binary32` or `binary64`, and byte order | Returns raw bits, sign, biased exponent, fraction and classification. The report preserves signed zero and quiet and signaling NaN payloads. Up to 65,536 complete records and 4 MiB of reports. |

Wide integers use decimal strings in JSON when their magnitude exceeds
9,007,199,254,740,991. The Python SDK accepts and returns builtin integers.
The Rust SDK owns the exact functions in `binary::representation`,
`binary::bcd`, and `binary::varint`. The JavaScript SDK generates named
types from the same contracts and accepts `Uint8Array` for direct byte ports.

```sh
cube binary format varint-decode --text '{"inputs":{"data":{"base64":"5Y4m"}},"parameters":{"profile":"uleb128","mode":"batch"}}'
# {"records":[{"consumed":3,"value":624485}]}
```

```python
import cube

options = {"bits": 64, "byte_order": "little", "signedness": "unsigned"}
data = cube.fixed_integer_encode([42, (1 << 64) - 1], options)
assert cube.fixed_integer_decode(data, options)["values"] == [42, (1 << 64) - 1]
```

BCD uses width-preserving digit text. A decimal scale belongs to the enclosing
format. Floating-point inspection uses these [IEEE field definitions][ieee].
The varint profiles follow [DWARF 5 §7.6][dwarf] and the
[Protocol Buffers encoding guide][protobuf]. BCD sign choices follow the
[signed packed-decimal layout][bcd].

[ieee]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_bits
[dwarf]: https://dwarfstd.org/doc/DWARF5.pdf
[protobuf]: https://protobuf.dev/programming-guides/encoding/
[bcd]: https://www.ibm.com/docs/en/cobol-zos/6.4.0?topic=arithmetic-formats-numeric-data

## Reed–Solomon block correction

`binary.reed_solomon.encode` appends parity to shortened messages.
`binary.reed_solomon.decode` receives complete codewords with explicit
zero-based erasure positions. It returns the corrected message and codeword,
changed positions, and initial and verified syndromes in root order. Corrections
must satisfy `2 × errors + erasures ≤ parity_bytes`. A valid alternate codeword
can also have zero syndromes. The result establishes code validity.

Select `field.profile` explicitly. The `qr-rs-gf256` profile uses
polynomial `0x11d`, generator 2, first root 0, consecutive roots, and
systematic message followed by parity. The `custom` profile requires a monic
degree-eight `polynomial` with constant term 1, a primitive `generator` and a
`first_root` exponent from 0 through 254. Use leading-zero shortening without
interleaving. The message must contain at least one byte. The total codeword
may have up to 255 bytes, including 2 to 64 parity bytes.

A batch can contain up to 4,096 blocks and 1 MiB of input bytes. The hard
ceilings are 16 Mi field operations, 16 MiB of scratch space and 4 MiB of
reports. Callers may lower `work_limit`. An uncorrectable block fails the
whole batch. The CLI can write one verified codeword as exact bytes using
`--codeword-file` with a new destination.

```python
import cube

settings = {"field": {"profile": "qr-rs-gf256"}, "parity_bytes": 10}
message = bytes.fromhex("40548454c4c4f0ec11ec11ec11ec11ec")
[encoded] = cube.reed_solomon_encode([message], settings)
assert encoded["parity"].hex() == "23732399ec08c9f737df"
damaged = bytearray(encoded["codeword"])
damaged[3] ^= 43
[corrected] = cube.reed_solomon_decode([(bytes(damaged), [3])], settings)
assert corrected["message"] == message and corrected["positions"] == [3]
```

The QR parity fixture is checked against the
[Nayuki reference implementation][nayuki]. Core builds and validates its
tables for each selected field. The field layout, shortening and correction
contracts are exposed through the direct typed APIs.

[nayuki]: https://github.com/nayuki/QR-Code-generator/blob/9a217d443937f60cc56c1458c39a211d7f2d3af5/python/qrcodegen.py

## PackBits, dictionary tokens and block transforms

[TIFF 6.0 PackBits](https://pdfa.org/wp-content/uploads/2020/07/TIFF6.pdf)
uses signed controls for literals, repeated bytes and no-ops. The encoder
prefers repeats of 3 to 128 bytes and leaves pairs in literals. Decoding checks
complete packets and an optional exact decoded length. Both forms accept up to
1 MiB of input. Decoded output is capped at 16 MiB. PackBits has version-1
pipeline adapters for exact byte input and output.

| Transform | What the result carries | Main Core bounds |
| --- | --- | --- |
| LZ77 | Literal or distance/length tokens with a 4,096-byte default window | 64 KiB source, 65,536 tokens, 1 MiB expansion |
| LZ78 | Prior dictionary index and a byte, with an optional final suffix | 64 KiB source, 65,536 tokens, 1 MiB expansion |
| Burrows–Wheeler | Last column and the exact primary index for inverse use | 65,536 bytes |
| Move-to-front | Byte indexes in a moving permutation | 65,536 bytes |

LZ77 chooses the longest match, then the smallest distance. Matches span 3
through 258 bytes and can overlap. LZ78 index zero names the empty phrase.
BWT uses the starting offset to break equal-rotation ties. MTF defaults to the
byte order 00 through FF. Pass the identical initial alphabet to invert a
custom permutation. LZ token sequences and BWT results use direct named ports.

## LZ4, bzip2 and LZW

| Profile | Required selection | Main Core bounds |
| --- | --- | --- |
| LZ4 | Raw block with exact decoded length, or standard frame | 16 MiB input/output, 64 MiB scratch, 10,000 frame blocks |
| bzip2 | Standard BZh stream, level 1 for encoding by default | 16 MiB input/output, 64 MiB scratch, 10,000 cumulative blocks |
| GIF89a LZW | Minimum code size 2 through 8, least-significant-bit first | 1 MiB input, 16 MiB output, 4,096 dictionary entries |
| TIFF 6.0 LZW | 256-byte alphabet, most-significant-bit first and early width changes | 1 MiB input, 16 MiB output, 4,096 dictionary entries |

LZ4 framing verifies the declared size and each selected checksum. The encoder
uses independent 64 KiB blocks, an explicit content size and both checksums.
Callers can lower input, output, scratch and block bounds. Frame decoding
rejects trailing frames, dictionaries and legacy magic. A raw block has no
size or integrity fields.

bzip2 checks each member's standard CRCs. Enable explicit concatenation to
use a single output and block budget across members. The encoder uses standard
levels 1 through 9 and work factor 30.

The LZW profiles operate on compressed code bytes. GIF data subblocks and TIFF
strip framing belong to the enclosing formats. Both profiles emit initial
clear and end codes. GIF keeps a full dictionary frozen until a clear. TIFF
clears before reaching the last two dictionary slots. Decoding accepts at most
seven zero padding bits after the end code.

## Canonical Huffman and fixed probability streams

Canonical Huffman builds binary code lengths from a byte frequency table. The
tree breaks ties by frequency, lowest symbol and node creation order. Codes
follow increasing length and then symbol. A singleton uses one zero bit per
symbol. The encoder returns the table, packed bytes and exact bit count.
Decoding takes code lengths, bit count and symbol count. It checks that the
codes fill a complete tree, with exceptions for singleton and empty streams.

Frequency records have a byte `symbol` and a canonical unsigned decimal
`count`. An omitted symbol has count zero. Decimal strings preserve every
64-bit count across interfaces.

`arithmetic16` uses [Witten–Neal–Cleary](https://doi.org/10.1145/214762.214771)
16-bit inclusive intervals, E1/E2/E3 renormalization and an MSB-first final
interval prefix. `range32` uses a byte-renormalized half-open range with
[delayed carry](https://github.com/ip7z/7zip/blob/25.01/C/LzmaEnc.c) and a
five-byte flush. Both models use ascending byte symbols and fixed frequencies
with total 1 through 16,383. There is no rescaling. Pass the identical model
and the exact original symbol count to decode. Canonical re-encoding checks
termination and trailing data. The raw streams carry no independent checksum
or embedded model.

To decode a PackBits buffer in Python, pass the bytes and expected size.

```python
import cube

plain = cube.packbits_decode(bytes.fromhex("fe5102585859fd5a"), expected_length=10)
assert plain == b"QQQXXYZZZZ"
```
