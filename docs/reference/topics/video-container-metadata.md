# Video container metadata

Inspect declared container metadata, extract selected baseline-JPEG frames,
compare frames or copy supported PCM audio into WAVE. Keep reader inputs stable
through each call; Cube does not take a full-file snapshot.

## Container inspection

`video.container.inspect` reads unfragmented MP4/MOV and WebM without a media
decoder. Results describe headers, not the validity of media packets, codec
configuration or checksums.

| Result | Interpretation |
| --- | --- |
| Track and codec IDs | Declared metadata |
| MP4 display dimensions | Original 16.16 values, separate from coded dimensions |
| Duration and time base | Exact integer ticks; derived seconds use floating point |
| Unavailable fields | `null` |

### Supported container subsets

| Container | Supported | Rejected |
| --- | --- | --- |
| MP4/MOV | Version 0/1 time headers, ordinary/extended atom lengths, one sample description per track | Fragmentation, encrypted entries, versioned QuickTime audio entries, zero-sized nested atoms |
| WebM | EBML DocType, Info, Tracks, known-sized children; Segment may extend to source end | Unknown-sized children, content encodings/encryption, nondefault track timestamp scaling |

MP4 handler metadata must precede sample descriptions. Unknown bodies and media
payloads are skipped using checked extents; they are not validated.

### Inspection limits

| Resource | Maximum |
| --- | --- |
| Physical input | 1 GiB |
| Records / tracks / depth | 10,000 / 32 / 8 |
| Cumulative metadata reads | 2 MiB |
| Field / WebM identifier | 4,096 / 128 bytes |

Only known header paths are traversed; full sample tables and packets are not
allocated. Format references:
[QuickTime](https://developer.apple.com/documentation/quicktime-file-format),
[EBML](https://www.rfc-editor.org/rfc/rfc8794) and
[WebM](https://www.webmproject.org/docs/container/).

## JPEG frame extraction

`video.frames.extract` supports self-contained MOV/ISO-BMFF baseline-JPEG
(`jpeg`) tracks. Select one track ID and 1–8 unique zero-based frame indexes;
results follow caller order.

### Frame-index validation

| Structure | Requirement |
| --- | --- |
| Description and data references | Self-contained version zero |
| Sample tables | Consistent `stsz`, `stsc`, `stco`/`co64` and `stts` |
| Sample ranges | Inside `mdat`, without overlap |
| Timing | Exact decode times |
| Optional `stss` | Every JPEG must be a sync sample |
| JPEG | Framing fills the sample; dimensions match its description |

Edit lists, composition offsets, fragmentation, encryption, external references,
description extensions and other sample-table features are unsupported. Pixel
coordinates are stored JPEG coordinates; display matrices are not applied.

### Extraction limits

| Resource | Maximum |
| --- | --- |
| Physical input | 1 GiB |
| Combined atoms/table rows | 10,000 |
| Samples | 10,000 |
| Metadata reads | 2 MiB |
| Selected JPEG | 2 MiB each; 16 MiB total |
| Conservative decoded RGBA | 16 MiB per frame; 64 MiB total |
| Decoder internal allocation | 64 MiB |
| CLI batch PNG output | 96 MiB |

Only selected JPEGs are read; intervening frames are not decoded. Baseline JPEG
framing, header and decode limits also apply. Cancellation is checked between
calls, not during a codec call; these limits are not a wall-clock timeout.

### Frame results and export

Each result contains an Image and provenance:

- Track and frame index.
- Decode start, duration and time base in exact ticks.
- Source offset and length.
- Encoded JPEG SHA-256.

There is no timestamp-seek API or playback validation. PNG export preserves the
decoded pixels of a lossy JPEG source, not the original JPEG bytes.

The CLI reserves all new output names exclusively and removes only its newly
created files on batch failure. SDK extraction performs no writes.

## Frame comparison

`video.frames.compare` accepts up to eight explicit pairs from the same MOV/JPEG
subset. It extracts their union once, with at most eight images, then compares
exact native samples or explicitly selected channels.

| Rule | Behavior |
| --- | --- |
| Dimensions | Must match |
| Transforms | No implicit resize or display transform |
| Comparison | Exact; no fuzzy or perceptual metric |
| Difference listing | 1–10,000 global records; default 1,000 |
| Counts | Exact even when listings are incomplete |
| Work | At most 8 Mi compared pixel pairs |
| Masks | No mask allocations |

Pair reports retain both frames' extraction, timing and hash provenance.
Previously extracted PNGs can use image comparison directly; keep their
extraction manifest.

Rust `comparison::compare_selected` accepts supplied Frame records and labels
their provenance as caller data. It does not authenticate that data.

## PCM audio extraction

`video.audio.extract` copies one explicitly selected self-contained MOV/ISO-BMFF
`raw ` PCM8 mono/stereo track into minimal RIFF/WAVE.

### Supported track

| Field | Requirement |
| --- | --- |
| Sound description | Version zero, 28 bytes |
| Samples | Unsigned 8-bit |
| Compression / packet fields | Zero |
| Sample rate | Integer from 1 to 65,535 |
| `stsz` | Fixed width equal to channel count |
| `stts` | Delta 1 |
| Time scale | Equal to sample rate |

Other codecs, widths, legacy table conventions, description extensions, edits,
composition offsets, fragmentation, encryption and external references fail.
The subset follows the
[QuickTime sound description](https://developer.apple.com/documentation/quicktime-file-format/sound_sample_description_version_0).

### Audio extraction limits

All selected-track ranges and relationships are validated before copying.

| Resource | Maximum |
| --- | --- |
| Physical input | 1 GiB |
| Metadata reads | 2 MiB |
| Cumulative atoms/table rows | 10,000 |
| Sample frames / duration | 1 Mi / 300 seconds |
| Temporary index memory | 64 MiB |
| Coalesced source ranges | 10,000 |
| PCM | 8 MiB |
| WAVE output | PCM size plus header/padding |

Coalesced ranges preserve sample order, including physically reordered chunks.
Samples, channel grouping and rate remain unchanged. There is no decoding,
normalization, resampling or byte conversion; ancillary chunks are omitted.

### Audio results and output

Results contain WAV bytes plus track, format, exact timing, source ranges,
PCM SHA-256 and WAV SHA-256. The CLI requires a new file or explicit binary stdout;
SDK extraction performs no writes. The generated WAV can enter ordinary audio
operations.

## Pipeline support

Video reader and image operations require direct calls. They have no pipeline
adapters and grant no recipe file access. Successful inspection or extraction
does not validate unselected media packets.
