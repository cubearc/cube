### Image analysis and steganography

Use `image` operations to inspect PNG structure, compare exact samples or extract
selected channels/bits. The [CLI fixture](../../guides/fixtures.md)
provides an original image with known low bits for these examples.

```sh
cube image histogram sample.png --channels red,green,blue --json
cube image bit-plane sample.png --channel red --bit 0 --stdout > plane.bin
```

PNG decoding preserves 8-bit grayscale, grayscale+alpha, RGB, RGBA and indexed samples.
Additional inputs are baseline Huffman JPEG (gray/RGB output), single-frame GIF
(RGBA canvas, transparent background), BMP (decoded RGB/RGBA), and static WebP
(RGB/RGBA). JPEG color-space conversion and lossy artifacts are decoder results,
not exact encoded/source samples. GIF/BMP palettes are expanded by their codecs.
`Image.provenance()` and `image inspect --json` label these representations.
Constructed and transformed results are labeled `constructed_8bit_samples`, without
claiming an encoded original or preserved source samples. For compatibility,
`Image.info().format` still says `png` for these objects' lossless export model;
use provenance for their actual origin. A subsequent decoded PNG describes that
new immediate PNG source, not its earlier transformation history.
Limits include 32 MiB encoded data, 16384 pixels per side, 4 Mi pixels and 16 MiB
of decoded samples. Decoder allocation requests are capped at 64 MiB where the
codec supports them; dimensions/output and encoded-input limits always apply.
GIF decoding examines at most two bounded frames to reject animation; animated
WebP is rejected from its header. Other formats/depths fail explicitly. No
ICC/gamma correction, EXIF orientation transform or premultiplication occurs.
GIF's first frame is placed on its transparent canvas. PNG chunk inspection checks
framing/CRCs without decoding pixels; bad-CRC reporting and payload export are opt-in.

Coordinates use a top-left origin; regions are `x,y,width,height`, entirely inside
the image. Selected channel order is preserved. Native indexed data stays palette
indexes; request expansion explicitly for RGBA. Histograms count exact selected
color tuples in sorted order, including alpha only when selected. Exceeding a
color budget fails, rather than approximating. Coordinate listings cap at 10000
pixels. Comparison requires equal dimensions and compatible selected samples.

`image.bitwise` combines two images with an explicitly selected `xor`, `and` or
`or`, reusing the byte transformations. Dimensions, 8-bit model/channel order,
palette and raw tRNS must match exactly; unequal sample buffers never cycle or pad.
Every physical sample participates, including alpha, without blending. Indexed
values combine as indices and must remain valid for the retained palette;
explicitly expand both inputs first when RGBA color values are intended.
Matching tRNS keys remain unchanged: output colors matching the retained key are
transparent. Other source metadata is dropped; provenance describes constructed
samples, not original compressed bytes. XOR can be reversed with the same second
image when all intermediate images remain valid; AND/OR are lossy.

The existing per-image pixel/dimension/sample limits apply to both inputs and the
result, so their sample buffers total at most 48 MiB. Decoder and PNG export limits
also apply. PNG export is lossless for resulting samples and creates a new file.
This operation is available through Rust, CLI and native Python, with no pipeline
adapter or new image pipeline value. Existing byte XOR and image comparison keep
their behavior.

`image.pairs` enumerates unique `(i,j), i<j` in caller order, reusing bitwise or
exact comparison APIs. Empty/single collections yield zero pairs; repeated object
identities fail (CLI also rejects duplicate canonical paths). Hard limits are eight
inputs, 28 pairs, 64 MiB retained input samples, 16 Mi pixel-pair work, 64 MiB
conservative PNG output and 1000 global difference records. All pairs, including
failed compatibility checks, count toward preflight. PNG bounds use the same
conservative per-image formula as color exports. The compact manifest is capped
at 512 KiB. Counts remain exact when difference listings are incomplete.
Pair errors do not discard other results; filesystem writes reuse the CLI export
reservation/rollback policy. No generic dispatcher or pipeline collection is added.

`image.colors.export` reuses the complete native-channel histogram and existing
single-color extraction. It returns canonical `color-<hex>.png` entries and counts;
absent or duplicate subsets fail. Native indices and physical alpha distinguish
colors until explicit palette expansion. Both color/output caps default to 16
(hard cap 64). Histogram plus extraction scans allow 16 Mi pixel-work; conservative
PNG bounds of `2*sample_bytes+4096` per image must total at most 64 MiB before work.
Actual encoded bytes are also checked. Rust/Python perform no implicit writes;
CLI reserves the complete collection plus manifest and rolls back its new files
on failure. This structured collection has no pipeline adapter.

Raw reconstruction supports unsigned 8-bit `gray`, `rgb`, `bgr`, `rgba` and `bgra`.
Width, height and physical layout are mandatory; offset defaults to zero and stride
to a packed row. Every row includes its declared padding, including the final row:
input length must equal `offset + height * stride`. Extra or missing bytes fail.
Only prefix and explicitly declared row padding are ignored. No byte-order option
is needed for one-byte samples. The ordinary image/input budgets apply. The result
uses canonical gray/RGB/RGBA samples, with lossless PNG export to a new file.
Raw images use direct APIs; no pipeline image value is introduced.

Alternative geometry is always a caller hypothesis. `image raw --compare-header`
and `Image.inspect_layout` report parsed header dimensions separately from the
validated raw candidate. Header-only evidence does not validate compressed data;
malformed recognized headers report a separate error, while unknown data has no
declared dimensions. PNG/JPEG/GIF/BMP/WebP header observations do not repair files.
Optional export creates a new PNG from the exact candidate layout.

PNG depths 1/2/4/16 remain unsupported by `Image`. The
[precision evaluation](https://github.com/cubearc/cube/issues/70) records native
packed-index/gray and big-endian 16-bit decoder fixtures, a compatibility strategy,
budgets and a proposed future representation. Implementation is gated on agreement
of that contract; no sample expansion/downsampling or public type migration occurs.

Exact-color selection matches complete native tuples, including physical alpha.
Select 1..=256 distinct colors. Masks preserve coordinates and use 255 for selected
pixels, zero elsewhere; RGBA layers keep selected original colors/alpha (including
tRNS) and use transparent black elsewhere. Transparent selected pixels still count
and retain their color bytes. Native indexed selection matches indices; explicitly
expand the palette to select RGBA colors, which can match multiple duplicate palette
entries. No fuzzy matching or hidden-message inference is performed.

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

Traversal also supports `diagonal`, `zigzag`, `spiral_cw` and `spiral_ccw`.
Diagonals visit increasing `x+y`, from bottom-left to top-right within each
anti-diagonal; zigzag reverses odd anti-diagonals. For a 3×3 image, zigzag visits
row-major indices `0,1,3,6,4,2,5,7,8`. Spirals proceed inward clockwise or
counterclockwise, starting rightward or downward respectively from the top left.
`start_corner` defaults to `top_left`; other corners reflect axes for linear/
diagonal traversal, or rotate the starting edge while preserving a spiral's named
direction. Regions retain absolute coordinates. Reverse reverses the final pixel
sequence only. Coordinates are generated with constant memory under image limits.

Pixel export uses the same traversal and corner options as LSB extraction, visiting
each coordinate before its selected channels. Outputs are raw sample bytes;
`--hex` renders exactly those bytes. Coordinate records retain source positions.
Existing native row-major pixel export remains unchanged. PNG channel visualization
requires default row order; other orders use bytes or records. Feed exported bytes
to existing byte transformations/pipelines explicitly. Encoded-image decoding is
kept outside recipes: image expansion/work budgets differ from byte adapters, and
discovery does not advertise an image pipeline adapter.

QR analysis accepts opaque supported images up to 262144 pixels and 1024 pixels
per side. A temporary view expands palettes, reuses the integer luma conversion,
then uses Quircs' Otsu thresholding. Transparent samples require explicit caller
preprocessing; no background is guessed. Detection bounds are 32 capstones,
65534 regions and eight grids; results flag reached detector limits and do not
promise exhaustive detection. `max_symbols` is 1..=8 and total payload is capped
at 64 KiB. Known fixtures cover two/four/eight pixels per module and quarter turns;
other scale, perspective, damage and contrast conditions may fail.

Reports distinguish no detection, detected but undecodable, decoded and partial
decode. Each symbol has raw payload bytes or a decoding error, plus available
corner/version/ECC/mask/type/ECI metadata. Binary bytes are preserved without text
encoding guesses. CLI JSON represents them as exact Hex; raw output requires
exactly one decoded symbol. No payload URL is followed or content executed.
Selections must be nonempty and distinct; output is capped at 16 MiB.
An extracted bit stream is a deterministic candidate, not proof of a hidden message.

### Pixel CSV v1

`image.csv.export` / `image.csv.import` exchange complete, small exact sample grids.
The first record is `cube_pixels,1,width,height,model,8`; the second fixes channel
order: `x,y,gray`, `x,y,gray,alpha`, `x,y,red,green,blue` or
`x,y,red,green,blue,alpha` for `gray`, `gray_alpha`, `rgb` or `rgba` respectively.
Coordinates are zero-based from the top left. Export is row-major; import allows
reordered rows but requires every declared coordinate exactly once.

```csv
cube_pixels,1,2,1,rgba,8
x,y,red,green,blue,alpha
0,0,1,2,3,0
1,0,4,5,6,255
```

LF/CRLF and an optional final newline are accepted. Every field may optionally be
wholly double-quoted. Embedded quotes, commas/newlines, blank records, BOM and
numeric whitespace/signs/prefixes/fractions are unsupported. Integers are decimal;
samples must be 0..255. Dimensions must be positive and fit normal image limits.
The additional limits are 10000 pixel records, 2 MiB input/output, 128 bytes per
physical row (excluding its line ending), 16 unquoted bytes per field and six
fields per record. Bounds precede allocation and incomplete images are never
returned. Errors identify the row/field where applicable.

The contract preserves dimensions/model/samples, including hidden RGB beneath
zero alpha; it does not preserve original encoding, EXIF, profiles or other
ancillary metadata. Indexed palettes and tRNS are explicitly rejected, rather
than silently expanded/dropped. Explicitly expanded indexed RGBA samples may be
exported as a new representation. Import constructs new 8-bit sample provenance.
Both operations are direct Rust/CLI/native Python APIs; neither is a pipeline
adapter because the engine has no image sample value.
