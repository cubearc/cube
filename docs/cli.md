# CLI Guide

For installation and a first operation, follow [Getting Started](getting-started.md).
Examples below use a POSIX shell. Run file-producing examples in a scratch
directory; Cube's destination options refuse existing files.

## Find the right command

Commands generally follow `cube <group> <operation> [arguments]`.
Use the groups for the task at hand:

- `encode`, `decode`, `cipher`, `hash`, `binary`, `text`, `number`: transform data.
- `inspect`, `file`, `image`, `audio`: inspect files or export selected data.
- `pipeline`, `recipe`: compose operations into repeatable linear workflows.
- `investigate`: suggest inspections, record evidence and replay local analysis.
- `ops`: discover operations and their descriptions.

`cube inspect path` is a top-level shortcut for basic file information.
Use `cube --help`, then progressively narrower help such as
`cube image --help` and `cube image lsb --help`.

## Discover operations

```sh
cube ops list
cube ops list --category file
cube ops list --json
cube ops describe file.signature.detect --json
```

Descriptors provide stable operation identifiers, categories, semantic input/output
types, parameters, constraints, defaults, determinism, reversibility and examples.
The JSON listing is sorted by identifier and uses `schema_version: 1`. Consumers
should tolerate added fields and operation identifiers. Each JSON descriptor adds
`pipeline: {"version": 1}` when a bounded adapter exists, or `pipeline: null` when
it does not. Human `ops describe` also reports this availability. The version is
the adapter contract required by a step, independent of the package version.
Unknown categories or identifiers are errors. `pipeline adapters` lists the same
executable subset; parameter, type and resource validation still applies.

## Supply text or bytes

For transformation commands, use `--text` for inline UTF-8 text or omit it to
read stdin until EOF. Quote text containing spaces or shell metacharacters.
Use `printf` when you do not want an extra newline:

```sh
cube encode base64 --text 'Hello'
printf 'Hello' | cube hash sha256
printf 'Cube' > sample.bin
cube encode hex < sample.bin
```

Encoding, hashing and XOR accept arbitrary bytes. Text operations and classical
ciphers require valid UTF-8. A filename passed to `--text` is treated as literal
text; it does not open that file. File/image/audio commands take explicit paths.
Paths must identify regular files; symbolic links, directories and devices fail.

Pipelines also accept `--input path`, `--hex value`, or `--value` tagged JSON.
Choose one input selector. Their default input is raw stdin.
Recipes instead require explicit named bindings; see [recipe files](operations.md#recipe-files).

## Choose an output format

- Ordinary text results have a final newline.
- Decoding, XOR and `text to-bytes` default to hexadecimal plus a newline;
  `--output raw` writes exact bytes without a newline.
- Inspection commands offer human-readable output and, where listed in help,
  `--json`. Some export commands require a destination choice.
- Pipeline/recipe runs use `--format raw|hex|json`. The default displays bytes as
  hex and text as text; JSON preserves type tags and captured intermediate results.

```sh
cube decode base64 --text 'SGVsbG8='
# 48656c6c6f
cube decode base64 --text 'AP8=' --output raw > decoded.bin
cube inspect decoded.bin --json
```

Do not convert arbitrary raw output through a text filter or text-mode file API.
If your shell's binary redirection behavior is uncertain, use the Python SDK's
`bytes` results and `Path.write_bytes` instead.

## Write results to files

`--output` has command-specific meanings: for a decoder it selects representation;
for extraction/transform commands it names a destination. Help is authoritative.

```sh
cube file extract sample.bin --offset 0 --length 4 --output selected.bin
cube file extract sample.bin --offset 0 --length 4 --stdout > selected-stdout.bin
```

File, image and audio exports use `--output path` or explicit raw `--stdout` where
supported. A file destination usually leaves metadata/status on stdout; `--stdout`
emits bytes only. Image transforms require a new PNG destination. Investigation
records use `--save path`. These destination options never overwrite existing
files, including aliases of the source.

Shell `>` redirection is controlled by your shell and can overwrite an existing
file before Cube runs. Use a new path or Cube's exclusive destination option.

## Transform encoded data and ciphers

```sh
cube text select --selector word_first --text 'Cube Uses Binary Evidence'
cube text extract --start 1 --stride 2 --text abcdef
cube text statistics --text ABAB --ngram-size 2 --json
cube text statistics --text 'B!Aa?雪' --case ascii_lower --filter ascii_letters
```

The first two commands produce `CUBE` and `bdf`; the statistics commands report
counts and numeric observations. Use `line_first` for an untrimmed acrostic;
`word_last` and `line_last` select final scalars, including punctuation or spaces.

```sh
cube encode a1z26 --text 'Hi Cube'
cube decode a1z26 --text '8-9 3-21-2-5'
```

A1Z26 preserves ASCII whitespace and decodes to uppercase. Text commands append
their usual output newline; punctuation and malformed numeric tokens fail.

```sh
cube encode base32 --text 'foo'
# MZXW6===
cube decode base58 --text 'JxF12TrwUP45BMd' --output raw
# Hello World (no added newline)
cube cipher caesar --shift 3 --text 'Attack at dawn!'
# Dwwdfn dw gdzq!
cube cipher vigenere --decode --key LEMON --text LXFOPVEFRNHR
# ATTACKATDAWN
cube cipher affine --a 5 --b 8 --text AFFINECIPHER
# IHHWVCSWFRCP
```

Use `--decode` with the original key/parameters to reverse supported ciphers.
Classical ciphers are for puzzles and education, not modern confidentiality.

## Extract from a caller-supplied book

```sh
printf 'Cube Uses Binary Evidence' > /tmp/cube-book-source.txt
cube cipher book --source /tmp/cube-book-source.txt --text '1:1,2:1,3:1,4:1'
# CUBE
```

Coordinates may also come from stdin. `--scheme line_character` selects
line:scalar pairs; `--scheme line_word_character` selects line:word:scalar triples.
Use `--index-base 0` for zero-based coordinates. The local source is read with a
1 MiB limit and remains unchanged; errors never substitute guessed characters.
See [coordinate semantics and limits](operations.md#classical-ciphers).

## Transform binary bytes

Byte transforms preserve binary output conventions:

```sh
cube binary and --key-hex ff0f --text AB
cube binary shift-left --bits 1 --unit-bytes 2 --text AB
cube binary swap-endian --unit-bytes 2 --text ABCD
```

Use `--output raw` for exact stdout bytes. AND/OR operands must have equal length;
shifts use independent unsigned big-endian units, with explicit zero-fill rules.

## Compute a hash or checksum

SHA-512 is available alongside SHA-256. MD5 and SHA-1 are legacy and unsuitable
for collision-resistant security. CRC32C and CRC-16/IBM-3740 are checksums, with
parameters listed in the [operations guide](operations.md#hashing-and-checksums).
New hash commands accept at most 16 MiB of exact input bytes.

```sh
cube hash sha512 --text abc
cube hash crc32c --text '123456789'
# e3069283
cube hash crc16-ibm3740 --text '123456789'
# 29b1
```

```sh
cube hash sha256 --text 'Hello'
# 185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969
cube hash crc32 --text '123456789'
# cbf43926
```

A piped newline changes the input and therefore the result. SHA-256 is one-way;
CRC32 detects accidental corruption and is not a cryptographic hash.

## Inspect binary files

```sh
cube file signature sample.bin --json
cube file hex sample.bin --offset 0 --length 4
cube file strings sample.bin --min-length 4 --json
cube file compare sample.bin selected.bin --json
```

Offsets count bytes from zero. `offset` plus `length` is an exclusive end; ranges
beyond EOF fail rather than clip. Hex defaults to at most 256 bytes. Differences
are successful comparison results, not exit-code failures. Check `complete` on
bounded scans/listings. See [file forensics](operations.md#file-forensics) for
signatures, EXIF, embedded candidates and appended-data limits.

### Inspect a structural boundary

```sh
cube file appended sample.jpg --json
cube file appended sample.gif --json
```

`logical_end`, `trailing_offset` and `trailing_bytes` are exact byte positions/counts.
Read `evidence` for the validation scope. Supported JPEG/GIF framing is checked
without decoding compressed pixels; an unsupported feature fails explicitly.
The same command preserves existing strict PNG framing/CRC behavior.

### Validate an embedded ZIP

First locate candidate offsets, then select one explicitly:

```sh
cube file embedded sample.bin --format zip --json
cube file zip sample.bin --offset 128 --json
```

A PK signature alone is a candidate. Validation reports exact archive bounds,
entry offsets and verification limits; it does not decompress files or verify
payload CRCs. Unsupported ZIP features fail explicitly. Use the reported range
with an explicit extraction command when recovery is intended.

### Recover a selected candidate

```sh
cube file carve sample.bin --offset 128 --output recovered.zip --json
cube file carve sample.bin --offset 128 --length 256 --max-bytes 256 --output selected.bin --json
```

The first command revalidates framing at the selected origin. The second selects
an exact raw range. Both require a new destination and report exact range
provenance and the recovered bytes' SHA-256; neither decompresses archive entries.
`file extract` remains available for existing explicit range/stdout workflows.

## Create small image and audio fixtures

These original fixtures need only Python's standard library. The PNG has eight
RGB pixels whose red-channel low bits spell the byte `A`. The WAVE file contains
four mono 16-bit PCM samples at 8000 frames/second.

```sh
python3 - <<'PY'
import base64
import wave
from pathlib import Path

png = (
    "iVBORw0KGgoAAAANSUhEUgAAAAQAAAACCAIAAADwyuo0AAAAGElEQVR4nGNh"
    "YmBgZGD4zwACLAwwFlAIABzlAguTT+b/AAAAAElFTkSuQmCC"
)
Path("sample.png").write_bytes(base64.b64decode(png))
with wave.open("sample.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(b"\x00\x00\x00\x40\x00\x00\x00\xc0")
PY
```

## Analyze an image

```sh
cube image inspect sample.png --json
cube image histogram sample.png --channels red,green,blue --json
cube image colors sample.png --colors 020000 --output exact-mask.png
cube image lsb sample.png --channels red --traversal zigzag --start-corner top_right --stdout > zigzag.bin
cube image pixels sample.png --channels red --traversal spiral_cw --hex
cube image pixels sample.png --channels red --stdout | cube pipeline run --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube image channel sample.png --channel red --output red.bin
cube image rotate sample.png --degrees 90 --output rotated.png
cube image lsb sample.png --channels red --bits 0 --stdout > extracted.bin
```

The histogram reports six `[2,0,0]` pixels and two `[3,0,0]` pixels; the LSB result
is one byte, `41` in hex (`A`). These known bits were deliberately placed in a
synthetic image; an extraction from another image does not prove hidden content.
Images accept PNG8, baseline JPEG, single-frame GIF, BMP and static WebP.
Use `cube image inspect input.jpg --json` to inspect decoded-pixel provenance.
Pixel coordinates start at `(0,0)`
at the top left; palette expansion and lossy transforms are explicit.
See [image analysis](operations.md#image-analysis-and-steganography).

Reconstruct explicitly selected raw bytes, without rewriting their source:

```sh
python -c 'from pathlib import Path; Path("pixels.bin").write_bytes(bytes([3,2,1,0,6,5,4,0]))'
cube image raw pixels.bin --width 1 --height 2 --layout bgr --row-stride 4 --output reconstructed.png
```

Omit `--output` to validate and inspect layout facts without writing a file.
Use `--compare-header` to report declared header dimensions separately from the
candidate layout, including header parsing errors. For another explicit hypothesis:

```sh
cube image raw pixels.bin --width 2 --height 1 --layout rgba --compare-header
```

For an already obtained QR image, use `cube image qr code.png --json`.
`--stdout` emits one decoded payload as exact bytes; `--output payload.bin`
creates a new destination. No detection is a normal JSON result, while ambiguous
or undecoded raw output fails explicitly.

## Inspect audio samples

```sh
cube audio inspect sample.wav --json
cube audio samples sample.wav --channels 0 --frames 4 --json
cube audio samples sample.wav --channels 0 --frames 4 --output channel.pcm
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```

Sample values are `[0,16384,0,-16384]`. A frame contains one sample per channel;
selections count frames, not bytes. Raw PCM exports need their sample rate, sample
width and channel mapping to be interpreted. Waveform output is min/max data,
not a plot or FFT. Only integer PCM RIFF/WAVE is supported; see
[audio analysis](operations.md#audio-analysis).

For an explicit frequency window, select a channel and N frames:

```sh
python - <<'PYTHON'
import wave
with wave.open("spectrum.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(b"\x00\x00\x00\x40\x00\x00\x00\xc0" * 2)
PYTHON
cube audio spectrum spectrum.wav --channels 0 --fft-size 8 --json
cube audio spectrogram spectrum.wav --channels 0 --fft-size 8 --hop-size 4 --final-window pad --json
cube audio bits spectrum.wav --channels 0 --bits 15 --frames 8 --json
cube audio reverse spectrum.wav --output reversed-spectrum.wav
cube audio dtmf spectrum.wav --channels 0 --json
```

The command defaults to 1,024 frames at `--start-frame` and the periodic Hann
window. Set `--window rectangular` or `hamming` explicitly when needed. If
`--frames` is provided it must equal `--fft-size`; short inputs fail. See
[spectrum units and bounds](operations.md#audio-analysis).

Tone intervals can be inspected before explicitly requesting Morse transcription:

```sh
python - <<'PYTHON'
import math
import struct
import wave
values = [round(12000 * math.sin(math.tau * 700 * i / 8000)) for i in range(800)]
with wave.open("morse-dot.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(struct.pack("<800h", *values))
PYTHON
cube audio morse detect morse-dot.wav --frequency-hz 700 --json > intervals.json
cube audio morse transcribe --unit-ms 100 --json < intervals.json
# tokens: ".", text: "E"
```

The frequency and dot duration are caller assumptions. Inspect uncertain segments
and timing resolution before treating a candidate transcript as meaningful.

Inspect a previously obtained native FLAC file without decoding audio:

```sh
cube audio metadata sample.flac --json
```

This supports STREAMINFO and bounded Vorbis comments. MP3/Ogg and ID3-prefixed
FLAC are explicitly unsupported. Duration is declared metadata, and the audio
payload/MD5 are unverified. See [metadata bounds](operations.md#audio-analysis).

## Run a pipeline or recipe

```sh
cube pipeline adapters
cube pipeline validate --input-type bytes --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube pipeline run --hex 00ff --steps '[{"operation":"encoding.hex.encode","version":1}]'
# 00ff
```

`--capture 0 --format json` retains the first step output; indexes start at zero.
`--limits` lowers pipeline budgets. Use `--format raw --output new-file` for exact
saved output. Types must match between steps; convert UTF-8 explicitly when needed.
[Composition contracts](operations.md#compose-operations) explain versions and limits.

After saving the [recipe example](operations.md#recipe-files) as `roundtrip.json`:

```sh
cube recipe validate roundtrip.json
cube recipe run roundtrip.json --hex payload=00ff --format raw --output result.bin
cube recipe check roundtrip.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
```

Recipe bindings are explicit `name=value` arguments: `--text`, `--hex`, `--input`
(raw file) or `--value` (tagged JSON). Validation does not execute steps. `check`
compares exact typed output and returns a nonzero status on a mismatch.

### Compose a forensic inspection

Decode exact bytes, then inspect their rows while retaining the decoded intermediate:

```sh
cube pipeline run --text '00ff' --steps '[{"operation":"encoding.hex.decode","version":1},{"operation":"file.hex.inspect","version":1,"parameters":{"width":1}}]' --capture 0 --format json
```

The final value is a typed forensic report; row offsets stay numeric and row bytes
use base64 in JSON. `--format raw` rejects structured reports. The file is supplied
by the caller or CLI input selector; steps never contain paths.

## Record an investigation

```sh
cube investigate triage sample.bin --json
cube investigate plan sample.bin --depth preview --json
cube investigate run sample.bin --depth preview --identify --save session.json --json
cube investigate report session.json
cube investigate replay session.json sample.bin --json
```

Triage suggests operations from evidence; it does not execute suggestions.
`plan` exposes ranges/budgets without reading source content. Preview adds bounded
hex/string analysis; image/EXIF/audio decoding is not automatic.
`--identify` records the complete input's SHA-256 identity for replay.

Default saved records omit raw inputs, paths, result contents, nonempty pipeline
parameters and user hypotheses. Per-run inclusion is explicit with `--include-result`
and `--include-parameters`; notes use `--hypothesis` and `--include-hypotheses`.
Use `--profile path` for a reusable profile or `--budgets '{"max_operations":1}'`
to override selected budgets. Read `status` and `replay_gaps`: partial analysis
can be a successful CLI result, and redacted records may be impossible to replay.
See [profiles and records](operations.md#investigation-profiles-and-records).

## Handle errors and exit codes

Results go to stdout; failures go to stderr. Exit codes are:

- `0`: successful operation, help/version, a comparison finding differences,
  or a valid investigation record (which may say `partial`).
- `1`: invalid operation input, resource/format failure, recipe mismatch or I/O failure.
- `2`: invalid command usage, such as missing arguments or an unknown flag.

A downstream closed pipe exits successfully. To request machine-readable errors:

```sh
cube --json-errors decode hex --text zz
```

This intentionally exits `1` with a `schema_version: 1` JSON error on stderr.
Fields include `code`, `message`, `operation`, `parameter` and byte `offset`;
unavailable context is `null`. Pipeline errors also identify the zero-based
`step`. Codes include `invalid_input`, `invalid_parameter`, `unsupported_format`,
`resource_limit`, `cancelled`, `io`, and CLI-only `invalid_usage`.
