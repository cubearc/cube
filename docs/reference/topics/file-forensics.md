# File forensics

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

## Validate a ZIP candidate

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

## Recover selected embedded bytes

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
