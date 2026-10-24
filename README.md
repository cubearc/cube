# Cube

[![CI](https://github.com/cubearc/cube/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cubearc/cube/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/cubearc/cube/branch/main/graph/badge.svg)](https://codecov.io/gh/cubearc/cube)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Cube is an open-source Rust toolkit for encoding, classical ciphers, hashing,
binary transformations, file/image forensics, Unicode text, symbolic encodings,
PCM WAVE analysis, and typed pipelines/recipes. Use the Rust library, `cube` CLI,
or Rust-backed Python package. All interfaces share the same implementations.

## Capabilities

| Operation | Behavior |
| --- | --- |
| Base32 | Uppercase, padded RFC 4648 standard encoding and strict decoding |
| Base58 | Case-sensitive Bitcoin alphabet; preserves leading zero bytes |
| Base64 | Padded RFC 4648 standard encoding and strict decoding |
| Hex | Lowercase encoding; decoding accepts either case |
| Caesar | Signed shift of ASCII letters modulo 26; preserves case and non-ASCII characters |
| Atbash / Vigenère / Beaufort / Affine | Explicit ASCII alphabets and keys; classical puzzle ciphers |
| Unicode text | Explicit normalization, scalar reversal and positional/alternating selection |
| Morse / Braille | Canonical International Morse text; exact Unicode Braille dot-mask cells |
| Integer bases | Exact signed integer text conversion between radices 2..36 |
| Pipelines / recipes | Bounded linear typed execution, strict versioned JSON and independent exact-output contract checks |
| PCM WAVE audio | Bounded metadata/chunk inspection, exact channels/frame ranges and integer waveform envelopes |
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

Bounded typed pipelines compose supported operations without implicit text
conversion. Community recipes belong separately in
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

## Morse and Braille cells

Morse supports International Morse **A–Z and 0–9** as defined by
[ITU-R M.1677-1](https://www.itu.int/rec/R-REC-M.1677-1-200910-I/).
Encode accepts lowercase, collapses runs of ASCII spaces and removes edge spaces.
Decode returns uppercase text; exact original case/spacing cannot be recovered.
The canonical grammar uses `.`/`-`, one ASCII space between symbols and ` / `
between nonempty words. Decode rejects edge/doubled spaces, unknown tokens,
unsupported punctuation/Unicode and newlines; no audio or timing inference occurs.

```sh
cube encode morse --text 'SOS 2'
# ... --- ... / ..---
cube decode morse --text '... --- ... / ..---'
# SOS 2
cube decode braille --text '⠀⠁⣿' --output hex
# 0001ff
```

Braille encodes **raw dot masks**, not ordinary text or natural-language Braille.
Byte bit `n−1` represents dot `n`; add the mask to U+2800 for the Unicode cell:

```text
1 4
2 5
3 6
7 8
```

Blank mask 0 → U+2800; mask 1 → ⠁; mask 255 → ⣿. Every byte round trips.
Encode consumes exact stdin bytes (or explicit UTF-8 bytes via `--text`);
decode accepts only U+2800..U+28FF and returns hex by default or exact binary with
`--output raw`. It does not strip a piped trailing newline: pass canonical text
explicitly. Errors identify original UTF-8 byte positions without dumping inputs.
These operations accept at most 1 MiB input; Morse output is bounded at 8 MiB,
Braille output at 3 MiB. Python exposes `morse_encode(str)`, `morse_decode(str)`,
`braille_encode(bytes)` and `braille_decode(str) -> bytes` over the Rust core.

## Integer radix conversion

```sh
cube number base --from-base 16 --to-base 2 --text FF
# 11111111
cube number base --from-base 36 --to-base 10 --text=-Z
# -35
```

`numbers::convert_base` / Python `convert_base(text, from_base, to_base)` convert
integer **representations**, separately from byte encodings. Bases are explicit
2..36, using ASCII 0–9/A–Z; input letter case is ignored and output is uppercase.
One leading ASCII `+`/`-` is accepted; leading zeros and negative zero are
canonicalized. No whitespace, radix-prefix syntax, separators, fractions, floats
or implicit byte-order conversion is supported. Magnitudes up to **2^128−1**
are handled exactly with checked Rust integer arithmetic, for either sign.
Input has at most 4096 digits (excluding the sign); output at most 128 digits plus
a negative sign. Invalid digits/radices and over-budget values fail explicitly.
Values round trip; original sign/zero padding/case spelling does not.

## PCM WAVE inspection

```sh
cube audio inspect sample.wav
cube audio inspect sample.wav --json
```

Initial audio support is little-endian **RIFF/WAVE integer PCM (format tag 1)**,
with 8/16/24/32-bit storage, 1–32 indexed channels and 1–384000 frames/sec.
This deliberately supports packed 24/32-bit tag-1 files encountered in analysis;
RF64, big-endian RIFX, floating-point, compressed and extensible WAVE, `wavl`
lists and multiple data chunks are explicit unsupported-format errors.
`fmt ` must precede `data`; PCM fmt length is 16, or 18 with an empty `cbSize`.
Unknown top-level chunks are retained as ID/offset/size facts without reading
payloads. Chunk sizes exclude their 8-byte headers and even-boundary padding.
Container bounds, mandatory padding presence, frame alignment and byte rates
validate strictly. Trailing bytes after RIFF are reported separately.

Rust `media::wave::inspect(&mut reader)` and `inspect_file(path)` return typed
metadata; Python `inspect_wave(bytes)` / `inspect_wave_file(path)` return the same
structured facts. Metadata inspection seeks past audio payloads without decoding
or buffering them. Physical files are capped at 1 GiB, framing at 10000 chunks;
paths must be regular files and are opened read-only. Frame counts are exact;
`duration_seconds = {numerator: frames, denominator: sample_rate}` is a rational
number of seconds, without rounding. A frame contains one sample per channel.
No device/recording/playback access or native multimedia library is required;
this works with the same Rust implementation on Linux, macOS and Windows.

### Exact PCM channel/range extraction

```sh
cube audio samples sample.wav --channels 1,0 --start-frame 0 --frames 100 --json
cube audio samples sample.wav --channels 0 --start-frame 100 --frames 1000 --output channel.pcm
cube audio samples sample.wav --channels 1 --frames 1000 --stdout > channel.pcm
```

Selections use absolute zero-based **frames**, with an exclusive end determined
by `--frames`; omission selects to end, zero selects empty data. Channel indexes
are ordered, unique and zero-based; omission preserves all source channels.
Outputs retain interleaved little-endian source bytes in requested channel order.
8-bit storage is unsigned 0..255; interpreted signed amplitude subtracts 128.
Packed 16/24/32-bit samples use two's-complement sign extension into i32.
No normalization, clipping, resampling, downmix or channel-label guessing occurs.

`--json` includes exact integer `values`, metadata and `data_hex`, capped at 10000
samples. `--stdout` writes only raw PCM bytes without a newline; `--output`
creates a new raw PCM file and prints interpretation metadata. Raw PCM is not a
WAVE container: retain its sample rate, width, channel mapping and frame size.
Existing destinations and source aliases are rejected. Rust
`media::pcm::extract(reader, selection)` / `extract_file(path, selection)` return
immutable `Samples` with `.raw()`, `.values()`, `.info()` and exclusive `.save()`.
Python `wave_samples(bytes, channels=[1,0], start_frame=0, frames=100)` and
`wave_file_samples(path, ...)` return metadata and exact `data: bytes`;
`include_values=True` explicitly adds the bounded signed integer listing.

Each request is capped at 1048576 frames, 300 seconds and 64 MiB of selected
source bytes. Extraction additionally caps output at 8 MiB / 2097152 selected
samples. Parameters and budgets validate before allocation. Source reads are
frame-aligned chunks of at most 64 KiB and cover only the selected frame range;
metadata scanning still skips other chunks. Typed Rust values allocate at most
8 MiB in addition to the raw result; structured CLI/Python output has the separate
10000-sample cap. Choose smaller intervals for larger recordings.

### Waveform envelope data

```sh
cube audio waveform sample.wav --channels 0 --bucket-frames 1024 --json
cube audio waveform sample.wav --start-frame 100 --frames 5000 --bucket-frames 1000
```

This returns **data**, not a plot: exact minimum/maximum signed amplitude values
per selected channel, ordered by absolute `start_frame`, with each bucket's
actual `frames`. The final partial bucket is included without padding, and zero
frames produce an empty series. `sample_rate` is the frame/sec time base; divide
absolute frame positions by it for exact time. Integer units match PCM extraction;
there is no normalization, decimation, resampling, FFT or signal detection.

`bucket_frames` defaults to 1024 and accepts 1..1048576; buckets larger than the
selection still produce one partial bucket. `max_buckets` defaults to 1000,
accepts 1..10000, and the total number of channel/bucket pairs is also capped at
10000. Exceeding a budget fails before reading audio, rather than returning an
incomplete series. Shared frame/duration/64 MiB source budgets apply; streaming
uses at most 64 KiB each for source/selected PCM blocks and retains only bounded
extrema/results. Output is independent of how those blocks are divided.

Rust `media::envelope::from_reader` / `from_file` stream selected data;
`from_samples(&Samples, bucket_frames, max_buckets)` consumes the PCM contract.
`Builder::new(&first, bucket_frames, max_buckets)` initializes metadata without
consuming the first block: call `.push(&first)` and then push contiguous matching
blocks before `.finish()`. Format, channel mapping and continuity errors fail
before consuming a block. Python `wave_envelope(bytes, bucket_frames=1024, ...)`
and `wave_file_envelope(path, ...)` expose the same selections and structured
results over the shared Rust streaming implementation.

## Typed linear pipelines

The pipeline engine has an explicit allowlist of bounded adapters for current
Base32/58/64/hex, classical ciphers, CRC32/SHA-256, XOR and UTF-8 conversion.
`cube pipeline adapters` reports each descriptor and its operation-contract
version. Discovery alone never grants execution: file/image/audio, shell,
network and plugin operations are not adapters in this initial engine.
All algorithms still live in their ordinary Rust functions.

```sh
cube pipeline validate --input-type bytes --steps '[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]'
cube pipeline run --hex 00ff --steps '[{"operation":"encoding.hex.encode","version":1},{"operation":"encoding.hex.decode","version":1}]' --format raw > result.bin
```

Steps require `operation`, explicit contract `version: 1`, and optional
`parameters`. Parameters retain JSON scalar types; XOR's binary key is explicitly
`{"key":{"base64":"/w=="}}`. Unknown fields/IDs/versions/parameters and
incompatible adjacent types fail before any transform. Key validation reuses
core validation without running an empty transform. Empty pipelines are identity.
Direct APIs remain unchanged. `text.utf8.encode` / `text.utf8.decode` are explicit
text/byte conversions, not automatic glue: Rust `text::to_utf8` / `from_utf8`,
CLI `text to-bytes --output raw|hex` / `text from-bytes`, and Python
`utf8_encode(str)` / `utf8_decode(bytes)` preserve NUL/newlines and reject invalid
UTF-8 without replacement or normalization (1 MiB limit).

Pipeline values distinguish `bytes`, UTF-8 `text`, and the initial inert structured
`byte_range`. Wire JSON uses `{"type":"bytes","base64":"AP8="}`,
`{"type":"text","data":"Hello"}`, or
`{"type":"byte_range","offset":3,"length":4}`; a missing/null range length
means unspecified, and checked ranges grant no file permissions. Only identity
currently accepts structured ranges; transform adapters consume bytes or text.

Hard limits are 64 steps, 1 MiB per value, 256 KiB parameter data per plan,
16 MiB conservative work bytes and 4 MiB retained intermediates. `--limits`
accepts a JSON object lowering `max_steps`, `max_value_bytes`, `max_work_bytes`
and `max_capture_bytes`. JSON transport is capped at 256 KiB, 16 levels and 4096
nodes, rejecting duplicate keys. Base58 adapters additionally cap each step's
input at 4096 bytes because its conversion cost is not linear. Before execution,
size/work bounds are calculated conservatively for the entire plan; these can
reject a plan whose data-specific decoded results would have been smaller.
Work counts initial input plus each step's input/output upper bounds. Range
values count as 16 logical bytes; byte/text values count their exact lengths.

`--capture 0,2 --format json` explicitly retains selected zero-based step outputs;
otherwise no intermediates are copied or persisted. Capture indexes are unique
and retention has an independent budget. CLI input is explicit `--text`, `--hex`,
`--input regular-file` or `--value tagged-JSON`; no selector reads exact raw stdin.
Byte output defaults to hex plus newline, text to UTF-8 plus newline;
`--format raw` adds nothing, and `--format json` preserves types/captures.
`--output new-file` uses exclusive creation, rejecting existing paths and aliases.

Rust `pipeline::Plan::validate` creates an immutable plan and `.execute` returns a
typed final value plus optional captures. An optional `AtomicBool` is checked at
entry and before/after each bounded operation; running steps are not preempted,
and no hard wall-clock timeout is claimed. Failures stop execution and retain
zero-based step/ID/parameter context and original causes, without dumping inputs.
Python `validate_pipeline(steps, "bytes")` and
`execute_pipeline(steps, b"\x00\xff", capture=[0])` use the Rust engine;
results contain native `bytes`/`str`, `output_type` and native-valued
`intermediates`. Optional `limits` lower budgets. A sticky `CancellationToken`
can be cancelled from another Python thread while the GIL is released; use a
new token per run. No filesystem/environment/network capability is granted.

## Versioned JSON recipes

Cube owns recipe parsing, validation and execution; community definitions belong
in [`cubearc/recipes`](https://github.com/cubearc/recipes). Recipes contain data,
not executable scripts. Format 1 uses JSON for established tooling, strict scalar
types and explicit base64 without YAML tags or a custom language. This original
round-trip example can be saved to a caller-selected `roundtrip.json`:

```json
{
  "format_version": 1,
  "engine_api": 1,
  "input": {"source": "slot", "name": "payload", "type": "bytes"},
  "output": {"type": "bytes"},
  "steps": [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1}
  ]
}
```

```sh
cube recipe validate roundtrip.json
cube recipe run roundtrip.json --hex payload=00ff --format raw > result.bin
cube recipe run roundtrip.json --input payload=sample.bin --format json --capture 0
cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
```

`validate` reads only the explicitly selected recipe, checks every contract and
reports types/limits without executing algorithms or requiring a slot binding.
`run` requires exactly the declared input slot; missing, extra and duplicate
bindings fail. Bindings use `--text name=value`, `--hex name=hex`,
`--input name=path` (regular raw-byte file) or `--value 'name={...}'` (tagged JSON).
Format 1 permits one slot, named with 1..64 ASCII letters, digits, `_` or `-`.
Alternatively use `"input":{"source":"inline","value":{"type":"bytes","base64":"AP8="}}`;
inline recipes accept no bindings. Bytes require canonical padded base64; text
uses `{"type":"text","data":"..."}` and inert structured ranges use
`{"type":"byte_range","offset":0,"length":2}` (`length:null` means unspecified).
No slot is automatically read from stdin or resolved as a path or environment variable.

Optional `limits` has the same lowered step/value/work/capture bounds as pipelines.
The complete UTF-8 recipe is limited to 256 KiB, depth 16 and 4096 JSON value nodes;
inline/parameter representations count toward these limits, so they can be smaller
than the 1 MiB runtime value maximum. Duplicate keys, unknown fields (including
`includes`, `shell` and `network`), invalid tags, floating-point contract versions
and unsupported format/engine/adapter versions are errors. Step errors include a
zero-based index and operation ID. Validation reports omit inline data, bindings
and parameter values. Errors do not print these values.

`format_version:1` and `engine_api:1` are exact compatibility contracts, independent
of the reported package version. Every step must specify its adapter `version`;
`cube pipeline adapters` reports supported IDs, versions and parameter metadata.
An ID alone is insufficient. Breaking engine or adapter changes require a new
corresponding contract version. Existing versions must keep their behavior or be
rejected explicitly. There is no automatic migration or fallback.

Execution uses the pipeline engine and its exact binary, deterministic, bounded
behavior. Captures are opt-in, `--format raw` emits exact bytes/text without an
added newline, and `--output path` creates an exclusive new file. Existing files
are never overwritten by this option. Definitions cannot execute shell commands,
read unrelated files or secrets, request network access, load native code or
persist results. All I/O comes from explicit CLI arguments or the SDK caller.
Recipes require no multimedia, graphical or additional native dependency.

### Independent recipe compatibility tests

`cube recipe check` is a stable public contract-test entry point that returns JSON
with `passed:true`, format/engine API and package version, or a nonzero exit status
with a safe diagnostic. It compares exact typed output; it does not write files.
A community repository can test its own definitions against any installed Cube:

```sh
cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
# Negative fixture: different expected bytes must fail, without exposing input data.
if cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"/wA="}'; then
  exit 1
fi
```

Additional negative fixtures should assert `recipe validate` fails for unsupported
versions, unknown operations/parameters, type mismatches and duplicate JSON keys.
Tests and expected results belong beside community definitions in `cubearc/recipes`;
they invoke the engine, never copy operation algorithms. Cube's own synthetic
integration tests exercise the same public boundary with success and failure cases.

```python
from pathlib import Path
import cube

document = Path("roundtrip.json").read_text(encoding="utf-8")
info = cube.validate_recipe(document)
result = cube.execute_recipe(document, {"payload": b"\x00\xff"}, capture=[0])
assert result["output"] == b"\x00\xff"
assert result["intermediates"] == {0: "00ff"}
assert cube.check_recipe(document, {"payload": b"\x00\xff"}, b"\x00\xff")["passed"]
```

The Python functions bind native `bytes`, `str` or tagged value dictionaries;
execution detaches from Python and accepts the same optional `CancellationToken`
as pipelines. Rust uses `pipeline::recipe::Recipe::parse`, `.info`, `.execute` and
`.check`. All interfaces share the Rust engine; none use CLI subprocesses as their
SDK implementation. Pipelines remain linear: branching, scripts, privileged
capabilities, plugins and graphical editing are outside this contract.

## Explainable input triage

```sh
cube investigate triage sample.bin
cube investigate triage sample.bin --json
```

`investigate triage` reads at most 12 header bytes from an explicitly selected
regular file. `--max-read-bytes` may lower that bound; insufficient budgets fail
before reading. Rust `investigation::triage::{bytes, reader}` and native Python
`cube.triage_input(data, max_read_bytes=12)` share the same rules. Rust callers can
also supply an existing `FileInfo` through `from_file_info`; its source is labeled
as caller evidence and no additional bytes are read.

Results contain stable rule IDs, supporting fact IDs/offsets, suggested available
operation IDs and explicit limitations in deterministic order. PNG candidates
suggest chunk inspection, JPEG/TIFF candidates suggest EXIF, WAVE candidates suggest
stream metadata, and unknown input suggests bounded hex/ASCII string inspection.
Signature candidates do not prove structural validity or concealed content. Triage
never executes suggestions, decodes pixels, extracts data or consults filenames,
network services or AI. Header contents, EXIF values and paths are not included.

## Investigation records and verified replay

```sh
cube investigate pipeline sample.bin --identify --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube investigate pipeline sample.bin --identify --steps '[]' --save session.json --json
cube investigate report session.json
cube investigate replay session.json sample.bin --json
```

Investigation sessions are caller-owned memory. Each run has a stable session-local
ID, input ID, operation/version, explicit parameters or redactions, result kind,
SHA-256 digest of compact typed result JSON, and succeeded/failed/skipped state.
Findings reference their input/run and offsets; signature candidates and user
hypotheses remain separately labeled. Runtime failures are recorded with contextual
errors. Invalid contracts fail before execution. `--identify` records SHA-256 of the
complete input as its content identity, not a claim of trust. Without that identity,
reports explain why size alone cannot verify replay.

The CLI prints an escaped human report or a versioned JSON record with
`schema_version:1`, explicit `status`, `replay_gaps`, `uncertainty`, and `session`.
The nested session contains `format_version:1`, engine API/package version,
`input`, `limits`, `runs`, `findings` and `hypotheses`. Paths, filenames, filesystem
timestamps and raw inputs are never stored. Default export excludes result
contents, nonempty pipeline parameters and user hypotheses. `--save path` is the
only saving action; it creates a new local JSON file and refuses overwrites,
including source aliases. Nothing is automatically uploaded or persisted.

Explicit `--include-result 2` retains/exports only that run's result. Pipeline
binary results use tagged canonical base64, text remains explicitly typed, and
structured inspection results retain their JSON types. `--include-parameters 2`
retains/exports that run's original parameters; they may contain keys.
`--hypothesis '...'` adds a user interpretation; saving it requires
`--include-hypotheses`. Omitted data cannot be recovered from a record. To remove
saved data, delete the explicitly chosen JSON file with normal filesystem tools;
there are no hidden session copies, database or cache. The SDK can discard a
selected in-memory result or redact parameters before export.

Records are limited to 128 KiB, retained result JSON to 64 KiB, runs to 64,
findings to 128 and hypotheses to 32 (1024 UTF-8 bytes each). Rust `SessionLimits`
may lower those bounds. Preview contracts accept at most 4096 bytes per range.
Direct session execution/replay buffers are capped at 1 MiB. Insufficient count,
record or retention budgets fail before execution or explicitly omit selected
results/findings with a partial-status warning; no omission is silently complete.
The shared strict JSON parser also rejects duplicate/unknown fields, excessive
nesting/nodes, invalid IDs, broken references, inconsistent states and versions.

Replay first validates all recorded contracts and required parameters, then
compares the caller-supplied complete input's size/SHA-256 identity. Only afterward
does it execute the same bounded Rust operations/pipeline engine and compare typed
result digests/states. Skipped runs stay skipped; user hypotheses are not replayed
as machine evidence. Redacted parameters, absent identities, unsupported versions,
changed inputs and differing results are explicit failures. Compatibility uses
engine API/operation-contract versions, not package-version equality. Reports are
observations, not signed attestations or proof that hidden content exists.

```python
import cube

data = b"\x00\xff"
session = cube.InvestigationSession(data, identify=True)
run_id = session.execute_pipeline(data, [
    {"operation": "encoding.hex.encode", "version": 1}
], retain_result=True)
session.add_hypothesis("A candidate worth inspecting", [2])
print(session.report())
record = session.to_json()  # private defaults; no automatic file I/O
replayed = cube.InvestigationSession.from_json(record).replay(data)
selected = session.to_json(include_results=[run_id], include_hypotheses=True)
session.discard_result(run_id)
```

The SDK's `run(data, definition)` accepts a strict JSON run contract: `identity`,
`triage` with `max_read_bytes`, `hex` with `offset/length/width`, `strings` with
`offset/length/min_length/max_results`, or `pipeline` with versioned steps and
lowered limits. `execute_pipeline` is a convenience using default pipeline limits.
`record_parameters=True` and `retain_result=True` opt into memory retention;
`to_json(include_parameters=[id], include_results=[id])` independently controls
export. `redact_parameters(id)` permanently removes recorded key values in memory.
Rust uses `investigation::session::{Session, RunSpec, Capture, Export}`. CLI and
native Python share these APIs; cancellation uses the existing optional token.
