# Structural format fixtures

`python-force-zip64.zip` contains the five ASCII bytes `hello` in a stored
member named `hello.txt`. CPython's standard library writes a ZIP64 local
header and a classic-sized central record when requested on a small member.

```python
from zipfile import ZipFile, ZipInfo

with ZipFile("python-force-zip64.zip", "w") as archive:
    info = ZipInfo("hello.txt", (1980, 1, 1, 0, 0, 0))
    with archive.open(info, "w", force_zip64=True) as member:
        member.write(b"hello")
```

The sparse ZIP64 fixture in `zip64_archives.rs` synthesizes a record after 4 GiB
through a bounded reader. It supplies framing bytes only.
