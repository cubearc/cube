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

## Open Cube Play

`cube play [path]` delegates to the separately installed Cube Play host launcher.
With no path, it preserves the current directory. Directory, file and challenge
paths are passed as literal operating-system arguments; Cube does not evaluate
challenge contents or manage Docker.

```sh
cube play
cube play sample.mp4
cube play rabbit.cube
cube play --no-browser ./investigation
cube play --offline ./investigation
cube play --stop ./investigation
```

Install `cube-play` beside `cube` in a trusted installation directory, or set
`CUBE_PLAY_LAUNCHER` to its absolute executable path. Relative overrides are rejected;
Cube never searches the investigation directory or `PATH` for this executable.
Installation and runtime prerequisites belong to [Cube Play](https://github.com/cubearc/play).

Cube checks the bounded `cube-play-launcher 1` protocol before delegation. An
incompatible or missing launcher returns actionable guidance; existing Cube
commands continue to work without Play or Docker. `--json-errors` is forwarded
and bootstrap errors use Cube's existing JSON envelope. `--offline` requires an
installed runtime image. `--stop` preserves investigation data.

On Unix, successful delegation replaces the Cube process, preserving signals and
exit status without an extra parent. On Windows, both programs share the console;
Cube waits and propagates the launcher's conventional 0–255 exit codes. Play owns
interruption cleanup, startup/reuse, browser opening and shutdown.

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

## Transform by scalar position

```sh
cube text transform masked --text ABCDEFGHIJKLMNOPQRSTUVWXYZ \
  --selector fibonacci --exclude 13,21 --index-base 1 \
  --selected atbash --otherwise caesar --otherwise-shift 3
# ZYXGVIJSLMNOPQRSTUVWXYZABC
```

Use `--selector indices --indices 0,2,5` for an explicit position set, or
`--selector every_nth --nth 2` for every second scalar. The default base is 0;
`--index-base 1` numbers the first scalar 1. Every Unicode scalar counts,
including spaces, punctuation and separate combining marks. See the
[selection semantics and limits](operations.md#position-aware-transformations).

Both sides accept `atbash`, `caesar`, `rot13` or `identity` (default for
`--otherwise`). Caesar requires its corresponding `--selected-shift` or
`--otherwise-shift`, including negative shifts. Ordinary output is the final text.
Add `--json` for original input, configuration, selected positions, deterministic
text, overrides and final text. Explicit `--override '1=Z'` records one manual
replacement after transformation; repeat the option to apply edits in order.
The deterministic text remains in JSON, so caller guesses can be distinguished
from the reproducible cipher result. No replacements are inferred automatically.

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
Classical ciphers are for challenges and education, not modern confidentiality.

### Versioned Base58Check

Use an explicit decimal version byte (0..255); payloads are limited to 1024 bytes.
This public Bitcoin Core vector is independent of wallet/private-key policy:

```sh
printf '%s' a31c06bd463e3923bc1aadbde48b16976c080717 | cube decode hex --output raw | cube encode base58check --version-byte 0
cube decode base58check --text 1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo --json
```

The decoded JSON retains the version and exact payload hex. Text is strict,
including stdin: a trailing newline is rejected. Empty payloads are supported,
but empty encoded text is invalid. The encoder also composes safely with typed
bytes under its smaller input and quadratic-work budgets; the structured decoder
has no pipeline adapter. [Limits and semantics](operations.md#versioned-base58check).

## Interpret directional symbols

Interpret an exact 12-hour clock reading or signed integer millidegrees:

```sh
cube number clock --time 3:00 --decode --json
cube number clock --hour-millidegrees -45000 --minute-millidegrees 0 --json
cube number clock --time 12:45 --json
```

The last reading reports boundary ambiguity, without guessing. Hour motion is
continuous. `--zero-axis N|E|S|W` and `--counterclockwise` explicitly change the
convention. Optional decoding maps hour to sender LH and minute to RH; it fails
for ambiguous/unsupported pairs and does not make clock mapping an official code.

Letter-only flag Semaphore uses the sender's anatomical left/right arms, viewed
facing the sender; directions are `N,NE,E,SE,S,SW,W,NW` in that receiver-facing plane:

```sh
cube encode semaphore --text 'A B'
cube decode semaphore --text 'S,SW / S,W'
```

Tokens are `LH,RH`, separated by one ASCII space; `/` encodes each literal space.
ASCII letters canonicalize to uppercase; digits, other whitespace and control
signals are rejected. D uses `S,N` and also accepts alternate `N,S` on decode.

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

## Arrange text in scalar grids

Inspect scalar factor pairs or select exact geometry; grids use tab-separated,
JSON-quoted cells and LF rows so control characters remain unambiguous:

```sh
cube text grid --text ABCDEF --json
cube text grid --text ABCDEF --width 3
cube text grid --text 0101 --width 2 --colors '[["0","000000ff"],["1","ffffffff"]]' --output grid.png
```

`--drop-whitespace` is explicit; no normalization occurs. `--traversal` reuses image
coordinate order with a top-left start. PNG mappings are complete explicit RGBA.

## Generate string combinations

Generate bounded ordered string candidates as JSON Lines (one quoted string per
record, preserving embedded newlines and controls); `--plan` only reports bounds:

```sh
cube text combinations --text '[["a","b"],["1","2"]]' --separator - --plan
cube text combinations --text '[["a","b"],["1","2"]]' --separator -
```

The rightmost group varies fastest. Duplicate strings remain separate results.
Plans exceeding `--max-results` (default 1000, hard cap 10000) fail before output;
there is no silent prefix. CLI group JSON uses the existing 256 KiB/4096-node parser.

## Transform binary bytes

Byte transforms preserve binary output conventions:

```sh
cube binary and --key-hex ff0f --text AB
cube binary shift-left --bits 1 --unit-bytes 2 --text AB
cube binary swap-endian --unit-bytes 2 --text ABCD
```

Use `--output raw` for exact stdout bytes. AND/OR operands must have equal length;
shifts use independent unsigned big-endian units, with explicit zero-fill rules.

## Select mathematical constant digits

`max-digits` controls the complete certified fractional prefix: default 256,
explicitly adjustable to 1..4096. Positions never increase that budget implicitly.
Index 0 is the first digit after the decimal point; plain output is the selected
fractional digits plus one newline:

```sh
cube number digits pi --count 8
cube number digits pi --start 256 --count 16 --max-digits 512 --json
cube number digits sqrt2 --indices 2,0,2 --json
```

Choose a range (`--start` default 0, `--count` default 16) or `--indices`, never both.
JSON includes the separate integer part, selected digits, prefix budget/hash and
certification method. Empty selections are valid. The hard cap, work and integer
bounds remain enforced with an override; see [numeric limits](operations.md#numeric-conversions).

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

## Validate offline key material

These examples use public Bitcoin Core/synthetic test material. Cube reads exact
stdin and reports only safe validation facts:

```sh
printf '%s' L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd | cube blockchain wif --expected-network mainnet --json
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain scalar --format hex --json
```

For supplied files, redirect stdin (`cube blockchain wif --json < supplied.wif`).
Text is strict: no trailing newline, prefix or whitespace is normalized. `--text`
is available under existing conventions but exposes its value in process arguments.
The default human report and optional JSON contain no private bytes, WIF or
fingerprints. The `testnet` expectation means the shared testnet/signet/regtest
version family, not independent chain identification. [Contracts and limits](operations.md#wif-and-scalar-validation).

### Derive a public Bitcoin address

Use explicit network/SEC1 flags with supplied exact input. This synthetic public
scalar-1 vector returns `1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH`:

```sh
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain bitcoin --format hex --network mainnet --compressed --json
cube hash ripemd160 --text 'Hello world!'
```

Default derivation output is just the public address plus a newline. `--json`
adds public SEC1/HASH160 hex and parameter facts. `--format wif` reuses validation
and rejects conflicts with `--network`/encoding. The command exports no private
material, accesses no network and creates no file.

### Deliberately convert private formats

This public scalar-1 example permits exact WIF stdout explicitly:

```sh
printf '%s' 0000000000000000000000000000000000000000000000000000000000000001 | cube blockchain export --format hex --to wif --network mainnet --compressed --private-output --stdout
```

Use `--output new-key.wif` instead of `--stdout` for an exclusive new file. On Unix,
private files are created with mode 0600; other systems use platform ACLs.
Existing files are rejected and partial new files are removed on write failure.
No newline/metadata is added to private stdout or file content, so an explicitly
chosen consumer can read exact bytes. For WIF input, `--to raw`/`--to hex` return
only the scalar; re-exporting WIF requires matching explicit network/encoding.
Missing intent/destination/required WIF parameters fails without private output.
Validation commands remain safe reports. Caller-owned buffers may retain copies.

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

Combine compatible images using every physical sample, including alpha:

```sh
cube image mirror sample.png --axis horizontal --output mirrored.png
cube image bitwise sample.png mirrored.png --operation xor --output combined.png --json
```

`--operation` requires `xor`, `and` or `or`. Both images must have identical
dimensions, sample models, palettes and raw transparency metadata. Native indexed
samples combine as indices; expand each image explicitly with `image convert
--expand-palette` to combine RGBA instead. Invalid resulting indices fail.
The output is a new lossless PNG; existing destinations are rejected. JSON reports
the selected operation, output image facts and constructed-sample provenance.

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

### Exchange pixel grids through CSV

Exchange small exact sample grids through versioned CSV (gray, gray-alpha, RGB or
RGBA, 8-bit, at most 10000 pixels). File outputs are exclusive; stdout contains
only CSV bytes. Import accepts `--input`, `--text` or stdin and validates every
coordinate before writing a new PNG:

```sh
cube image csv export sample.png --output pixels.csv
cube image csv import --input pixels.csv --output restored.png --json
cube image csv export sample.png --stdout | cube image csv import --output copied.png
```

Palette/tRNS inputs are rejected. See [CSV v1](operations.md#pixel-csv-v1) for the
schema and limits; encoded bytes and ancillary metadata are outside this contract.

### Process explicit image collections

Plan exact pair counts before exporting a bounded collection:

```sh
cube image pairs first.png second.png third.png --operation xor --plan
mkdir pairs
cube image pairs first.png second.png third.png --operation xor --output-dir pairs --json
```

The allowlist is `xor`, `and`, `or`, `compare`; paths are explicit and duplicate
canonical paths fail. Pairs follow input indexes `(0,1),(0,2),(1,2)`. PNG names use
`pair-00-01.png`; the manifest includes per-pair errors. A compatibility failure
retains successful outputs and exits 1; output collisions/write failures roll back
this invocation's new files. Default output is a concise count summary.

### Export exact-color collections

Export one exact-color mask per distinct native tuple to an existing directory:

```sh
mkdir layers
cube image color-layers sample.png --output-dir layers
```

The new `manifest.json` and canonical `color-<hex>.png` files are reserved together.
Collisions fail without overwriting; failed writes remove only this invocation's
new files. Use `--representation layer` for RGBA or an explicit `--colors` subset.
`--max-colors` and `--max-outputs` default to 16 and may be lowered or raised to 64.

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

### Transform positions in a pipeline

```sh
cube pipeline run --text ABCDEFGHIJKLMNOPQRSTUVWXYZ --steps '[{"operation":"text.transform.masked","version":1,"parameters":{"selector":"fibonacci","exclude":{"unsigned_list":[13,21]},"index_base":1,"selected":"atbash","otherwise":"caesar","otherwise_shift":3}}]' --format json
```

The text output is `ZYXGVIJSLMNOPQRSTUVWXYZABC`; `masked_audits["0"]` preserves
the full report, including deterministic text and each explicit replacement.
Use `{"text_list":["1=雪","1=🧊"]}` for the `overrides` parameter to record edits
in order. Both pipeline and recipe runs require `--format json` for this adapter;
ordinary/raw output fails instead of discarding audits. `recipe check` includes
these audits with its exact-output check. The existing recipe format/engine API 1
can use this version-1 adapter. [Contracts and bounds](operations.md#compose-operations)
explain tagged empty lists and shared retention/work budgets.

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

## Inspect video containers

```sh
cube video inspect sample.mp4 --json
cube video inspect sample.webm --json
```

These commands read declared container/track metadata without decoding media.
Unsupported encryption, fragmentation and profiles fail explicitly. See
[container scope and limits](operations.md#video-container-metadata). Output never
claims that media payloads are valid or playable; the source stays unchanged.

For the supported self-contained MOV/JPEG subset, choose explicit frame indexes
and an existing output directory. The command writes new PNG files and emits a
JSON manifest with source ranges, encoded hashes and exact decode times:

```sh
mkdir frames
cube video frames clip.mov --track-id 1 --indexes 0,2 --output-dir frames
```

Names are `track-1-frame-0.png` and `track-1-frame-2.png`. Existing destinations
cause an error; failed batches clean up only newly created files. Read the
[JPEG extraction scope and budgets](operations.md#video-container-metadata).

Compare explicit pairs directly from the supported MOV, preserving source
provenance in JSON, or reuse the existing command for previously exported PNGs:

```sh
cube video compare clip.mov --track-id 1 --pairs 0:1,0:2 --max-results 100
cube image compare frames/track-1-frame-0.png frames/track-1-frame-2.png --json
```

The difference-list budget applies to the whole video batch; exact differing-pixel
counts and per-pair completeness remain available. Optional `--channels` selects
existing sample channels; mismatched dimensions fail without resizing.

For a supported raw PCM8 MOV track, export a new WAV and reuse the audio commands:

```sh
cube video audio sound.mov --track-id 1 --output track.wav
cube audio inspect track.wav --json
```

File output emits JSON provenance and never overwrites. `--stdout` explicitly
streams binary WAV bytes without appended text. This subset preserves unsigned
PCM8 mono/stereo samples and rate; compressed codecs and other layouts are
unsupported. See the [audio extraction limits](operations.md#video-container-metadata).

## Inspect saved HTML and HAR

```sh
python - <<'PYTHON'
import json
from pathlib import Path
Path("saved.html").write_text("<a href='../not-opened'>Link</a><script>fetch('https://invalid.example')</script>")
artifact = {"log": {"version": "1.2", "entries": [{"request": {"method": "GET", "url": "https://invalid.example/asset"}, "response": {"status": 200, "content": {"mimeType": "application/octet-stream", "text": "AP8=", "encoding": "base64", "size": 2}}}]}}
Path("saved.har").write_text(json.dumps(artifact))
PYTHON
cube web html saved.html --json
cube web har saved.har --json
```

These commands inspect only the explicitly selected local artifact. References,
scripts and handlers remain inert; no URL/path is resolved and no browser data is
accessed. See the [supported subset and limits](operations.md#offline-web-artifacts).

Extract a selected embedded HAR body to a new file, inspect its provenance, or
send its exact bytes to the existing pipeline:

```sh
cube web asset har saved.har --index 0 --json
cube web asset har saved.har --index 0 --output selected.bin
cube web asset har saved.har --index 0 --stdout | cube pipeline run --steps '[{"operation":"encoding.hex.encode","version":1}]' --format raw
# 00ff (without an added newline)
```

The output file must not exist. HTML uses `cube web asset html saved.html --index 0
--selector inline --stdout` for an explicitly closed script/style body, or
`--selector data_url` for a reported reference containing strict base64. No path
or remote reference is followed. Both asset operations also compose directly as
version-1 text-to-bytes steps; select an index and use explicit UTF-8 conversion
only if a later text operation needs it.

### Ethereum public derivation

Use exact raw32 (default) or hex64 scalar input. This public scalar-1 example
returns the default EIP-55 text; `--checksum lowercase` selects lowercase output:

```sh
printf '%064x' 1 | cube blockchain ethereum --format hex
cube hash keccak256 --text abc
```

`--json` includes only public coordinates/address and checksum choice. Keccak-256
is not SHA3-256; derivation never queries a chain. WIF is not an Ethereum scalar
transport. Prefer exact stdin over private material in command-line arguments.

### Validate supplied key candidates

Provide an explicit JSON array of WIF/hex64 entries. This synthetic public test
input demonstrates one valid scalar and one isolated invalid candidate:

```sh
printf '[{"format":"hex","value":"%064x"},{"format":"wif","value":"invalid"}]' 1 |
  cube blockchain candidates --json
```

The report has stable zero-based indexes, safe structural facts/errors and complete
counts. Default output omits all input and derived addresses. Add
`--derive bitcoin --network mainnet --compressed` (or `--uncompressed`) for public
Bitcoin output; add `--derive ethereum [--checksum eip55|lowercase]` for Ethereum
from hex scalars. Conflicting WIF intent produces a separate address error while
preserving its valid classification. Unused context flags are rejected.

The input must fit the [candidate limits](operations.md#bounded-key-candidates).
No candidate generation, file writes or network access occurs. Prefer stdin;
command-line text may remain in shell history/process listings.

## Puzzle mappings

Choose a Bacon alphabet explicitly, or use the shared coordinate square with an
explicit Tap preset:

```sh
cube cipher bacon --variant extended26 --text AB
# AAAAA AAAAB
cube cipher bacon --variant classical24 --decode --text 'ABAAA BAABB'
# IU
cube cipher polybius --preset tap --text 'CK J'
# 1,3 1,3 / 2,5
cube cipher polybius --alphabet 'αβγδ' --side 2 --index-base 0 --text 'δ α' --json
# [[1,1],null,[0,0]]
```

`--alias 'a=α'` adds a custom scalar alias. `--decode` accepts the exact
`ROW,COLUMN / ROW,COLUMN` grammar, retaining each space as a slash token.
`--notation binary` and `binary_inverted` choose explicit Bacon bit orientation.

```sh
cube text keyboard --source qwerty --target dvorak --text qwerty
# ',.pyf
cube encode keypad --mode telephone --text HELLO
# 43556
cube encode keypad --mode multi_tap --text HELLO
# 44|33|555|555|666
cube decode keypad --mode telephone --text 4
# {"text":null,"alternatives":["GHI"],"reversible":false}
cube text class --unmapped filter --text 'éa!B'
```

Keypad decode and character classes always emit JSON to retain ambiguity and
position evidence. `text class --y vowel|consonant|unmapped` resolves Y explicitly.
Keyboard `--unmapped error` reports unknown scalars; its default preserves them.

Custom `--source`/`--target` layouts, keypad `--mapping` and class `--definition`
accept inline strict JSON, with the schemas and bounds in
[Puzzle mappings](operations.md#puzzle-ciphers-and-symbolic-mappings). For example:

```sh
cube encode keypad --mode multi_tap --text '0 ' \
  --mapping '{"ascii_fold":false,"keys":[{"digit":"0","symbols":" 0"}]}'
# 00|0
cube text class --text 'αβ' \
  --definition '{"ascii_fold":false,"classes":[{"name":"greek","characters":"αβ","symbol":"!"}]}'
```

## Custom positional digits

```sh
cube number alphabet --source-alphabet 01 --target-alphabet abcdefghij \
  --from-base 2 --to-base 10 --text 1010
# ba
```

Alphabets and active bases are always explicit; custom digits are exact Unicode
scalars. See [numeric alphabet limits](operations.md#custom-numeric-alphabets).

## Explicit text traversal

```sh
cube text keyed-skip --steps 2 --count 5 --text ABCDE
cube text josephus --steps 2 --text ABCDE
```

JSON reports contain extracted BDACE/BDAEC respectively, original positions,
remaining source size and charged work/scratch estimates. `--steps 2,3` cycles
explicitly. Options include `--start`, `--index-base 0|1`, `--direction reverse`,
keyed-only `--no-wrap` and lower `--max-work`, `--max-memory`, `--max-output`.
See [traversal semantics and bounds](operations.md#keyed-skip-and-josephus-traversal).

## Positions inside every word

```sh
cube text word-extract --positions '[1,3]' --text 'Cube Uses Binary Evidence'
cube text word-extract --per-word '[[2,1,2],[-1]]' --text 'ab cd'
cube text word-extract --positions '[2]' --short-word skip --text 'A bé'
```

These emit JSON with extracted CbUeBnEi, babd and é respectively. The last report
retains the skipped request for A. Positive origin is 1 by default; `--index-base
0` permits zero. Negative positions count from each word's end. See
[word selection contracts](operations.md#signed-positions-inside-words).

## Numeric text evidence

```sh
cube text word-lengths --text 'A é'
cube text codepoints --format unicode --text 'A é'
cube text word-values --sums --text 'A é'
cube text word-values --sums --table '[["A",1],["B",2]]' --text AB
```

All output is JSON, preserving numeric fields and original boundaries. Formats
for codepoints are decimal, hex and unicode (U+XXXX). Custom tables match exact
scalars; `--unmapped skip` retains null omissions. `--max-sum` bounds checked sums.
See [numeric evidence limits](operations.md#numeric-text-evidence).

## Inspect Unicode and compare scalar views

```sh
cube text unicode-inspect --text 'Aaé' --max-results 1000
cube text unicode-compare --text 'é' --other 'é' --normalization NFC
cube text unicode-compare --text A --other-file saved-text.txt
```

The second report is exact_equal=false, normalized_equal=true and retains the
original differences separately. `--other` and bounded UTF-8 `--other-file` are
mutually exclusive. Listing options are `--max-results` (zero counts-only), lower
`--max-work` and `--max-output`. Inspect declared Unicode versions and complete
flags; see [Unicode report semantics](operations.md#unicode-observations-and-scalar-differences).

## Exact modular arithmetic

Unsigned operands use 1..20 ASCII decimal digits, no sign/prefix/whitespace.
JSON uses decimal strings to retain full u64 and signed coefficient precision.

```sh
cube number gcd --a 48 --b 18
cube number extended-gcd --a 240 --b 46 --json
cube number mod-inverse --a 3 --modulus 11
cube number mod-pow --base 4 --exponent 13 --modulus 497 --json
```

These return 6, gcd=2/x=-9/y=47, 4, and `{"value":"445"}` respectively.
A nonexistent inverse errors; power accepts exponent zero and modulus 1.

## Decimal digital root

Use strict decimal text with <=4,096 digits and optional +/-; file/stdin input
must contain exactly that numeral, including no final newline. Only radix 10 is
supported. JSON contains a single small numeric field.

```sh
cube number digital-root --text=-38
cube number digital-root --text +000999 --radix 10 --json
```

These return 2 and `{"value":9}`. Floats, prefixes and over-128-bit magnitudes
error instead of being interpreted or rounded.
