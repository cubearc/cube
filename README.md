# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source Rust toolkit for encoding, classical ciphers, hashing,
binary transformations, file forensics, and image analysis. Use the Rust library, `cube` CLI, or Rust-backed
Python package. All interfaces share the same implementations.

## Capabilities

| Operation | Behavior |
| --- | --- |
| Base32 | Uppercase, padded RFC 4648 standard encoding and strict decoding |
| Base58 | Case-sensitive Bitcoin alphabet; preserves leading zero bytes |
| Base64 | Padded RFC 4648 standard encoding and strict decoding |
| Hex | Lowercase encoding; decoding accepts either case |
| Caesar | Signed shift of ASCII letters modulo 26; preserves case and non-ASCII characters |
| ROT13 | Rotates ASCII letters; preserves case and non-ASCII characters |
| CRC32 | CRC-32/ISO-HDLC (IEEE), exactly eight lowercase hexadecimal digits |
| SHA-256 | Lowercase hexadecimal digest |
| XOR | Byte-wise XOR with a nonempty repeating key |
| File forensics | Bounded signatures, metadata/EXIF, hex, ASCII strings, binary comparison, PNG trailers, and safe extraction |
| PNG8 images | Exact samples/palettes, chunk CRCs, channels, histograms, pixel exports, transforms, comparisons, bit planes and configurable LSB extraction |

Caesar, ROT13, and repeating-key XOR are classical puzzle tools, not secure encryption.
Encoding decoders reject whitespace. Base32 and Base64 reject invalid padding and
trailing bits; Base32 also rejects lowercase letters.
Base58 maps each leading zero byte to `1` and uses no padding or Base58Check checksum.
CRC32 is a checksum for accidental corruption, not a cryptographic hash.
It uses width 32, polynomial `0x04C11DB7`, reflected input/output, initial value
`0xFFFFFFFF`, and final XOR `0xFFFFFFFF`. Empty input returns `00000000`.

## Build and install

Install stable Rust through [rustup](https://rustup.rs/) and a platform C linker.
Then build from source:

```sh
git clone https://github.com/cubearc/cube.git
cd cube
cargo build --workspace --locked
cargo install --path crates/cube-cli --locked
cube --version
```

For an uninstalled development build, use `cargo run -p cube-cli --` in place of
`cube`. No published registry packages are required.

## CLI quick start

```sh
cube --help
cube ops list
cube encode base32 --text "foo"                   # MZXW6===
cube decode base32 --text "MZXW6==="               # 666f6f
cube decode base32 --text "MZXW6===" --output raw  # foo (no added newline)
cube encode base58 --text "Hello World"           # JxF12TrwUP45BMd
cube decode base58 --text "JxF12TrwUP45BMd" --output raw # Hello World
cube encode base64 --text "Hello"                 # SGVsbG8=
cube decode base64 --text "SGVsbG8="               # 48656c6c6f
cube decode base64 --text "SGVsbG8=" --output raw   # Hello (no added newline)
cube encode hex --text "Hello"                    # 48656c6c6f
cube decode hex --text "48656c6c6f" --output raw   # Hello
cube cipher caesar --shift 3 --text "Attack at dawn!" # Dwwdfn dw gdzq!
cube cipher caesar --decode --shift 3 --text "Dwwdfn dw gdzq!" # Attack at dawn!
cube cipher caesar --shift -1 --text "AaZz"        # ZzYy
cube cipher rot13 --text "Hello"                  # Uryyb
cube hash crc32 --text "123456789"                # cbf43926
cube hash crc32 --text ""                         # 00000000
cube hash sha256 --text "Hello"
cube binary xor --text "Hello" --key "key"        # 230015070a
cube binary xor --text "Hello" --key-hex 6b6579    # 230015070a
```

Omit `--text` to read stdin until EOF. Input bytes, including newlines, are
preserved. Use `printf`, rather than `echo`, when a trailing newline is unwanted:

```sh
printf 'Hello' | cube encode base64
printf '123456789' | cube hash crc32               # cbf43926
printf 'AP8=' | cube decode base64 --output raw > bytes.bin
cube binary xor --key-hex ff --output raw < bytes.bin > transformed.bin
```

Commands under `cube decode` and XOR default to **hexadecimal plus a newline**
so arbitrary bytes have a predictable representation. `--output raw` writes exact bytes without a
newline. Other operations write text followed by a newline. Text ciphers require
UTF-8; encoding, hashing, and XOR accept arbitrary stdin bytes. `--text` and `--key`
use UTF-8. Specify exactly one of `--key` or `--key-hex` for XOR.

Caesar requires `--shift` in the signed 32-bit range `-2147483648` to `2147483647`.
Positive shifts advance ASCII letters; negative shifts move backward. Shifts are
normalized modulo 26, so zero and multiples of 26 preserve the text. Use `--decode`
with the original shift to reverse it, including the minimum shift. Non-ASCII
characters, punctuation, whitespace, and combining characters remain unchanged;
no Unicode normalization is performed. Shift 13 agrees with ROT13.

Results go to stdout and errors to stderr. Exit codes are `0` for success
(including help/version), `1` for invalid operation input or I/O failure, and
`2` for invalid command usage. A downstream closed pipe exits successfully.

## Python quick start

Use Python 3.10 or newer with Rust installed. From the repository root:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install .
```

On Windows, create the environment with `py -m venv .venv` and activate it with
`.venv\Scripts\Activate.ps1` in PowerShell.

```python
import cube

assert cube.base32_encode(b"foo") == "MZXW6==="
assert cube.base32_decode("MZXW6===") == b"foo"
assert cube.base58_encode(b"Hello World") == "JxF12TrwUP45BMd"
assert cube.base58_decode("112") == b"\x00\x00\x01"
assert cube.base64_encode(b"Hello") == "SGVsbG8="
assert cube.base64_decode("SGVsbG8=") == b"Hello"
assert cube.hex_encode(b"\x00\xff") == "00ff"
assert cube.hex_decode("00FF") == b"\x00\xff"
assert cube.caesar_encode("Attack at dawn!", 3) == "Dwwdfn dw gdzq!"
assert cube.caesar_decode("Dwwdfn dw gdzq!", 3) == "Attack at dawn!"
encoded = cube.caesar_encode("AaZz 世界", -2147483648)
assert cube.caesar_decode(encoded, -2147483648) == "AaZz 世界"
assert cube.rot13("Hello, 世界") == "Uryyb, 世界"
assert cube.crc32(b"123456789") == "cbf43926"
assert cube.crc32(b"\x00\xff\x80") == "81dda740"
assert cube.crc32(b"") == "00000000"
assert cube.sha256(b"Hello") == (
    "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969"
)
assert cube.xor(b"Hello", b"key") == b"\x23\x00\x15\x07\x0a"
```

Binary inputs and outputs use `bytes`; encoded text and classical ciphers use `str`.
Encode Unicode explicitly, for example `cube.sha256("Olá".encode("utf-8"))`.
Invalid encoded input and empty XOR keys raise `ValueError`; incorrect argument
types raise `TypeError`. Caesar's `shift` uses the same signed 32-bit range as the
CLI; out-of-range integers raise `OverflowError`. Type stubs are included.
The local distribution name is `cubearc-cube`; the import name is `cube`.
Package registry availability has not been established.

## Tests and development

```sh
cargo test --workspace --all-features --locked
python -m pytest bindings/python/tests
```

Complete the [development setup](CONTRIBUTING.md#setup) before Python tests.
That guide also covers formatting, linting, and core coverage commands.
CI enforces at least 80% core line coverage locally before attempting
an upload to Codecov. Badges display the external services' current status.

Maintainers: activate `cubearc/cube` in Codecov. Private repositories require a
repository upload token stored as the GitHub Actions secret `CODECOV_TOKEN`.
For a public repository, either use that token or disable public-repository token
authentication in the CubeArc organization's **Global Upload Token** settings.
Public fork pull requests can upload without the upstream secret. If using a token,
add it separately to Dependabot secrets for Dependabot uploads. See
[Codecov's token documentation](https://docs.codecov.com/docs/codecov-tokens).
Unconfigured or failed external uploads do not fail CI; local coverage failures do.
The coverage badge is not evidence of an upload until Codecov has been connected
and received a report.

## Architecture and direction

`crates/cube-core` owns the algorithms, grouped by encoding, ciphers, hashing, and
binary transformations. `crates/cube-cli` and `bindings/python` adapt those APIs
to their respective interfaces. See [the architecture overview](docs/architecture.md).
Open the Rust API reference with `cargo doc -p cube-core --no-deps --open`.

Future work may add puzzle-solving operations and composable pipelines. Those
features are not implemented here. Recipes will live separately in
[`cubearc/recipes`](https://github.com/cubearc/recipes).

Track bugs and feature requests in [GitHub Issues](https://github.com/cubearc/cube/issues).
Planning lives in [CubeArc Roadmap](https://github.com/orgs/cubearc/projects/1),
with release scope defined by [milestones](https://github.com/cubearc/cube/milestones).
Use `cube ops list` to discover implemented commands and `--json` for their
shared operation descriptors.

Contributions are welcome through [CONTRIBUTING.md](CONTRIBUTING.md). Community
participation follows the [Code of Conduct](CODE_OF_CONDUCT.md). Report security
issues using [SECURITY.md](SECURITY.md).

## License

Cube is licensed under the [Apache License 2.0](LICENSE). Additional notices for
the EXIF parser and its dependency are in [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES)
and included in Python distributions.

## Operation discovery

Operation metadata comes from static Rust descriptors beside each implementation.
Existing direct Rust and Python functions remain the execution API.

```sh
cube ops list --category encoding
cube ops list --json
cube ops describe cipher.caesar.encode
cube ops describe binary.xor --json
```

JSON discovery uses `schema_version: 1` and stable namespaced operation IDs.
Descriptions declare semantic input/output types, parameter constraints/defaults,
determinism, reversibility and examples. JSON lists sort by ID; the default human
list retains its existing command order. Unknown IDs/categories fail explicitly.
Consumers should tolerate added fields and operations; IDs are never reused for
incompatible behavior. Metadata does not imply a generic operation executor.

```python
assert cube.describe_operation("binary.xor")["input_type"] == "bytes"
assert cube.operations("binary")[0]["id"] == "binary.xor"
```

Use `--json-errors` for a single schema-versioned error object on stderr. Success
output is unchanged. The error fields are `code`, `message`, `operation`,
`parameter`, and absolute byte `offset`; unavailable context is `null`. Codes
include `invalid_input`, `invalid_parameter`, `unsupported_format`,
`resource_limit`, `io` and CLI-only `invalid_usage`. Exit codes remain 0/1/2.
Native Python operation `ValueError` exceptions retain their class and expose the
same context as attributes. Python argument type/range errors remain the native
`TypeError`/`OverflowError`. No failed input contents are stored or uploaded.

## File inspection and extraction

```sh
cube inspect sample.bin
cube file signature sample.bin --json
cube file embedded sample.bin --format png --max-results 100
cube file extract sample.bin --offset 8 --length 16 --output selected.bin
cube file extract sample.bin --offset 8 --length 16 --stdout > selected.bin
```

Inspection is read-only. Paths must identify regular files; directories, devices
and symbolic links are rejected. Header rules recognize PNG, JPEG, GIF87a/89a,
PDF, ZIP local/empty headers, gzip/deflate, RIFF/WAVE and classic TIFF. Matches
provide exact byte evidence, not proof of a valid file or hidden data. Unknown
headers return no candidates. Embedded scanning preserves overlapping matches and
absolute offsets; it reads at most 64 MiB and returns `complete: false` when the
scan or result budget is exhausted. Result limits range from 1 to 10000.

`inspect` reads filesystem size/timestamps and 12 header bytes. Unavailable times
are null. Timestamps carry seconds/nanoseconds relative to the Unix epoch and an
explicit `before_unix_epoch` flag. No EXIF fields are disclosed automatically.

Extraction requires explicit offset, length and destination or raw stdout. Ranges
are half-open; zero-length extraction at EOF succeeds. Overflow and ranges beyond
EOF fail. Destinations are created exclusively: existing files, same-path aliases,
hard links and symlinks cannot be overwritten. A newly created partial output is
removed on copy failure. Streaming copies use a 64 KiB buffer and permit up to
1 GiB; byte-returning APIs allocate at most 16 MiB. Input files must remain stable
during analysis; short reads fail instead of returning a complete result.

```python
info = cube.inspect_file("sample.bin")
candidates = cube.scan_file("sample.bin", format="png", max_results=100)
assert cube.extract_bytes(b"\x00\xff\x80", 1, 2) == b"\xff\x80"
# For larger results, use extract_file_range or copy_file_range with a binary writer.
```

```sh
cube file hex sample.bin --offset 16 --length 64 --width 16
cube file strings sample.bin --min-length 4 --json
cube file compare first.bin second.bin --max-results 100 --json
```

Hex inspection defaults to the first 256 bytes, with widths from 1 to 256. JSON
rows carry explicit hex; Python rows additionally expose original `bytes`. ASCII
previews replace nonprintable bytes with `.`. Hex allocations, including row
and rendered-data overhead, are bounded by a conservative 16 MiB budget.

String extraction recognizes exactly ASCII bytes 0x20–0x7e, including spaces.
Minimum length counts bytes, not Unicode characters; no encoding is guessed.
Runs cross buffer boundaries and include EOF. A run cannot exceed 1 MiB, total
text is bounded to 16 MiB, and the scan/result caps are explicit in `complete`.

Binary comparison reports one-byte differing ranges at corresponding offsets,
including unequal tails (`null`/`None` means absent, not zero). The selected range
is validated against the larger file. Counts remain exact when the difference
listing is capped; `complete` describes the listing. Comparison processes at most
64 MiB per request and differences are successful results (CLI exit 0).

Python provides `file_hex`, `file_strings` and `compare_files` with equivalent
keyword parameters and structured results. File algorithms remain solely in Rust.

```sh
cube file exif sample.jpg --json
cube file appended sample.png --json
```

EXIF is an **explicit** JPEG/classic-TIFF operation backed by the pure-Rust,
BSD-2-Clause `kamadak-exif` parser. Inputs are limited to a 1 MiB metadata prefix;
TIFF beyond that size is unsupported by this bounded interface. Safety preflight
limits directory traversal to 32 unique directories, 4096 entries and 2 MiB of
original decoded value payloads. Cyclic/shared directories and invalid offsets
fail. The parser additionally limits primary IFDs to eight. No image pixels are
decoded, external references followed, or metadata saved. Explicit EXIF output may
contain location or other sensitive tags already present in the input.

EXIF results retain IFD/context/tag IDs, repeated and unknown tags, original typed
values and a separate human display with units. ASCII values remain byte arrays,
rationals retain numerator/denominator pairs, and nonstandard floating-point
values retain exact IEEE bits. Absent EXIF is a successful `present: false` result;
malformed and unsupported input use distinct errors. Python provides
`inspect_exif(data)` and `file_exif(path)` with matching structured results.

Appended-data detection currently supports basic PNG framing. It checks the PNG
signature, IHDR parameters, chunk lengths/order and CRCs through a valid IEND,
then reports exact logical-end/trailing offsets and counts. It does **not** decode
or validate compressed pixels and does not assert that trailing bytes hide data.
Unknown critical chunks and APNG are unsupported. The scan is limited to 64 MiB
before IEND and 10000 chunks; trailing content is not loaded. Python exposes
`appended_data(path)`. See the [PNG specification](https://www.w3.org/TR/png-3/).

## PNG8 image foundation

`cube image inspect sample.png --json` reports exact dimensions, source color
model, palette, transparency and raw chunk metadata. `cube image chunks sample.png
--json` inspects framing/CRCs through the shared file-forensics parser. Strict
inspection rejects CRC mismatches; `--allow-bad-crc` explicitly reports them while
still rejecting unsafe framing. Payloads are absent by default; `--payload-budget`
opts into at most 1 MiB of total raw chunk payload export.

Image decoding supports 8-bit grayscale, grayscale+alpha, RGB, RGBA and indexed
PNG, preserving row-major interleaved samples with a top-left origin. Other
formats/depths fail explicitly. Limits: 32 MiB encoded, 16384 pixels per side,
4 Mi pixels, 16 MiB of decoded samples and a 32 MiB decoder-internal budget.
No gamma/profile conversion, premultiplication or compositing occurs; text/ICC
metadata is not decompressed. eXIf inspection is explicit and reuses bounded TIFF
parsing. Encoding preserves sample values, palette and tRNS, and drops all other
metadata to avoid stale attributes. Source bytes need not round-trip identically.

```python
image = cube.Image.open("sample.png")
info = image.info()
samples = image.samples  # exact bytes, including original indices for indexed PNG
png = image.encode_png()
image.save_png("new-output.png")  # exclusive creation
```

`cube image convert sample.png --output new.png` re-encodes exact samples.
Indexed expansion is explicit through `--expand-palette` or
`image.expand_palette()` and produces unpremultiplied RGBA, supplying alpha 255
for palette entries without tRNS. Images and existing outputs are never overwritten.

Image pixels use zero-based `(x,y)` coordinates. Regions are `[x,y,width,height]`,
positive and entirely inside the image; no clipping occurs. Native channel names
are `red`, `green`, `blue`, `alpha`, `gray`, and `index` as available. Selections
preserve user order and reject duplicates/missing channels. Indexed native data
remains indices; request palette expansion explicitly for decoded RGBA values.

```sh
cube image pixel sample.png --x 0 --y 0 --json
cube image pixels sample.png --region 0,0,2,2 --channels blue,red --json
cube image channel sample.png --channel red --output red.bin
cube image channel sample.png --channel alpha --png-output alpha.png
cube image histogram sample.png --channels red,green,blue --json
cube image compare first.png second.png --mask-output differences.png --json
```

Raw exports require `--output` (new file plus shape/channel metadata on stdout)
or explicit `--stdout` (bytes only, no newline). `--json` exports coordinate records
under a 10000-pixel cap. One-channel exports can use `--png-output` without scaling.
Histograms count exact selected tuples in lexicographic order; alpha is included
when selected, including hidden RGB under transparent pixels. Native indexed
histograms count indices; expanded palette aliases count as the same RGBA color.
More than `--max-colors` distinct tuples fails rather than approximating counts.

Comparison requires equal dimensions and compatible native models/palette/tRNS,
or an explicit compatible channel subset or palette conversion. No resizing,
blending or quantization occurs. Difference counts remain exact when the listing
is capped; `complete` identifies that condition. Optional masks use exactly 0/255.
Use Python `image.pixel`, `.pixels`, `.channel`, `.histogram` and `.compare` with
matching keyword parameters. Pixel/channel results contain original `data: bytes`
and shape/channel metadata; comparisons can return a native `mask: Image`.

Image geometry uses exact sample rearrangement: `image rotate --degrees 0|90|180|270`
rotates clockwise (90/270 swap dimensions); `image mirror --axis horizontal|vertical`
reverses x/y respectively. `image crop --region x,y,width,height` uses a checked
half-open rectangle and discards unselected pixels. All require `--output new.png`.
Indices, palettes, transparency and alpha survive geometry unchanged.

`image invert` applies `255-value` to color channels, preserving alpha unless
explicitly selected with `--channels`. `image reorder --channels blue,green,red,alpha`
requires a full permutation and keeps the output model fixed; moving alpha is explicit.
Both are sample-reversible. `image grayscale` loses color using
`(77R+150G+29B+128)>>8` on stored samples, without gamma/profile correction.
Alpha is retained; color-key transparency becomes per-pixel alpha to avoid collisions.
Color transforms reject indexed data unless `--expand-palette` is explicit.
Every encoded/derived image drops other source metadata (including EXIF/profile/text);
lossless PNG samples do not imply byte-identical files. Existing destinations are rejected.
Python exposes immutable `Image.rotate(90)`, `.mirror("horizontal")`,
`.crop((x,y,width,height))`, `.invert()`, `.reorder([...])` and `.grayscale()`;
each returns a new `Image` with reusable samples.

`image bit-plane sample.png --channel red --bit 0 --stdout` returns one byte per
bit, each exactly 0 or 1, in row-major order. Bit zero is the sample LSB; bits 0–7
are supported. `--packed` emits MSB-first packed bytes with zero padding; file
metadata includes the valid-bit count. `--png-output plane.png` explicitly maps
0/1 to 0/255 for display. `--json` returns bounded coordinate records and packed hex.
`Image.bit_plane("red", 0, visualization=True)` returns raw bytes under
`pixels["data"]`, `packed`, `valid_bits` and an optional display `image`.

```sh
cube image lsb sample.png --channels red,green --bits 0,1 --traversal row --bit-order msb --incomplete reject --output extracted.bin
cube image lsb sample.png --channels alpha --bits 7,6,5,4,3,2,1,0 --traversal column --reverse --stdout > extracted.bin
```

LSB extraction nests pixel coordinates, requested channels, then requested bit
positions. Channels and bits must be nonempty and distinct. Defaults are native
channel order (including alpha or palette indices), bits `[0]`, row-major, forward,
MSB packing and rejection of incomplete bytes. `--traversal column` visits y before
advancing x; `--reverse` reverses coordinates only. `--bit-order lsb` puts the first
bit at output bit 0 instead of bit 7. `--incomplete drop|pad` explicitly discards
trailing bits or zero-pads unused positions; effective parameters and available,
valid and dropped-bit counts accompany file output. `--region x,y,width,height`
and explicit `--expand-palette` work as in other analysis commands. Extraction
is bounded by the shared image limits and emits at most 16 MiB. Raw stdout never
adds a newline or metadata. No extraction implies that a hidden message exists.
Python `image.lsb(channels=["red"], bits=[0], traversal="row", reverse=False,
bit_order="msb", incomplete="reject")` returns the same metadata and exact `data`
as `bytes`, ready for other Cube operations; unknown keyword options fail.

## Classical cipher additions

Atbash reverses ASCII alphabets (`ABC xyz` → `ZYX cba`). Vigenère uses A=0,
`C=(P+K) mod 26`, with inverse `P=(C-K) mod 26`. Standard Beaufort uses
`C=(K-P) mod 26` and is self-inverse (not Variant Beaufort). Vigenère/Beaufort
accept a nonempty ASCII-letter key, ignoring key case and advancing only for
ASCII input letters. Affine uses `C=(aP+b) mod 26`; a,b must be 0..25 and a must
be coprime to 26. Decoding uses a's modular inverse. All preserve ASCII case and
other Unicode/whitespace without normalization. These are classical puzzle
ciphers, not secure encryption. New cipher operations accept at most 1 MiB of
UTF-8 input and 64 KiB of key; invalid keys are errors even for empty input.

```sh
cube cipher atbash --text 'ABC xyz'
cube cipher vigenere --key LEMON --text ATTACKATDAWN
cube cipher vigenere --decode --key LEMON --text LXFOPVEFRNHR
cube cipher beaufort --key B --text A
cube cipher affine --a 5 --b 8 --text AFFINECIPHER
```

Python exposes `atbash(text)`, `vigenere_encode(text,key)`,
`vigenere_decode(text,key)`, `beaufort(text,key)`, `affine_encode(text,a,b)`
and `affine_decode(text,a,b)`, calling the same Rust functions. New operations
raise contextual `ValueError` on invalid parameters or resource limits.

## Explicit Unicode text operations

`text normalize --form NFC|NFD|NFKC|NFKD` uses
[UAX #15](https://www.unicode.org/reports/tr15/) with Unicode **18.0.0** data from
pinned `unicode-normalization` 0.1.26. NFC/NFD canonically compose/decompose;
NFKC/NFKD also fold compatibility distinctions. Normalization is not reversible
and never happens implicitly in another operation. Dependency/data-version updates
require compatibility review; outputs are deterministic for the published version.
Input is limited to 1 MiB UTF-8 and expanding output to 8 MiB.

`text reverse` reverses Unicode **scalar values**, including combining marks,
emoji joiners and CRLF pairs; visual graphemes may split. Reversing twice recovers
exact input. `text extract --indices 3,0,3` selects ordered zero-based scalar
positions, preserving duplicates; `--indices ''` returns empty text.
Alternatively, `--start 1 --stride 2 --stop 6` selects alternate positions before
an exclusive stop (start defaults to 0, stop to scalar length). Choose exactly one
mode; stride must be positive, all bounds must fit, and lists have at most 65536
indexes. No clipping, byte indexing, trimming or normalization occurs.

```sh
cube text normalize --form NFC --text 'café'
cube text reverse --text 'Cube 🧊'
cube text extract --start 1 --stride 2 --text abcdef
```

Python: `normalize_text(text, "NFC")`, `reverse_text(text)` and
`extract_characters(text, indices=[3,0,3])` or
`extract_characters(text, start=1, stride=2, stop=6)` call Rust directly.
`cube.UNICODE_VERSION` publishes the normalization data version tuple.
CLI text results retain the existing final-newline convention; invalid UTF-8
stdin fails without partial stdout.
