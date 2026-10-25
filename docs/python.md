# Python Guide

Use `cube` as a library for text, bytes and structured analysis results. The SDK
shares Rust implementations with the CLI; you do not need to run CLI subprocesses.
Run file-writing examples in a scratch directory with new destination names.

## Installation

Install Python 3.10+, Git, stable Rust and a C linker; see
[prerequisites](getting-started.md#prepare-your-tools). Clone the repository if
needed, then install from its root in a virtual environment:

```sh
git clone https://github.com/cubearc/cube.git
cd cube
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install .
```

On Windows, replace the environment-creation/activation lines with PowerShell:

```powershell
py -m venv .venv
.\.venv\Scripts\Activate.ps1
```

Then run the same `python -m pip` commands. The local distribution is
`cubearc-cube`, but the import name is `cube`. These instructions build source;
they do not assume a published package. For editable development and rebuilding
after Rust changes, follow [Contributing](../CONTRIBUTING.md#setup).

## Start with bytes and text

```python
import cube

assert cube.a1z26_encode("Hi Cube") == "8-9 3-21-2-5"
assert cube.a1z26_decode("8-9 3-21-2-5") == "HI CUBE"
assert cube.select_text("Cube Uses Binary Evidence", "word_first") == "CUBE"
assert cube.extract_characters("abcdef", start=1, stride=2) == "bdf"
assert cube.book_decode("Cube Uses Binary Evidence", "1:1,2:1,3:1,4:1") == "CUBE"
stats = cube.text_statistics("ABAB", ngram_size=2)
assert stats["frequencies"] == [{"ngram": "AB", "count": 2}, {"ngram": "BA", "count": 1}]
assert stats["shannon_entropy_bits"] == 1.0
assert cube.and_bytes(b"\x00\xff", b"\xff\x0f") == b"\x00\x0f"
assert cube.shift_left(b"\x01\xff", 1, unit_bytes=2) == b"\x03\xfe"
assert cube.swap_endian(b"ABCD", unit_bytes=2) == b"BADC"
```

```python
import cube

encoded = cube.base64_encode(b"Hello")
assert encoded == "SGVsbG8="
assert cube.base64_decode(encoded) == b"Hello"
assert cube.hex_decode("00FF") == b"\x00\xff"
assert cube.vigenere_decode("LXFOPVEFRNHR", "LEMON") == "ATTACKATDAWN"
assert cube.affine_encode("AFFINECIPHER", 5, 8) == "IHHWVCSWFRCP"
```

Use `str` for encoded representations, classical ciphers and Unicode text.
Use `bytes` for binary inputs and decoded results. No operation silently interprets
arbitrary bytes as text. Convert deliberately when the encoding is known:

```python
assert cube.utf8_encode("Olá") == "Olá".encode("utf-8")
assert cube.utf8_decode(b"Hello") == "Hello"
assert cube.crc32(b"123456789") == "cbf43926"
assert cube.crc32c(b"123456789") == "e3069283"
assert cube.crc16_ibm3740(b"123456789") == "29b1"
assert len(cube.sha512(b"abc")) == 128
assert cube.sha256(b"Hello") == (
    "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969"
)
assert cube.xor(cube.xor(b"\x00\xff", b"key"), b"key") == b"\x00\xff"
```

`md5` and `sha1` support legacy comparisons and are unsuitable for
collision-resistant security. All hash functions consume exact caller-owned
bytes; CRC variants are checksums. See [variant semantics](operations.md#hashing-and-checksums).

Binary results have no added newline or hexadecimal display layer. To save them:

```python
from pathlib import Path

Path("decoded.bin").write_bytes(cube.base64_decode("AP8="))
assert Path("decoded.bin").read_bytes() == b"\x00\xff"
```

`Path.write_bytes` is a Python filesystem action and overwrites an existing file.
Use a new destination. Cube's own extraction/image save methods create files
exclusively and reject existing destinations.

## Inspect a local file

Create a small original fixture, then reuse its results without parsing terminal text:

```python
from pathlib import Path
import cube

source = Path("sample.bin")
source.write_bytes(b"Cube\x00\xff")
info = cube.inspect_file(source)
assert info["size"] == 6
rows = cube.file_hex(source, offset=4, length=2)
assert rows[0]["bytes"] == b"\x00\xff"
assert rows[0]["hex"] == "00ff"
assert cube.extract_bytes(source.read_bytes(), 4, 2) == b"\x00\xff"
strings = cube.file_strings(source, min_length=4)
assert strings["strings"][0]["text"] == "Cube"
assert strings["complete"] is True
```

Paths may be `str` or `pathlib.Path`; regular files are required, and symbolic links
are rejected. Inspection returns dictionaries/lists with explicit offsets and
completeness. Unavailable timestamps are `None`; EXIF is a separate explicit call.
For larger copying tasks, use `extract_file_range` or `copy_file_range` with a
binary writer instead of reading the entire file into Python memory.

A signature returned by `file_signatures(data)` or `scan_file(path)` is a candidate,
not proof that the format is valid. Read the [file limitations](operations.md#file-forensics).

### Validate and recover an embedded ZIP

Use the original `mystery.bin` from the
[recovery tutorial](getting-started.md#recover-an-embedded-file):

```python
from pathlib import Path
import cube

source = Path("mystery.bin")
candidates = cube.scan_file(source, format="zip")
assert any(item["offset"] == 8 for item in candidates["candidates"])
archive = cube.validate_zip_file(source, offset=8)
assert archive["length"] == 120
assert archive["payload_verified"] is False
recovered = Path("python-recovered.zip")
report = cube.carve_file(source, recovered, 8, max_bytes=archive["length"])
assert recovered.read_bytes() == source.read_bytes()[8:128]
assert report["sha256"] == cube.sha256(recovered.read_bytes())
```

The core revalidates the candidate before copying. Supply `length` only when you
intend an exact raw range without a format-validation claim. Existing outputs
raise `OSError`; invalid candidates/ranges/budgets raise `ValueError` with
`operation == "file.carve"`. Reports contain source size, range and recovered
content identity without automatically saving source paths in sessions.
`validate_zip(data, offset=8)` supports caller-owned buffers with the same report.
Read `scanned_bytes`, `candidate_scan_complete` and `evidence`; payloads are not
expanded or verified, and entry names stay inert exact hex data.

`appended_data(path)` also reports supported PNG/JPEG/GIF/ZIP structural
boundaries. JPEG/GIF compressed pixels are not decoded. These direct file APIs
perform explicit caller-selected local I/O, outside recipe steps.

## Analyze an image

This example constructs an original two-pixel image without an external asset:

```python
import cube

image = cube.Image.from_samples(2, 1, "rgb", b"\xff\x00\x00\x00\x00\xff")
assert image.pixel(0, 0) == [255, 0, 0]
assert image.channel("red")["data"] == b"\xff\x00"
histogram = image.histogram(channels=["red", "green", "blue"])
assert histogram["pixels"] == 2
assert len(histogram["colors"]) == 2
rotated = image.rotate(90)
assert (rotated.width, rotated.height) == (1, 2)
assert (image.width, image.height) == (2, 1)
image.save_png("two-pixels.png")
assert cube.Image.open("two-pixels.png").samples == image.samples
```

Images are immutable; transforms return a new `Image`. `Image.open(path)` loads a
supported raster, and `Image.decode(data)` accepts its encoded bytes. PNG8 `samples`
retain exact native values and palette indexes until explicit expansion. JPEG,
single-frame GIF, BMP and static WebP expose decoded 8-bit samples; inspect
`image.provenance()` to distinguish codec output from native PNG8 samples.
Use `pixel`, `channel`, `histogram`, `bit_plane` or `lsb` to inspect selected data.
See [image semantics and limits](operations.md#image-analysis-and-steganography).

Raw layouts are explicit and preserve source buffers:

```python
raw = cube.Image.from_raw(bytes([3, 2, 1, 0, 6, 5, 4, 0]), 1, 2, "bgr", row_stride=4)
assert raw.pixel(0, 1) == [4, 5, 6]
mask = raw.exact_colors([[4, 5, 6]])
assert mask["selected_pixels"] == 1
assert mask["image"].samples == b"\x00\xff"
stream = raw.lsb(bits=list(range(7, -1, -1)), traversal="spiral_ccw")
assert stream["data"] == raw.samples
ordered = raw.pixels(channels=["blue", "red"], traversal="spiral_cw")
assert ordered["data"] == bytes([3, 1, 6, 4])
encoded = raw.encode_png()
geometry = cube.Image.inspect_layout(encoded, len(encoded), 1, "gray")
assert geometry["declared"]["width"] == 1
assert geometry["declared"]["height"] == 2
assert geometry["candidate"]["width"] == len(encoded)
assert raw.qr()["status"] == "no_detection"
```

Structured results can contain native `bytes` or `Image` objects, so they are not
necessarily directly JSON-serializable. Convert those fields explicitly when
serializing; do not replace binary values with guessed UTF-8 text.

## Work with PCM WAVE data

For a self-contained WAVE fixture, follow the
[CLI fixture instructions](cli.md#create-small-image-and-audio-fixtures). Then:

```python
import cube

wave_info = cube.inspect_wave_file("sample.wav")
assert wave_info["format"]["sample_rate"] == 8000
samples = cube.wave_file_samples("sample.wav", channels=[0], frames=4, include_values=True)
assert samples["values"] == [0, 16384, 0, -16384]
assert isinstance(samples["data"], bytes)
envelope = cube.wave_file_envelope("sample.wav", channels=[0], bucket_frames=2)
assert len(envelope["buckets"]) == 2
```

Functions without `_file` accept encoded WAVE bytes. Frame ranges and channel
indexes are explicit; waveform data is an integer min/max series, not a plot.
[Audio analysis](operations.md#audio-analysis) describes supported formats and units.

## Discover more operations

```python
import cube

encodings = cube.operations("encoding")
assert any(item["id"] == "encoding.base64.decode" for item in encodings)
xor = cube.describe_operation("binary.xor")
assert xor["input_type"] == "bytes"
assert xor["output_type"] == "bytes"
assert xor["pipeline"] == {"version": 1}
assert cube.describe_operation("file.info.inspect")["pipeline"] is None
```

`operations()` returns descriptors sorted by identifier; the optional category is
an exact filter. Descriptors describe supported inputs, outputs and parameters.
`pipeline` is a version object for an executable bounded adapter, or `None` when
no adapter exists. Use that exact version in a step; normal preflight and resource
limits still apply. Use ordinary functions or `Image`/`InvestigationSession`
methods for direct SDK calls; their signatures are in the shipped stubs.
`pipeline_adapters()` lists the same executable subset with its descriptors.

## Handle invalid input

```python
import cube

try:
    cube.hex_decode("zz")
except ValueError as error:
    assert error.code == "invalid_input"
    assert error.operation == "encoding.hex.decode"

try:
    cube.inspect_file("missing-cube-example.bin")
except OSError as error:
    assert error.code == "io"
```

Operation validation errors use `ValueError` with `code`, `operation`, `parameter`
and `offset` attributes where available. Pipeline errors add `step` context.
File I/O failures use `OSError` in the file APIs. Incorrect Python argument types
raise `TypeError`; integers outside a binding's numeric range raise `OverflowError`
(for example a Caesar shift outside signed 32-bit range). These Python argument
errors need not have operation-context attributes. Correct invalid input or choose
a smaller selection when a resource limit is reported.

## Compose a pipeline or recipe

```python
import cube

steps = [
    {"operation": "encoding.hex.encode", "version": 1},
    {"operation": "encoding.hex.decode", "version": 1},
]
plan = cube.validate_pipeline(steps, "bytes")
assert plan["output_type"] == "bytes"
result = cube.execute_pipeline(steps, b"\x00\xff", capture=[0])
assert result["output"] == b"\x00\xff"
assert result["intermediates"] == {0: "00ff"}
```

Validation checks every step before execution. Results keep native `bytes`/`str`;
intermediate capture is opt-in. `limits` lowers budgets. An optional
`CancellationToken` supports cooperative cancellation; it is sticky, so use a new
token for each run. [Composition contracts](operations.md#compose-operations)
define the shared types, versions and limits.

After saving the [recipe example](operations.md#recipe-files) as `roundtrip.json`:

```python
from pathlib import Path
import cube

document = Path("roundtrip.json").read_text(encoding="utf-8")
assert cube.validate_recipe(document)["format_version"] == 1
assert cube.execute_recipe(document, {"payload": b"\x00\xff"})["output"] == b"\x00\xff"
assert cube.check_recipe(document, {"payload": b"\x00\xff"}, b"\x00\xff")["passed"]
```

Bindings are values supplied by your program. Recipe definitions do not read
files, execute shell commands or grant network access.

### Inspect bytes in a pipeline

```python
import cube

result = cube.execute_pipeline(
    [{"operation": "encoding.hex.decode", "version": 1},
     {"operation": "file.hex.inspect", "version": 1, "parameters": {"width": 1}}],
    "00ff", capture=[0],
)
assert result["intermediates"][0] == b"\x00\xff"
assert result["output_type"] == "forensic_report"
assert result["output"]["data"]["rows"][1]["bytes"] == b"\xff"
```

Signature and printable-string adapters return the other concrete report variants.
Reports retain offsets and completeness; they do not validate embedded files or
permit filesystem access. `pipeline_adapters()` declares each adapter's input and
output types; direct reader-based APIs retain their own descriptor types.

## Keep an investigation record

```python
import json
import cube

profile = json.dumps({"format_version": 1, "depth": "preview", "identify": True})
data = b"\x00original synthetic text\x00\xff"
plan = cube.plan_investigation(len(data), profile)
result = cube.investigate(data, profile)
session = result["session"]
record = session.to_json()
assert cube.InvestigationSession.from_json(record).replay(data)
```

The result contains `plan`, diagnostic `metrics` and an `InvestigationSession`.
No file is saved automatically. Inspect the record's `status` and `replay_gaps`;
budget/cancellation outcomes may be partial. To record a specific pipeline:

```python
import cube

session = cube.InvestigationSession(b"\x00\xff", identify=True)
run_id = session.execute_pipeline(
    b"\x00\xff", [{"operation": "encoding.hex.encode", "version": 1}],
    retain_result=True,
)
session.add_hypothesis("A candidate worth inspecting")
selected = session.to_json(include_results=[run_id], include_hypotheses=True)
session.discard_result(run_id)
```

Raw inputs and paths are never recorded. Results and nonempty parameters are
omitted by default; memory retention and export are separate choices.
`record_parameters=True` retains keys in memory, while
`to_json(include_parameters=[run_id])` selects them for export. Use
`redact_parameters(run_id)` to remove them. Already omitted data cannot be restored.
See [profile, record and replay limits](operations.md#investigation-profiles-and-records).

## API reference and development

The included [type stubs](../bindings/python/python/cube/_native.pyi) are the Python
API signature reference. Use the [Operations Guide](operations.md) for behavior
and constraints, or [Contributing](../CONTRIBUTING.md) for rebuilding, tests and
the Rust API reference.

### Numerical spectrum

```python
import struct
import cube
from io import BytesIO
import wave

buffer = BytesIO()
with wave.open(buffer, "wb") as writer:
    writer.setnchannels(1)
    writer.setsampwidth(2)
    writer.setframerate(8000)
    writer.writeframes(struct.pack("<8h", *([0] * 8)))
spectrum = cube.wave_spectrum(buffer.getvalue(), fft_size=8, channels=[0])
assert spectrum["frequencies_hz"] == [0, 1000, 2000, 3000, 4000]
assert spectrum["amplitudes"] == [[0.0] * 5]
gram = cube.wave_spectrogram(buffer.getvalue(), fft_size=8, hop_size=4, final_window="pad")
assert gram["frame_starts"] == [0, 4]
assert gram["valid_frames"] == [8, 4]
packed = cube.wave_bits(buffer.getvalue(), bits=[0])
assert packed["data"] == b"\x00"
assert packed["valid_bits"] == 8
reversed_wav = cube.reverse_wave(buffer.getvalue())
assert cube.wave_samples(cube.reverse_wave(reversed_wav))["data"] == bytes(16)
assert cube.detect_dtmf(buffer.getvalue())["events"] == []
```

Selections use the existing WAVE/PCM rules. The result retains channels, absolute
start frame, window, units and the actual sample rate. See the
[audio analysis contract](operations.md#audio-analysis) for normalization and limits.

### Inspect tone intervals before transcription

```python
import math
import struct
import wave
from io import BytesIO
import cube

values = [round(12000 * math.sin(math.tau * 700 * i / 8000)) for i in range(800)]
buffer = BytesIO()
with wave.open(buffer, "wb") as writer:
    writer.setnchannels(1)
    writer.setsampwidth(2)
    writer.setframerate(8000)
    writer.writeframes(struct.pack("<800h", *values))
intervals = cube.detect_morse_intervals(buffer.getvalue(), frequency_hz=700)
assert "text" not in intervals
transcript = cube.transcribe_morse(intervals["intervals"], sample_rate=8000, unit_ms=100)
assert transcript["tokens"] == "." and transcript["text"] == "E"
```

The frequency and dot unit are explicit assumptions. Detection retains uncertain
intervals; transcription returns no text when timings are ambiguous. DTMF likewise
returns timed candidates/evidence through `detect_dtmf`, without a forced string.

### Inspect compressed-audio metadata

```python
from pathlib import Path
import cube

metadata = cube.inspect_audio_metadata(Path("sample.flac").read_bytes())
assert metadata["format"] == "flac"
assert metadata["verification"] == "metadata_only_audio_payload_unverified"
```

This is the bounded native-FLAC metadata profile; it does not decode audio or
artwork. MP3/Ogg are unsupported. A null duration means the source did not declare
a sample count; declared checksums are never presented as verified.
