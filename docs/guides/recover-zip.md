# Recover an embedded ZIP


Run this example in a new scratch directory. Python's standard library creates
an original ZIP fixture; Cube performs the investigation and recovery:

```sh
python - <<'PYTHON'
import io
import zipfile
from pathlib import Path

buffer = io.BytesIO()
with zipfile.ZipFile(buffer, "w") as archive:
    info = zipfile.ZipInfo("note.txt", (1980, 1, 1, 0, 0, 0))
    info.create_system = 3
    info.external_attr = 0o600 << 16
    archive.writestr(info, b"Cube\x00\xff")
encoded = buffer.getvalue()
assert len(encoded) == 120
Path("mystery.bin").write_bytes(b"CubeArc\x00" + encoded + b"\x00tail")
PYTHON
cube inspect mystery.bin --json
cube file hex mystery.bin --length 32 --json
cube file strings mystery.bin --json
cube file embedded mystery.bin --format zip --json
cube file zip mystery.bin --offset 8 --json
cube file carve mystery.bin --offset 8 --output recovered.zip --json
cube file extract mystery.bin --offset 8 --length 120 --output selected-range.zip
```

The host's unknown header is an observation. The PK match at offset 8 is a
candidate; ZIP structure validation establishes the supported archive range
`[8, 128)`, with five trailing bytes. Other PK matches are not automatically
independent valid archives. The report states that payload CRCs and compressed
content were not verified. Recovery revalidates framing, writes a new file and
returns provenance including SHA-256; the explicit range copy produces the same
120 bytes. Existing destinations are rejected and `mystery.bin` is unchanged.
Nothing is decompressed or executed.



Use [pipelines](pipelines.md) to compose byte transformations before inspection.
See the [file reference](../reference/topics/file-forensics.md) for formats and limits.

## Python



Use the original `mystery.bin` from the
[fixture above](#recover-an-embedded-zip):

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
