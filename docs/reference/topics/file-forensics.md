# File forensics

Inspect file metadata, find signatures and strings, compare bytes, or recover an
explicit range. Path-based inputs must be stable regular files. Byte offsets start at zero;
a range ends exclusively at `offset + length`.

## Inspection and comparison

| Operation | Result | Interpretation |
| --- | --- | --- |
| Inspect | Size, timestamps and up to 12 header bytes | EXIF is read only when requested |
| Signature | Supported format candidates at the header or embedded offsets | A match does not prove validity or hidden content; unknown headers have no match |
| Strings | Runs of ASCII bytes `0x20–0x7e` | No Unicode detection |
| Compare | Exact difference count and a bounded listing | Missing tail bytes are `null`/`None`, not zero; differences are not errors |
| EXIF | Typed, repeated and unknown tags from JPEG/classic TIFF | Absent EXIF succeeds; malformed metadata fails |

Signature rules recognize common PNG, JPEG, GIF, PDF, ZIP, gzip, WAVE and
classic-TIFF markers. Embedded scans identify candidates without extracting them.

For example, inspect signatures and printable strings in your own file:

```sh
cube file signature sample.bin --json
cube file strings sample.bin --min-length 4 --json
```

### Supplied byte artifacts

The Rust `artifact_info` API and JavaScript `file.info.inspect` version 2 can inspect
owned bytes without a filesystem reference. Size and up to 12 header bytes come
from those bytes. The result identifies `source_kind: "artifact"`; `regular_file`,
`modified` and `created` are null because OS facts are unknown. Native `file_info`
and `file_info_reader` retain their regular-file checks and available timestamps.
A reference string alone never grants filesystem access.

### Inspection limits

| Resource | Maximum |
| --- | --- |
| Embedded-signature or string scan | 64 MiB |
| One string run | 1 MiB |
| Accumulated string text | 16 MiB |
| EXIF metadata prefix | 1 MiB |
| Byte-returning extraction | 16 MiB |
| Streaming extraction | 1 GiB |

Listings can stop before all results are returned. Check `complete`. EXIF tags
may contain sensitive information already present in the source.

## Appended data

Appended-data detection finds a validated logical boundary, then counts trailing
bytes without scanning them. Trailing bytes alone do not establish hidden data.

| Format | Boundary validation | Unsupported cases |
| --- | --- | --- |
| PNG | Chunk framing and CRCs | APNG; unknown critical chunks |
| JPEG | Baseline sequential Huffman, one or three components; segment/table/frame/scan relationships, stuffing and restart order | Progressive or arithmetic processes |
| GIF | Palettes, raster rectangles, extensions and sub-block lengths | Plain-text rendering |
| ZIP | Classic archive validation described below | Incomplete candidate searches |

End-marker bytes inside framed payloads do not establish a boundary. Pixel and
entropy data are not decoded. Work to the logical end is limited to 64 MiB and
10,000 framing records.

For compatibility, reports retain the `file.png_boundary` semantic type and PNG
field names. For other formats, `validated_chunks` counts framing records.

## Validate a ZIP candidate

`file zip` / `validate_zip` validates a classic ZIP at a caller-selected origin.
Local headers, central directory, EOCD counts and offsets, names, CRC/size fields
and entry extents must agree. Records must be contiguous and nonoverlapping;
archive offsets are relative to the selected origin.

### ZIP support

| Supported | Rejected |
| --- | --- |
| Stored and deflate methods | Encryption and ZIP64 |
| UTF-8 names and comments | Multi-disk archives |
| Signed 32-bit data descriptors | Unsigned descriptors |
| Origin-relative offsets | Alternate self-extracting offset variants |

Names remain inert `name_hex` data. Nothing is unpacked.

### ZIP limits and result meaning

| Resource | Maximum |
| --- | --- |
| Suffix searched after the origin | 16 MiB |
| EOCD candidates | 64 |
| Entries | 1,000 |
| Cumulative local/central header visits | 10,000 |
| Metadata | 1 MiB |
| Total declared uncompressed data | 64 MiB |

- `scanned_bytes` and `candidate_scan_complete` describe the searched suffix.
  Interpretations outside that prefix are not ruled out.
- `payload_verified` is always false. Compressed content, actual CRC integrity
  and authenticity are not verified.
- Boundary detection and carving require a complete search, rather than choosing
  a potentially ambiguous extent.
- `file appended` uses this validator for ZIP at offset zero. ZIP validation is
  available through direct APIs, without a pipeline adapter.

## Recover selected embedded bytes

`file carve` / `carve_file` requires an explicit offset and a new destination.

| Length setting | Selection |
| --- | --- |
| Omitted | Revalidate a supported PNG, JPEG, GIF or ZIP at that origin |
| Explicit | Copy the exact raw range without claiming format validation |

Validation uses coordinates relative to the candidate; reports retain absolute
source offsets. If a ZIP suffix cannot be searched completely within the budget,
select an explicit known range.

### Output and failure behavior

The output budget defaults to 16 MiB and can be adjusted up to the 1 GiB streaming
hard limit. Copying uses 64 KiB chunks.

- Validate range, size and format before creating output.
- Reject existing destinations and aliases of the source.
- Remove a newly created partial output on failure.
- Fail if the source size or modification time changes. Keep input stable;
  recovery is not an atomic filesystem snapshot.

The report includes source size, selected range, bytes written, selection mode,
format evidence when validated, and SHA-256 of recovered bytes. The digest
identifies content, not trust.

Sources are read-only. Carving does not decompress or execute content, and does
not automatically retain paths or bytes in investigation sessions.
