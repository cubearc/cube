# Operations Guide

Choose operations by what you know about the input and what you want to learn.
Use `cube ops list --json` to find categories and stable operation identifiers,
then `cube ops describe <identifier>` and the command's `--help` for exact
parameters. Python exposes the same metadata through `operations()` and
`describe_operation()`. This guide explains choices and representative workflows;
it is not a second catalog. See the [CLI](cli.md) or [Python](python.md) guide for
input/output handling and locally generated image/audio fixtures.

## Encoding and decoding

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

## Classical ciphers

Use the `ciphers` category when you have a classical letter substitution and a
specified key/alphabet. These operations do not provide secure modern encryption.

```sh
cube cipher vigenere --key LEMON --text ATTACKATDAWN
# LXFOPVEFRNHR
cube cipher affine --decode --a 5 --b 8 --text IHHWVCSWFRCP
# AFFINECIPHER
```

Caesar, ROT13 and Atbash operate on ASCII letters and preserve case and other
Unicode. Vigenère/Beaufort keys contain nonempty ASCII letters; key position
advances only for ASCII input letters. Standard Beaufort is self-inverse, not
Variant Beaufort. Affine uses `C=(aP+b) mod 26` with A=0, `a,b` in 0..25 and `a`
coprime to 26. Use the original parameters to decode. Caesar accepts a signed
32-bit shift; newer keyed ciphers cap input at 1 MiB and keys at 64 KiB.

Book coordinates use the exact caller-supplied UTF-8 source. Choose
`word_character` (`word:scalar`), `line_character` (`line:scalar`) or
`line_word_character` (`line:word:scalar`); separate records with commas. All
components use the explicit base (default 1, optionally 0). Decimal leading zeroes
and surrounding ASCII whitespace are accepted; signs and empty records fail.
Words split on Unicode whitespace and retain punctuation, matching `text.select`.
LF/CRLF line endings are excluded; blank lines count, bare CR and line spaces
remain data, and a final newline creates no extra line. Characters are Unicode
scalars, including separate combining marks, with no normalization.

Record the source edition and a SHA-256 of its exact bytes to reproduce results.
Source and coordinate text each allow 1 MiB, with at most 65,536 coordinates and
indexed units (lines plus words for triples). Output is at most 256 KiB. Invalid
coordinates fail as a whole, with a record number and no guessed substitute.
No book is bundled, fetched or stored. This direct operation has no pipeline
adapter: the separate source stays caller-controlled rather than being embedded
in recipe parameters. Its text result can be passed to ordinary text operations.

Modern encrypted-file formats remain outside the current cipher API. The
[format evaluation](https://github.com/cubearc/cube/issues/78) records a possible
age v1/X25519 subset and the parser-budget/interoperability work needed before
implementation. There is no modern decryption operation in this release.

## Hashing and checksums

Use SHA-256 or SHA-512 for one-way content digests. Neither is password hashing
or message authentication. MD5 and SHA-1 are legacy compatibility tools, unsuitable
for collision-resistant security. They help compare historical artifacts without
providing a security guarantee. All consume exact bytes and return lowercase hex:
SHA-256 has 64 digits, SHA-512 128, SHA-1 40 and MD5 32, including leading zeroes.

Checksums detect accidental corruption and are not cryptographic hashes:

| CLI name | Exact variant | Polynomial | Init | Reflected input/output | Final XOR | Hex digits |
| --- | --- | --- | --- | --- | --- | --- |
| `crc32` | CRC-32/ISO-HDLC (IEEE) | `04c11db7` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc32c` | CRC-32/ISCSI (Castagnoli) | `1edc6f41` | `ffffffff` | yes/yes | `ffffffff` | 8 |
| `crc16-ibm3740` | CRC-16/IBM-3740 | `1021` | `ffff` | no/no | `0000` | 4 |

```sh
cube hash sha512 --text abc
cube hash crc32c --text '123456789'
# e3069283
cube hash crc16-ibm3740 --text '123456789'
# 29b1
```

The maintained RustCrypto `sha2`, `sha1` and `md-5` crates supply digests;
`crc` supplies the two new exact CRC variants, while existing CRC32 retains
`crc32fast`. Vectors follow [RFC 6234](https://www.rfc-editor.org/rfc/rfc6234),
[RFC 1321](https://www.rfc-editor.org/rfc/rfc1321) and the
[CRC crate catalog](https://github.com/akhilles/crc-catalog).

New hash CLI commands read at most 16 MiB from stdin; existing SHA-256/CRC32 behavior
is unchanged. Direct Rust/Python functions hash caller-owned byte buffers with
constant digest state and linear work; callers control their buffer size. All
have explicit version-1 byte-to-text adapters under the existing pipeline limits.
Discover these operations under `hashing`; include/exclude newlines deliberately.

## Binary transformations

New byte transforms accept at most 16 MiB and never interpret text. AND/OR require
equally long operands (empty/empty is valid); no repetition or padding occurs.
NOT flips eight bits per byte. Byte reversal reverses the whole buffer; endian
swapping reverses bytes inside each complete 1/2/4/8-byte unit, keeping unit order.
Both reversals round trip.

Shifts use independent unsigned big-endian 8/16/32/64-bit units (one byte by
default), with no carry across unit boundaries. They zero-fill and discard bits
outside the width. Shift zero is identity, exactly the width yields zero, and
larger shifts or partial units fail. These are fixed-width unit operations, not
signed arithmetic or one unbounded integer over the entire buffer. The existing
XOR remains unchanged. New byte adapters reuse these functions; binary operand
parameters are capped at 64 KiB in pipelines and value-dependent lengths are
checked at execution, while types/parameters/budgets are preflighted.

Use byte-wise repeating-key XOR to inspect a known mask or reverse a challenge
transformation. Applying the same nonempty key twice restores the original bytes.
It is not secure encryption.

```sh
cube binary xor --text 'Hello' --key-hex 6b6579
# 230015070a
```

`--key` supplies UTF-8 key bytes; `--key-hex` supplies arbitrary bytes. Choose exactly
one. Use raw output when chaining binary operations. The descriptor is `binary.xor`.

## Text operations

Letter-only flag Semaphore follows the anatomical LH/RH positions in the
[Marine Corps Museum chart](https://www.usmcmuseum.com/uploads/6/0/3/6/60364049/nmmc_semaphore_flag_booklet_final_1.pdf)
and [Scout Association chart](https://prod-cms.scouts.org.uk/media/5508/generic-resources-semaphore-signals.pdf).
The [ANBG text](https://www.cpbr.gov.au/flags/semaphore.html) differs for I (RH up);
both charts show RH high, so I is `SW,NW`, distinct from K `N,SW`.
No artwork is bundled. Coordinates are receiver-facing: sender LH points toward
screen-right when extended outward. `N,NE,E,SE,S,SW,W,NW` denote eight directions.
Tokens are `LH,RH`, separated by exactly one ASCII space; `/` represents each
literal ASCII space. A-Z letters canonicalize case; digits, other whitespace and
procedural/numeric modes are excluded. J is literal J; D encodes canonically as
`S,N` and also decodes `N,S`. Reversed arms are otherwise not inferred.
Empty input is empty output. Limits are 1 MiB transport and 16384 symbols/tokens;
encoding expands to less than 96 KiB. Two version-1 text pipeline adapters preflight
16384 input bytes for encoding, 98304 for decoding and a 6x/1x output bound.
Existing engine/adapter versions retain their behavior. Clock interpretation is
a separate operation, not part of this alphabet.

Grid factors and rendering retain Unicode scalars, not grapheme clusters. Input
is capped at 1 MiB and 65536 retained scalars; optional Rust White_Space removal
reports removed/retained counts using `WORD_UNICODE_VERSION`. Factors are ascending
width, include both orientations within 16384 per side, and reject excessive
candidate counts (default/hard cap 128). Explicit widths must divide the count.
Placement reuses existing traversal coordinates. Plain text uses JSON-quoted cells,
tabs and LF rows; its bounded scalar count keeps expansion below 1 MiB.
Color rendering requires exactly one explicit RGBA tuple per distinct scalar
(<=256), with no unused/missing/duplicate mappings; PNG construction is reused.
No fonts, inference or pipeline grid value is introduced. CLI mappings use a strict
JSON array of `[scalar, eight Hex digits]`, limited to 8192 bytes; Python uses a dict.

Text statistics count Unicode scalars and overlapping n-grams (`n=1..16`). Defaults
are no normalization, preserved case and all scalars; ASCII-only lowercasing and
letter filtering are explicit choices. Counts sort by descending frequency, then
lexical text order. IC is `sum(c*(c-1))/(N*(N-1))`, undefined below two scalars;
Shannon entropy is `-sum(p*log2(p))` bits/scalar, undefined for empty input and zero
for a constant distribution. Both measures always use single-scalar counts, even
when n-grams are requested. Floating results use f64 (allow numerical tolerance).

Statistics accept 1 MiB UTF-8 input, reuse normalization's 8 MiB output cap, then
limit the retained sample to 1 Mi scalars. Work is limited to 8 Mi scalar-window
units; character and n-gram maps each allow at most 10000 distinct values. Exceeding
these caps fails explicitly. `max_results` limits the sorted listing and `complete`
discloses omitted rows; total windows/distinct counts remain exact. These are
observations, not cipher identification. Structured results use direct APIs/JSON;
no text-statistics pipeline value or lossy string adapter is introduced.

`text.select` extracts first/last Unicode scalars per word or line. Words split
on Unicode whitespace and keep punctuation. Lines follow LF/CRLF boundaries;
bare CR, leading/trailing spaces and punctuation remain data. Empty lines add no
character; `line_first` is an acrostic without trimming. Combining marks and emoji
sequences are scalars, not grapheme clusters. The word tokenizer's Unicode version
is exposed as `WORD_UNICODE_VERSION`, separately from normalization data.
Input/output are bounded to 1 MiB. Positional and every-Nth selection still use
the existing `text.extract` semantics; the new selector also has a text adapter.

Use `text` for explicit normalization, reversal and positional extraction, rather
than treating arbitrary data as UTF-8.

```sh
cube text reverse --text 'Cube 🧊'
# 🧊 ebuC
cube text extract --start 1 --stride 2 --text abcdef
# bdf
```

Positions/reversal use Unicode scalar values, not UTF-8 byte offsets or visual
characters: combining marks and emoji sequences can split. Ordered index lists
may repeat positions; stride selections have an exclusive stop and reject out-of-
range bounds. Nothing is silently trimmed, clipped or normalized.

NFC/NFD/NFKC/NFKD normalization is explicit and may lose distinctions. Check
`cube.UNICODE_VERSION` for the SDK's data version. Text input is capped at 1 MiB;
normalization output at 8 MiB. `text.utf8.encode`/`text.utf8.decode` convert types
without replacement or normalization; invalid UTF-8 fails.

## File forensics

Start with facts about the file, then select a bounded range to inspect or copy.
Use the `file` category for metadata, signatures, hex, strings and differences.

```sh
cube file signature sample.bin --json
cube file strings sample.bin --min-length 4 --json
```

A file signature is a candidate, not proof of format validity or hidden content.
Header rules recognize common PNG/JPEG/GIF/PDF/ZIP/gzip/WAVE/classic-TIFF markers;
embedded scans find candidates at offsets and do not recursively extract them.
An unknown header simply has no supported match. `inspect` reads size/timestamps
and up to 12 header bytes; EXIF requires an explicit request.

Byte offsets start at zero; ranges end exclusively at `offset + length`. Input
must be a stable regular file. Important limits and result meanings:

- Embedded/string scans inspect at most 64 MiB; result listings are bounded.
  Read `complete` rather than assuming all candidates were returned.
- Strings are ASCII bytes 0x20–0x7e, not guessed Unicode. One run is at most 1 MiB;
  accumulated text is bounded to 16 MiB.
- Binary comparisons keep exact difference counts even when a listing is capped.
  Missing tail bytes are absent (`null`/`None`), not zero; differences are not errors.
- Explicit extraction can stream up to 1 GiB; byte-returning APIs cap allocation
  at 16 MiB. Destinations are new files and source aliases are rejected.
- EXIF image metadata (such as camera settings) supports JPEG/classic TIFF with
  a 1 MiB metadata prefix. Tags preserve
  original types and repeated/unknown values; absent EXIF succeeds, malformed
  input fails. Tags may contain sensitive information already in the input.
- Appended-data detection reports exact boundaries for PNG framing/CRCs, baseline
  sequential Huffman JPEG with one or three components, and GIF raster framing.
  JPEG checks segment/table/frame/scan relationships, stuffing and restart-marker
  order; GIF checks palettes, raster rectangles, extensions and sub-block lengths.
  End-marker bytes inside framed payloads do not establish a boundary. JPEG
  progressive/arithmetic processes, GIF plain-text rendering, APNG and unknown
  critical PNG chunks are unsupported. Compressed pixels/entropy codes are not
  decoded. Work to the logical end is capped at 64 MiB/10000 framing records;
  trailing bytes are counted without scanning them and do not establish hidden data.
  The existing PNG report fields and legacy `file.png_boundary` semantic type name
  remain compatible; `validated_chunks` counts framing records for other formats.

### Validate a ZIP candidate

`file zip` / `validate_zip` validates a caller-selected classic ZIP origin. Local
headers, central-directory records, EOCD counts/offsets, names, CRC/size fields and
entry extents must agree. Offsets inside the archive are relative to the selected
origin. Records must be contiguous and nonoverlapping. Stored and deflate methods,
UTF-8 names/comments and signed 32-bit data descriptors are supported. Encrypted,
ZIP64, multi-disk, unsigned-descriptor and alternate self-extracting offset variants
fail explicitly. Names remain inert exact `name_hex` data; nothing is unpacked.

Validation admits at most 16 MiB beyond the selected origin, 64 EOCD candidates,
1000 entries, 10000 cumulative local/central header visits, 1 MiB metadata and
64 MiB total declared uncompressed data. `scanned_bytes` and
`candidate_scan_complete` expose whether the candidate suffix was fully searched;
competing interpretations outside that prefix are not ruled out. `payload_verified`
is always false: compressed content, actual CRC integrity and authenticity are not
verified. `file appended` reuses this same validator for a ZIP at offset zero.
Boundary/carving requests reject incomplete ZIP searches instead of choosing a
potentially ambiguous extent. ZIP is available through direct APIs rather than
the current pipeline report subset.

### Recover selected embedded bytes

`file carve` / `carve_file` requires an explicit offset and a new destination.
Omit length to revalidate a supported PNG/JPEG/GIF/ZIP candidate at that origin;
supply length to copy an exact raw range without claiming format validation.
Candidate validation uses relative reader coordinates, while the report keeps
absolute source offsets. ZIP candidate carving requires a complete bounded suffix
search; if that is not possible, choose an explicit known range instead.

The output budget defaults to 16 MiB and can be lowered or raised to the existing
1 GiB streaming-copy hard limit. Range/size/format checks happen before output
creation; existing destinations and source aliases are rejected. Copying reuses
the 64 KiB streaming range API. A new partial output is removed on failure.
The report includes source size, selected range, bytes written, selection mode,
format/evidence when validated, and SHA-256 of the exact recovered bytes. This
identifies content, not trust. Source size/mtime changes during recovery fail;
callers must keep inputs stable because this is not an atomic filesystem snapshot.
No paths or bytes are automatically retained in investigation sessions. Nothing is
decompressed or executed, and source files are opened read-only.

## Image analysis and steganography

Use `image` operations to inspect PNG structure, compare exact samples or extract
selected channels/bits. The [CLI fixture](cli.md#create-small-image-and-audio-fixtures)
provides an original image with known low bits for these examples.

```sh
cube image histogram sample.png --channels red,green,blue --json
cube image bit-plane sample.png --channel red --bit 0 --stdout > plane.bin
```

PNG decoding preserves 8-bit grayscale, grayscale+alpha, RGB, RGBA and indexed samples.
Additional inputs are baseline Huffman JPEG (gray/RGB output), single-frame GIF
(RGBA canvas, transparent background), BMP (decoded RGB/RGBA), and static WebP
(RGB/RGBA). JPEG color-space conversion and lossy artifacts are decoder results,
not exact encoded/source samples. GIF/BMP palettes are expanded by their codecs.
`Image.provenance()` and `image inspect --json` label these representations.
Constructed and transformed results are labeled `constructed_8bit_samples`, without
claiming an encoded original or preserved source samples. For compatibility,
`Image.info().format` still says `png` for these objects' lossless export model;
use provenance for their actual origin. A subsequent decoded PNG describes that
new immediate PNG source, not its earlier transformation history.
Limits include 32 MiB encoded data, 16384 pixels per side, 4 Mi pixels and 16 MiB
of decoded samples. Decoder allocation requests are capped at 64 MiB where the
codec supports them; dimensions/output and encoded-input limits always apply.
GIF decoding examines at most two bounded frames to reject animation; animated
WebP is rejected from its header. Other formats/depths fail explicitly. No
ICC/gamma correction, EXIF orientation transform or premultiplication occurs.
GIF's first frame is placed on its transparent canvas. PNG chunk inspection checks
framing/CRCs without decoding pixels; bad-CRC reporting and payload export are opt-in.

Coordinates use a top-left origin; regions are `x,y,width,height`, entirely inside
the image. Selected channel order is preserved. Native indexed data stays palette
indexes; request expansion explicitly for RGBA. Histograms count exact selected
color tuples in sorted order, including alpha only when selected. Exceeding a
color budget fails, rather than approximating. Coordinate listings cap at 10000
pixels. Comparison requires equal dimensions and compatible selected samples.

`image.bitwise` combines two images with an explicitly selected `xor`, `and` or
`or`, reusing the byte transformations. Dimensions, 8-bit model/channel order,
palette and raw tRNS must match exactly; unequal sample buffers never cycle or pad.
Every physical sample participates, including alpha, without blending. Indexed
values combine as indices and must remain valid for the retained palette;
explicitly expand both inputs first when RGBA color values are intended.
Matching tRNS keys remain unchanged: output colors matching the retained key are
transparent. Other source metadata is dropped; provenance describes constructed
samples, not original compressed bytes. XOR can be reversed with the same second
image when all intermediate images remain valid; AND/OR are lossy.

The existing per-image pixel/dimension/sample limits apply to both inputs and the
result, so their sample buffers total at most 48 MiB. Decoder and PNG export limits
also apply. PNG export is lossless for resulting samples and creates a new file.
This operation is available through Rust, CLI and native Python, with no pipeline
adapter or new image pipeline value. Existing byte XOR and image comparison keep
their behavior.

`image.pairs` enumerates unique `(i,j), i<j` in caller order, reusing bitwise or
exact comparison APIs. Empty/single collections yield zero pairs; repeated object
identities fail (CLI also rejects duplicate canonical paths). Hard limits are eight
inputs, 28 pairs, 64 MiB retained input samples, 16 Mi pixel-pair work, 64 MiB
conservative PNG output and 1000 global difference records. All pairs, including
failed compatibility checks, count toward preflight. PNG bounds use the same
conservative per-image formula as color exports. The compact manifest is capped
at 512 KiB. Counts remain exact when difference listings are incomplete.
Pair errors do not discard other results; filesystem writes reuse the CLI export
reservation/rollback policy. No generic dispatcher or pipeline collection is added.

`image.colors.export` reuses the complete native-channel histogram and existing
single-color extraction. It returns canonical `color-<hex>.png` entries and counts;
absent or duplicate subsets fail. Native indices and physical alpha distinguish
colors until explicit palette expansion. Both color/output caps default to 16
(hard cap 64). Histogram plus extraction scans allow 16 Mi pixel-work; conservative
PNG bounds of `2*sample_bytes+4096` per image must total at most 64 MiB before work.
Actual encoded bytes are also checked. Rust/Python perform no implicit writes;
CLI reserves the complete collection plus manifest and rolls back its new files
on failure. This structured collection has no pipeline adapter.

Raw reconstruction supports unsigned 8-bit `gray`, `rgb`, `bgr`, `rgba` and `bgra`.
Width, height and physical layout are mandatory; offset defaults to zero and stride
to a packed row. Every row includes its declared padding, including the final row:
input length must equal `offset + height * stride`. Extra or missing bytes fail.
Only prefix and explicitly declared row padding are ignored. No byte-order option
is needed for one-byte samples. The ordinary image/input budgets apply. The result
uses canonical gray/RGB/RGBA samples, with lossless PNG export to a new file.
Raw images use direct APIs; no pipeline image value is introduced.

Alternative geometry is always a caller hypothesis. `image raw --compare-header`
and `Image.inspect_layout` report parsed header dimensions separately from the
validated raw candidate. Header-only evidence does not validate compressed data;
malformed recognized headers report a separate error, while unknown data has no
declared dimensions. PNG/JPEG/GIF/BMP/WebP header observations do not repair files.
Optional export creates a new PNG from the exact candidate layout.

PNG depths 1/2/4/16 remain unsupported by `Image`. The
[precision evaluation](https://github.com/cubearc/cube/issues/70) records native
packed-index/gray and big-endian 16-bit decoder fixtures, a compatibility strategy,
budgets and a proposed future representation. Implementation is gated on agreement
of that contract; no sample expansion/downsampling or public type migration occurs.

Exact-color selection matches complete native tuples, including physical alpha.
Select 1..=256 distinct colors. Masks preserve coordinates and use 255 for selected
pixels, zero elsewhere; RGBA layers keep selected original colors/alpha (including
tRNS) and use transparent black elsewhere. Transparent selected pixels still count
and retain their color bytes. Native indexed selection matches indices; explicitly
expand the palette to select RGBA colors, which can match multiple duplicate palette
entries. No fuzzy matching or hidden-message inference is performed.

Rotation, mirroring and channel permutations rearrange samples reversibly;
inversion preserves alpha unless selected. Cropping discards pixels and grayscale
conversion loses color. Saved PNGs retain samples/palette/transparency but drop
other metadata: sample-preserving does not mean byte-identical source files.

A bit plane returns one 0/1 byte per selected pixel; optional packing uses MSB-first
bytes, and visualization maps 0/1 to 0/255 explicitly. Bit 0 is a sample's least
significant bit (LSB). Configurable LSB extraction visits pixels, then requested
channels, then requested bit positions. It defaults to bit 0,
row-major/forward traversal, native channel order and MSB-first packing. Reversal
changes pixel order only. Incomplete bytes fail unless explicit drop/pad is chosen.

Traversal also supports `diagonal`, `zigzag`, `spiral_cw` and `spiral_ccw`.
Diagonals visit increasing `x+y`, from bottom-left to top-right within each
anti-diagonal; zigzag reverses odd anti-diagonals. For a 3×3 image, zigzag visits
row-major indices `0,1,3,6,4,2,5,7,8`. Spirals proceed inward clockwise or
counterclockwise, starting rightward or downward respectively from the top left.
`start_corner` defaults to `top_left`; other corners reflect axes for linear/
diagonal traversal, or rotate the starting edge while preserving a spiral's named
direction. Regions retain absolute coordinates. Reverse reverses the final pixel
sequence only. Coordinates are generated with constant memory under image limits.

Pixel export uses the same traversal and corner options as LSB extraction, visiting
each coordinate before its selected channels. Outputs are raw sample bytes;
`--hex` renders exactly those bytes. Coordinate records retain source positions.
Existing native row-major pixel export remains unchanged. PNG channel visualization
requires default row order; other orders use bytes or records. Feed exported bytes
to existing byte transformations/pipelines explicitly. Encoded-image decoding is
kept outside recipes: image expansion/work budgets differ from byte adapters, and
discovery does not advertise an image pipeline adapter.

QR analysis accepts opaque supported images up to 262144 pixels and 1024 pixels
per side. A temporary view expands palettes, reuses the integer luma conversion,
then uses Quircs' Otsu thresholding. Transparent samples require explicit caller
preprocessing; no background is guessed. Detection bounds are 32 capstones,
65534 regions and eight grids; results flag reached detector limits and do not
promise exhaustive detection. `max_symbols` is 1..=8 and total payload is capped
at 64 KiB. Known fixtures cover two/four/eight pixels per module and quarter turns;
other scale, perspective, damage and contrast conditions may fail.

Reports distinguish no detection, detected but undecodable, decoded and partial
decode. Each symbol has raw payload bytes or a decoding error, plus available
corner/version/ECC/mask/type/ECI metadata. Binary bytes are preserved without text
encoding guesses. CLI JSON represents them as exact Hex; raw output requires
exactly one decoded symbol. No payload URL is followed or content executed.
Selections must be nonempty and distinct; output is capped at 16 MiB.
An extracted bit stream is a deterministic candidate, not proof of a hidden message.

## Symbolic encodings

Use `symbols` for textual International Morse or reversible Unicode Braille cells.
These operations do not detect symbols from audio or images.

```sh
cube encode morse --text 'SOS 2'
# ... --- ... / ..---
cube decode braille --text '⠀⠁⣿'
# 0001ff
```

Morse supports A–Z/0–9; encoding accepts lowercase and canonicalizes ASCII spaces.
Decoding returns uppercase, with one space between tokens and ` / ` between words.
Unsupported punctuation, noncanonical spacing and trailing newlines fail.
Braille represents raw eight-dot masks, not natural-language Braille transcription:
bit `n-1` is dot `n`, and a byte maps to U+2800 plus its mask. Every byte round trips.
Both accept at most 1 MiB input; canonicalized Morse case/spacing does not round-trip.

A1Z26 maps ASCII A–Z to canonical decimal 1–26 tokens joined by hyphens within
each word: `Hi Cube` becomes `8-9 3-21-2-5`. Space and ASCII control whitespace
TAB/LF/VT/FF/CR are preserved exactly, including repeated or boundary whitespace.
Decoding returns uppercase. Punctuation, non-ASCII, signs, leading zeros, empty
hyphen tokens and numbers outside 1–26 fail at their byte offset. Encoding accepts
1 MiB and produces at most 3 MiB; decoding accepts 3 MiB. Uppercase supported
inputs round trip; lowercase case is deliberately lost. Both have bounded text
pipeline adapters, subject to the existing smaller pipeline value/work budgets.

## Numeric conversions

Use `numbers` to change an integer's textual base, separately from byte encodings
and byte order.

```sh
cube number base --from-base 16 --to-base 2 --text FF
# 11111111
cube number base --from-base 36 --to-base 10 --text=-Z
# -35
```

Bases are explicit 2..36. Digits use ASCII 0–9/A–Z; a leading sign is allowed,
while whitespace, radix prefixes, separators, fractions and floats are rejected.
Magnitudes up to `2^128-1` are exact, with at most 4096 input digits. Output
canonicalizes case, sign and leading zeros; the integer value is preserved.

The v0.11.0 numeric evaluation is recorded in [issue #77](https://github.com/cubearc/cube/issues/77).
Prime tests, bounded Fibonacci and modular arithmetic remain deferred; base
conversion is the implemented numeric API. No proposed operation is advertised
through discovery.

## Audio analysis

`audio.spectrum` analyzes exactly N selected integer PCM frames with a power-of-two
FFT in 8..16,384. It never truncates or pads implicitly. Each selected channel is
independent, in caller order; frequencies are `k*sample_rate/N`, including DC and
Nyquist. Samples use existing signed/centered PCM decoding divided by `2^(bits-1)`.
Rectangular, periodic Hann (default) and periodic Hamming windows are available.
Magnitudes divide by the window sum and double interior bins, giving one-sided
peak full-scale amplitude, not power density or dB. Silence gives zero, and empty
or short selections fail. Numeric results agree within floating-point tolerance;
they are not guaranteed bit-identical across platforms.

At most 65,536 channel/bin cells and 4 Mi `channels*N*log2(N)` work units are
allowed. Existing PCM input/range/sample limits apply first. RustFFT is used without
optional SIMD features or a native backend. Results are structured Rust/Python
numeric arrays and CLI JSON. No pipeline adapter or new pipeline value is added
for sample blocks; existing audio APIs and recipes remain unchanged.


`audio.spectrogram` reuses these FFT/window/amplitude rules. Its matrix is
`[channel][window][bin]`; `frame_starts` and `times_seconds` refer to absolute
source window starts, not centers. Choose `hop_size` in 1..N. `final_window=drop`
(default) keeps full windows and reports the uncovered tail; `pad` visits every
hop start before end and zero-fills tails using full-window normalization.
`valid_frames` records how much source data each window contains. Empty input
produces no windows; short input produces no windows with drop. The same 65,536
cell/4 Mi work budgets cover the entire matrix, checked before FFT planning.
No plotting dependency or second FFT implementation is added.

`audio.bits.extract` visits selected frames, selected channels in their caller
order, then requested unique bit positions. Reverse traversal reverses frames
without reversing channel groups. Bit zero means the native storage LSB, including
unsigned 8-bit PCM and two's-complement 16/24/32-bit PCM; little-endian storage
never gets amplitude centering, sign extension or normalization for extraction.
Packing reuses the image MSB/LSB and reject/drop/zero-pad rules and reports available,
valid and dropped bit counts. It allows 16 Mi visited bits/2 MiB output, in addition
to existing PCM bounds. Output is exact bytes, never guessed text. The same shared
packing types remain available at their existing image API paths.

`audio.reverse` generates a new minimal PCM WAV containing only fmt/data chunks.
It reverses time frames while preserving selected channel order, rate, integer
width and native sample bytes; it never reverses bytes within samples. Reversing
twice restores sample data. Source ancillary metadata is omitted, so this is not
a byte-for-byte container round trip. Existing PCM limits bound allocation to
8 MiB plus the WAV header/pad; sizes and byte rate are checked before writing.
The CLI creates a new file exclusively and removes a partial output on write
failure. The SDK returns new WAV bytes and performs no implicit file write.

DTMF analysis evaluates the nominal row frequencies 697/770/852/941 Hz and
columns 1209/1336/1477/1633 Hz from [ITU-T Q.23](https://www.itu.int/rec/T-REC-Q.23/en).
It requires exactly one selected channel and a sample rate of at least 8 kHz.
Bounded Goertzel measurements use nonoverlapping rectangular windows; frequency
offsets, speech, distortion and boundary mixtures can be inconclusive. This is
signal-analysis evidence, not a certified telephony receiver or confirmed string.

Defaults: 40 ms windows, minimum 80 ms tones/40 ms gaps, RMS level >=-40 dBFS,
6 dB dominance over the second row/column, <=8 dB absolute twist and >=75% pair
energy ratio. All thresholds are explicit advanced parameters in discovery/help.
Partial tone windows, ambiguous bands, excessive twist, short runs and insufficient
gaps retain reasons and uncertainty. Silence has a separate state. Events retain
candidate symbols and measured windows; only runs meeting all selected thresholds
have a non-null `symbol`, still labeled `candidate`. Timing tolerance is one window.
Limits are 262,144 frames and 4,096 windows, with one fixed set of eight frequency
measurements per frame. There is no automatic frequency search or device access.

Morse-like analysis has two separate operations. `audio.morse.detect` measures a
caller-supplied nominal frequency (>=100 Hz, below Nyquist) on exactly one channel
at >=8 kHz. Defaults are 10 ms rectangular windows, -40 dBFS minimum RMS and 80%
nominal-tone energy ratio. Below-level windows are silence; other audible or
partial windows without enough expected tone energy remain uncertain. Adjacent
same-state windows merge into exact absolute frame intervals. Resolution is one
window, with the same 262,144-frame/4,096-window budget as signal analysis.

`audio.morse.transcribe` separately requires a dot duration in 40..1,000 ms and
relative tolerance in 0..40% (default 20%). It uses 1/3-unit tones and 1/3/7-unit
internal gaps from [International Morse timing](https://www.itu.int/rec/R-REC-M.1677/en).
Boundary silence is ignored. Invalid/noncontiguous/overlapping interval ranges
fail; uncertain timings or ambiguous matches return no text. Recognized tokens
reuse Cube's existing Morse text decoder. Unsupported token sequences preserve
tokens with `unsupported_tokens` and no invented letter. No automatic frequency
search, speed estimation, radio capture or forced transcription occurs.

`audio.metadata.inspect` supports native FLAC STREAMINFO and one optional
Vorbis-comment block through Lofty. MP3, Ogg, ID3-prefixed FLAC and reserved block
types are unsupported. It reports declared sample rate/channels/bit depth, duplicate
key/value comments, block offsets and duration at millisecond precision. Zero
total samples means unknown duration; an absent/all-zero declared MD5 is null.
The audio payload and MD5 are unverified, so a metadata result is not proof of a
valid audio stream. Artwork and noninterpreted blocks are never expanded.

Limits: 1 GiB physical source, 1 MiB total metadata, 256 blocks/comments, 64 KiB per
vendor/comment field and 16 KiB exported text. Framing, extents and comment lengths
are preflighted before maintained parsing; duplicate STREAMINFO/comment blocks,
truncation and invalid text fail. The operation reads no audio payload, performs
no transcoding/decompression and changes no parser-global settings. Lofty's
maintained default feature configuration is required by an upstream build issue;
Cube's native-FLAC-only entry rejects ID3 before parsing, so compressed-tag code
is unreachable. Format rules follow [RFC 9639](https://www.rfc-editor.org/rfc/rfc9639).
This separate operation preserves the existing integer-WAVE `audio inspect` API.

Use `audio` for RIFF/WAVE metadata, exact channel/frame samples and min/max waveform
data. PCM stores sample amplitudes directly, without a compressed audio codec.
Start with the [synthetic WAVE fixture](cli.md#create-small-image-and-audio-fixtures).

```sh
cube audio inspect sample.wav --json
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```

Supported audio is little-endian integer PCM RIFF/WAVE (format tag 1), with
8/16/24/32-bit samples and 1–32 channels. Compressed/float/extensible WAVE, RIFX,
RF64 and multiple data chunks are unsupported. Metadata inspection skips payloads;
physical input is capped at 1 GiB and 10000 chunks.

A frame contains one sample per channel. Selections use zero-based absolute frames
and ordered unique channel indexes. Raw output preserves original little-endian
sample bytes; 8-bit storage is unsigned, while interpreted values subtract 128.
Wider samples use signed two's complement. There is no resampling, normalization
or downmix. Raw PCM output is not a WAVE file; retain its interpretation metadata.

Sample extraction caps selections at 1048576 frames, 300 seconds and 64 MiB source
bytes, with 8 MiB raw output. Structured value listings cap at 10000 samples.
Waveform buckets contain exact min/max signed amplitudes, absolute start frames
and the final partial bucket; the sample rate supplies the time base. Budgets
bound channel/bucket pairs. This is data, not a plot, spectrogram, FFT, Morse/DTMF
detector or playback feature. Video container inspection is described below.

## Compose operations

Pipelines are implemented linear sequences over explicit bytes, UTF-8 text and an
inert structured byte range. Direct Rust/Python functions keep their normal types.
`cube ops list --json`, `cube ops describe` and the Python discovery functions
report `pipeline`: an adapter contract version object when supported, or null
(`None` in Python) otherwise. `cube pipeline adapters` /
`cube.pipeline_adapters()` lists the same executable subset. Current adapters cover
bounded encodings, classical ciphers, hashes/checksums, XOR and UTF-8 conversion.
Selected in-memory file inspections also have adapters: `file.signature.detect`,
`file.hex.inspect` and `file.strings.extract`. They accept bytes and return a
`forensic_report` with a concrete `signatures`, `hex` or `strings` result. Adapter
listings declare their pipeline input/output types separately from the direct
reader API's descriptor. Hex and strings reuse checked offsets/lengths; omitted
length selects through the end of the buffer (unlike the CLI file hex preview).
Other file operations and image/audio APIs require direct calls.

Reports preserve numeric offsets, signature evidence, completeness and exact
bytes. JSON hex rows encode their `bytes` field as canonical base64; native Python
uses `bytes`. They are inert observations, not file-validation proofs. No transform
currently consumes reports; compatible byte transformations can precede them and
captures can retain any selected step. Report input supports identity; JSON inputs
retain the existing document-size limit. Use JSON output for reports; raw output
is rejected. Conservative budgets include record overhead and may reject a result
that would be smaller for particular data.

The current catalog's operations are available through direct Rust functions or
methods, CLI commands, and native Python functions or methods. The descriptor's
`command` identifies its CLI route; Python stubs describe SDK entry points.
This includes orchestration such as recipe execution and investigation sessions,
which is available directly but cannot itself be nested as a pipeline adapter.
Read-only file APIs still require caller-selected I/O; discovery grants no file
permission. Signature detection, hex rows and printable strings can also inspect caller-supplied
buffers in a pipeline without acquiring file access.

Every step declares an operation identifier, exact contract `version`, and optional
parameters. The entire plan validates before transformation. Adjacent types must
match: a hash's text output is not silently converted to bytes for the next step.
Use explicit UTF-8 conversion operations when that is intended. Errors stop at the
first failing step and identify its zero-based index. An empty pipeline is identity.

JSON values use explicit tags: `{"type":"bytes","base64":"AP8="}`,
`{"type":"text","data":"Hello"}`, or
`{"type":"byte_range","offset":3,"length":4}`. Binary parameters use
`{"key":{"base64":"/w=="}}`. Range length may be omitted/null, checked offsets
cannot overflow, and a range grants no file access. Only identity currently accepts
range values. Parameter scalar types are not silently coerced.

Hard pipeline bounds are 64 steps, 1 MiB per value, 256 KiB parameter data,
16 MiB conservative work and 4 MiB captured intermediates. Lower these with
`max_steps`, `max_value_bytes`, `max_work_bytes` and `max_capture_bytes` in the
limits object. Work counts initial input and each step's input/output upper bounds; ranges count
as 16 logical bytes. Conservative bounds can reject a data-specific smaller result.
Base58 adapters additionally cap step input at 4096 bytes. Captures are explicit,
unique zero-based step indexes. Cancellation is cooperative between bounded steps,
not a hard timeout. No shell, network, native loading or arbitrary file capability
is granted. See [CLI](cli.md#run-a-pipeline-or-recipe) and
[Python](python.md#compose-a-pipeline-or-recipe) examples.

### Recipe files

Recipes make a validated pipeline reusable. Save this original round-trip example
as `roundtrip.json` in your working directory:

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

Format 1 uses JSON, one symbolic input slot or one tagged inline input, an explicit
output type and ordered steps. Slots use 1..64 ASCII letters/digits/underscores/
hyphens. Missing/extra bindings are errors; inline recipes accept none. An inline
input instead looks like `{"source":"inline","value":{"type":"bytes","base64":"AP8="}}`.
Bindings are caller-supplied values, never implicit paths, stdin or environment reads.

Format version, engine API and each adapter version are exact compatibility
contracts, independent of package versions. Matching operation identifiers alone
is insufficient. Unsupported versions are rejected; there is no automatic migration.
Optional `limits` uses the pipeline's lowered budgets.

Recipe/JSON transport is limited to 256 KiB, depth 16 and 4096 value nodes. Inline
bytes and parameters count toward the document limit. Duplicate keys, unknown
fields, noncanonical base64 and invalid types fail; there are no includes,
expressions, executable tags or privileged capabilities. Validation never executes
algorithms. `recipe check` / `check_recipe` compares exact typed output so community
recipes can test compatibility independently. A different expected value must fail.
The engine owns contracts; community definitions/examples live in
[cubearc/recipes](https://github.com/cubearc/recipes), without duplicated algorithms.

## Investigation profiles and records

Use `investigation` to decide what to inspect next and record reproducible checks.
Triage reads up to 12 header bytes and cites stable rules, evidence/offsets and
suggested available operations. PNG suggests chunks, JPEG/TIFF suggests EXIF,
WAVE suggests stream metadata and unknown bytes suggest hex/strings. Suggestions
are not automatically executed and never establish concealed content.

Profiles have two depths: metadata performs triage; preview adds a 256-byte hex
view and strings from the first 4096 bytes (minimum length 4, at most 32 results).
Optional identity hashes the complete input; only an admitted prefix is otherwise
buffered. For a reusable profile, save this as `preview.json`:

```json
{
  "format_version": 1,
  "depth": "preview",
  "identify": true,
  "budgets": {"max_operations": 4, "max_elapsed_ms": 5000}
}
```

Only `format_version` is required. The default depth is metadata; identity is off.
Default budgets allow 1 MiB source buffering, 4 MiB charged inspected ranges,
16 operation units, 64 KiB retained results and 5000 ms cooperative time.
Hard operation/time ceilings are 64 units and 60000 ms; other limits can only be
lowered from those defaults. Budget fields are `max_input_bytes`,
`max_inspected_bytes`, `max_operations`, `max_retained_bytes`, `max_elapsed_ms`.
Repeated source ranges are charged again; a pipeline wrapper and each step consume
operation units, while pipeline value/work limits still apply. `plan` exposes the
allowlist, ranges, budgets, capture bounds and skipped reasons without content reads.

Optional `pipeline` contains `steps` and lowered `limits` from the composition
contract. `retain_results:[2]` selects in-memory results by session-local run ID;
`record_parameters:true` opts into pipeline parameter retention. Captures use
conservative bounds, so a small actual result can still be skipped. Deadlines and
cancellation are checked around reads/operations, without preemption; elapsed time
is diagnostic and excluded from deterministic records. Read partial/skipped/failed
outcomes rather than treating budget exhaustion as a complete investigation.

Session records use `schema_version:1`, `status`, `replay_gaps`, `uncertainty` and
`session`. The session has format/engine versions, input identity, limits, runs,
findings, separately labeled hypotheses and optional profile context/counters.
Findings trace to an input/run/offset. Raw input contents, paths and timestamps are
not saved. Default export also omits selected result contents, nonempty parameters
and hypotheses; retention and export are independent explicit choices.

Records cap at 128 KiB, retained result JSON at 64 KiB, runs at 64, findings at 128,
and hypotheses at 32 (1024 UTF-8 bytes each). Direct session/replay input is at most
1 MiB. Loading validates IDs, references, states and retained-result digests.
Human reports escape untrusted strings. Saving requires a caller-selected new
file; remove it with normal filesystem tools. No hidden session database or copies
are created. Omitted/redacted data cannot be recovered from a saved record.

Replay validates all contracts and required parameters, checks caller-supplied
size/SHA-256 identity, then compares reproduced typed result digests/states. Size
alone cannot establish identity; missing digests, redactions, incompatible versions
or changed inputs are explicit failures. Skipped runs stay skipped. Hypotheses are
not replayed as machine evidence. Digests identify content, not trust, and reports
are not signed attestations. See [CLI](cli.md#record-an-investigation) and
[Python](python.md#keep-an-investigation-record) for executable workflows.

## Video container metadata

`video.container.inspect` reads declared metadata from unfragmented MP4/MOV and
WebM without a decoder. Track/codec IDs, coded dimensions, time bases and durations
are observations; media packets, codec configuration and checksums are unverified.
MP4 display dimensions retain the original 16.16 values separately from coded
sample-entry dimensions. Integer durations/time bases remain exact; derived
seconds use floating point. Absent durations and other unavailable fields are null.

The MP4/MOV subset supports version 0/1 movie/media/track time headers, ordinary
and extended atom lengths, and one sample description per track. Handler metadata
must precede sample descriptions. Fragmentation, encrypted sample entries,
versioned QuickTime audio entries and zero-sized nested atoms are unsupported.
WebM requires its EBML DocType, Info and Tracks, supports known-sized child
elements, and permits a Segment sized to the source end. Unknown-sized children,
content encodings/encryption and non-default track timestamp scaling are unsupported.
Unknown bodies and media payloads are skipped by checked extents, not validated.

Limits are 1 GiB physical input, 10,000 records, 32 tracks, depth 8, 2 MiB cumulative
metadata reads and 4,096 bytes per field (128 for WebM identifiers). Only known
header paths are traversed. Small framing code enforces these limits without
allocating full sample tables/packets or adopting a decoder/demuxer runtime.
Format references: [QuickTime](https://developer.apple.com/documentation/quicktime-file-format),
[EBML](https://www.rfc-editor.org/rfc/rfc8794) and
[WebM](https://www.webmproject.org/docs/container/). This reader operation has no
pipeline adapter; it does not grant a recipe file access or imply extraction support.

`video.frames.extract` is a separate, explicit MOV/ISO-BMFF baseline-JPEG (`jpeg`)
extractor. Select one track ID and 1..=8 unique zero-based indexes in caller order.
It validates self-contained version-zero descriptions/data references, stsz/stsc/
stco or co64/stts relationships, exact decode times and all sample ranges within
mdat without overlap. Optional stss must identify every JPEG as a sync sample.
Edit lists, composition offsets, fragmentation, encryption, external references,
description extensions and other sample-table features are unsupported. Pixel
coordinates are stored JPEG coordinates; track display matrices are not applied.

The combined inspection/index pass caps atoms/table rows at 10,000, metadata reads
at 2 MiB and samples at 10,000. Input is at most 1 GiB. Only selected encoded JPEGs
are read: <=2 MiB each and <=16 MiB total. Existing baseline framing/header/decode
limits apply; JPEG framing must fill the sample and dimensions must match the
description before decoding. Conservative RGBA-size bounds are <=16 MiB/frame and
<=64 MiB total decoded samples; the existing decoder has a 64 MiB internal limit.
No intervening frames are decoded. Cancellation occurs between calls, not during
a codec call; these explicit work bounds do not claim a wall-clock timeout.

Each result contains an existing Image and track/index/decode-start/duration/time
base/source offset/length/encoded-SHA-256 provenance. Exact ticks retain timing;
there is no timestamp-seek API or playback validation. JPEG pixels are decoded
lossy-source data. Exported PNG preserves those pixels, not original JPEG bytes.
CLI batches reserve all new names exclusively, cap PNG output at 96 MiB and remove
only files created by a failed batch. SDK extraction performs no writes. Images
and reader I/O remain outside the current pipeline value/adapter contract.

`video.frames.compare` selects up to eight explicit frame pairs from that same
MOV/JPEG subset. It extracts their union once (at most eight images), then reuses
`Image::compare` for exact native samples or explicit channels. Pair reports retain
both frames' exact extraction/time/hash provenance. Different dimensions fail;
there is no implicit resizing, display transform, fuzzy metric or perceptual claim.
The global difference-list budget is 1..=10000 records (default 1000); exact counts
remain available and each listing declares completeness. Work is capped at 8 Mi
compared pixel pairs, with no mask allocations. Existing image APIs compare
previously extracted PNGs without a decoder; retain their extraction manifest.
Rust `comparison::compare_selected` accepts supplied Frame records and labels their
provenance as caller data rather than authenticating it. Reader/image operations
remain direct APIs, with no pipeline adapter or changed recipe contract.

`video.audio.extract` remuxes one explicitly selected self-contained MOV/ISO-BMFF
`raw ` PCM8 mono/stereo track to minimal RIFF/WAVE. The version-zero 28-byte sound
description must declare unsigned 8-bit samples, zero compression/packet fields,
an integer sample rate 1..=65535, fixed stsz width equal to channels, delta-1 stts
and time scale equal to sample rate. Other legacy table conventions, description
extensions, codecs, widths, edits, composition offsets, fragmentation, encryption
and external references fail explicitly. This focused subset follows the
[QuickTime sound description](https://developer.apple.com/documentation/quicktime-file-format/sound_sample_description_version_0).

Shared indexing validates all selected-track sample ranges and relationships
before copying. Limits are 1 GiB physical input, 2 MiB metadata reads, 10,000
cumulative atoms/table rows, 1 Mi sample frames, 300 seconds and 64 MiB temporary
index memory. Coalesced source ranges stay in sample order (up to 10,000 records),
so physically reordered chunks remain reproducible without a seek/read per frame.
PCM is capped at 8 MiB and output at that size plus the WAV header/pad. Samples,
channel grouping and rate remain unchanged; there is no decoder, normalization,
resampling or byte conversion. Source ancillary chunks are omitted.

The result includes WAV bytes plus track/format/exact-time/source-range/PCM-hash/
WAV-hash provenance. CLI output requires a new file or explicit binary stdout;
SDK extraction performs no writes. Existing audio APIs consume the generated WAV.
The operation has no pipeline adapter, privileged recipe I/O or new engine type.
Reader inputs must remain stable during inspection/extraction; Cube does not make
a full-file snapshot or claim that unselected media packets are valid.

## Offline web artifacts

`web.html.inspect` accepts UTF-8 HTML as inert data. A maintained tokenizer reports
selected src/href/poster/data/action references, event-handler counts, tokenized
text excluding script/style, and exact source ranges for inline script/style.
References use HTML character decoding; text is tokenized, not rendered visibility.
It is not a DOM, browser-grade repair algorithm or full CSS/srcset parser. Duplicate
attributes follow the tokenizer's first-value rule and increment parse errors;
unterminated inline bodies are labeled. Recoverable HTML errors are observations.

HTML limits: 1 MiB input, 10,000 tokens/attributes, 64 attributes/tag, 128 syntactic
nesting, 64 recoverable errors, 256 reference/inline records, 64 KiB fields/body/text
and 128 KiB aggregate exported text. No script/event handler executes, no URL is
followed, no relative path is opened and no assets are downloaded. Parent path
segments, control characters or other schemes are observations, not threat claims.

`web.har.inspect` reads a documented HAR 1.2 request/response metadata subset.
It reports method/URL/status, optional MIME/size/time/date fields and body
availability/encoding, without exporting headers, cookies, post data or bodies.
Unknown values are null. A single leading UTF-8 BOM is accepted. Existing bounded
JSON parsing rejects duplicate keys, depth >16, >4,096 nodes or input >256 KiB;
HAR adds <=128 entries, <=64 KiB strings, <=256-byte keys, <=8,192-byte URLs and
<=128 KiB exported text. This subset is not complete HAR conformance validation.
Browser credentials and referenced files are never accessed. Both inspection
operations have no pipeline adapter or privileged I/O in their core functions.

`web.html.asset` selects a zero-based inline script/style record or reference from
that inspection. Inline selection preserves exact source UTF-8 bytes and requires
an explicit closing tag. `selector=data_url` accepts only strict standard-base64
data URLs after HTML character decoding; paths, remote URLs, percent decoding and
non-base64 data URLs are unsupported. `web.har.asset` selects an entry with an
embedded `response.content.text`. Base64 bodies yield decoded bytes; omitted
encoding yields UTF-8 of the logical JSON string, which may differ from original
HTTP wire bytes. Unknown encodings and external-body references fail. No automatic
decompression, content interpretation or filesystem/network resolution occurs.

Selections return bytes plus artifact SHA-256, record/pointer, source range where
available, representation, size and output SHA-256. Output is at most 64 KiB and
all inspection budgets still apply. CLI output must explicitly use `--stdout`,
`--output` (exclusive new file) or `--json` (exact Hex plus provenance). The two
asset operations have version-1 text-to-bytes adapters that reuse the existing
pipeline engine, value kinds, preflight, captures and lowered budgets. Adapter
output is bytes; use direct extraction first when provenance must be retained.
Caller-selected local files can enter the ordinary byte pipeline separately;
artifact references never authorize opening them. Existing recipes remain valid.
