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

## Hashing and checksums

Use SHA-256 to identify content with a one-way digest; it cannot be decoded into
the original input and is not password hashing or authentication. CRC32 is a
checksum for accidental corruption, not a cryptographic hash.

```sh
cube hash sha256 --text 'Hello'
cube hash crc32 --text '123456789'
# cbf43926
```

Both consume exact bytes and return lowercase hexadecimal text. Cube's CRC32 is
CRC-32/ISO-HDLC (IEEE), with eight digits including leading zeros. Discover these
operations under `hashing`; include/exclude newlines deliberately.

## Binary transformations

Use byte-wise repeating-key XOR to inspect a known mask or reverse a puzzle
transformation. Applying the same nonempty key twice restores the original bytes.
It is not secure encryption.

```sh
cube binary xor --text 'Hello' --key-hex 6b6579
# 230015070a
```

`--key` supplies UTF-8 key bytes; `--key-hex` supplies arbitrary bytes. Choose exactly
one. Use raw output when chaining binary operations. The descriptor is `binary.xor`.

## Text operations

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
- Appended-data detection currently validates basic PNG framing/CRCs through IEND,
  then reports trailing offsets/counts. It does not validate compressed pixels;
  APNG and unknown critical chunks are unsupported. Work before IEND is capped
  at 64 MiB/10000 chunks. Trailing bytes do not establish concealed content.

## Image analysis and steganography

Use `image` operations to inspect PNG structure, compare exact samples or extract
selected channels/bits. The [CLI fixture](cli.md#create-small-image-and-audio-fixtures)
provides an original image with known low bits for these examples.

```sh
cube image histogram sample.png --channels red,green,blue --json
cube image bit-plane sample.png --channel red --bit 0 --stdout > plane.bin
```

Decoding supports 8-bit grayscale, grayscale+alpha, RGB, RGBA and indexed PNG.
Limits include 32 MiB encoded data, 16384 pixels per side, 4 Mi pixels and 16 MiB
of decoded samples. Other formats/depths fail explicitly; no gamma/profile
conversion, premultiplication or compositing occurs. PNG chunk inspection checks
framing/CRCs without decoding pixels; bad-CRC reporting and payload export are opt-in.

Coordinates use a top-left origin; regions are `x,y,width,height`, entirely inside
the image. Selected channel order is preserved. Native indexed data stays palette
indexes; request expansion explicitly for RGBA. Histograms count exact selected
color tuples in sorted order, including alpha only when selected. Exceeding a
color budget fails, rather than approximating. Coordinate listings cap at 10000
pixels. Comparison requires equal dimensions and compatible selected samples.

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

## Audio analysis

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
detector or playback feature. Video analysis is not implemented.

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
