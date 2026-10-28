### Video container metadata

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
