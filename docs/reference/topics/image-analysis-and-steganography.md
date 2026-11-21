# Image analysis and steganography

Inspect image structure, compare exact samples, or extract channels and bits.
Use the [sample-file recipe](../../guides/fixtures.md) for an image with known
low bits.

## Supported formats

| Input | Decoded representation |
| --- | --- |
| PNG | 8-bit gray, gray+alpha, RGB, RGBA or indexed samples, preserved |
| Baseline Huffman JPEG | Gray or RGB; decoder color conversion and lossy artifacts |
| Single-frame GIF | RGBA on a transparent canvas; palette expanded |
| BMP | RGB or RGBA; palette expanded |
| Static WebP | RGB or RGBA |

Animation and other formats or depths fail explicitly. GIF checks at most two
bounded frames to reject animation; animated WebP is rejected from its header.
PNG depths 1, 2, 4 and 16 are unsupported; see the
[precision evaluation](https://github.com/cubearc/cube/issues/70).

No ICC/gamma correction, EXIF orientation or premultiplication is applied.

### Image limits

| Resource | Maximum |
| --- | --- |
| Encoded input | 32 MiB |
| Width or height | 16,384 pixels |
| Pixels | 4 Mi |
| Decoded samples | 16 MiB |
| Decoder allocation request | 64 MiB where supported by the codec |

Dimensions, output and encoded-input limits always apply.

### Provenance

`Image.provenance()` and `image inspect --json` identify the decoded representation.
Constructed or transformed output is labeled `constructed_8bit_samples`, without
claiming original encoded bytes or preserved source samples.

`Image.info().format` reports `png` for the lossless export model, including
non-PNG inputs; use provenance for the origin. Decoding an exported PNG records
that immediate source, not its earlier transformation history.

For the warp and stereo profiles, Rust and Python retain provenance from a
supplied native `Image`. A logical `image.samples` port carries sample data
and reports constructed-sample provenance.

## Coordinates, channels and histograms

| Setting | Rule |
| --- | --- |
| Origin | Top left |
| Region | `x,y,width,height`, entirely inside the image |
| Channels | Nonempty, distinct selections; caller order preserved |
| Indexed samples | Palette indices until explicitly expanded to RGBA |
| Histogram | Exact selected tuples in sorted order; alpha only when selected |
| Color budget | Excess fails without approximation |
| Coordinate listing | At most 10,000 pixels |
| Byte output | At most 16 MiB |

For example, inspect RGB colors or extract the red channel's least significant bit:

```sh
cube image histogram sample.png --channels red,green,blue --json
cube image bit-plane sample.png --channel red --bit 0 --stdout > plane.bin
```

PNG chunk inspection checks framing and CRCs without decoding pixels.
Bad-CRC reporting and payload export require explicit options.

## Comparison and bitwise operations

Exact comparison requires equal dimensions and compatible selected samples.
`image.bitwise` combines two images with `xor`, `and` or `or`.

| Requirement | Behavior |
| --- | --- |
| Compatibility | Exact dimensions, 8-bit model/channel order, palette and raw tRNS |
| Samples | Every physical sample participates, including alpha; no blending |
| Length | Unequal buffers fail; no cycling or padding |
| Indexed output | Combined indices must remain valid for the palette |
| Transparency | Matching tRNS retained; colors matching its key are transparent |
| Metadata | Other metadata dropped; output has constructed-sample provenance |

Expand both inputs explicitly before combining RGBA values from indexed images.
XOR is reversible with the same second image when all intermediate images remain
valid. AND and OR are lossy.

Per-image limits apply to both inputs and output, totaling at most 48 MiB of sample
buffers. Decoder and export limits also apply. PNG export preserves resulting
samples and creates a new file.

### Pair collections

`image.pairs` enumerates `(i,j)` with `i < j` in caller order, using bitwise or exact
comparison. Empty and single-input collections yield no pairs. Repeated object
identities fail; the CLI also rejects duplicate canonical paths.

| Resource | Maximum |
| --- | --- |
| Inputs / pairs | 8 / 28 |
| Retained input samples | 64 MiB |
| Pixel-pair work | 16 Mi |
| Conservative PNG output | 64 MiB |
| Global difference records | 1,000 |
| Compact manifest | 512 KiB |

All pairs count toward preflight, including incompatible pairs. Counts remain
exact when listings are incomplete. A pair error does not discard other results.

## Exact-color selection and export

Select 1–256 distinct complete native color tuples, including physical alpha.
There is no fuzzy matching or hidden-message inference.

| Result | Selected pixels | Other pixels |
| --- | --- | --- |
| Mask | 255 | 0 |
| RGBA layer | Original colors and alpha, including tRNS | Transparent black |

Selected transparent pixels still count and retain their color bytes. Native
indexed selection matches indices. Explicit palette expansion selects RGBA colors,
which can match multiple duplicate palette entries.

### Color collections

`image.colors.export` uses the complete native-channel histogram and returns
`color-<hex>.png` entries with counts. Absent or duplicate requested subsets fail.
Native indices and physical alpha distinguish colors until expansion.

| Resource | Limit |
| --- | --- |
| Colors / outputs | Default 16 each; hard cap 64 |
| Histogram and extraction work | 16 Mi pixels |
| Conservative PNG size | `2 * sample_bytes + 4096` per image |
| Total PNG budget | 64 MiB, checked before work and against actual encoded bytes |

For pair and color collections, the CLI reserves every output and the manifest,
then removes its newly created files on failure. Rust and Python perform no
implicit writes.

## Raw reconstruction

Reconstruct unsigned 8-bit `gray`, `rgb`, `bgr`, `rgba` or `bgra` samples.
Width, height and physical layout are required.

| Setting | Default or requirement |
| --- | --- |
| Offset | 0 |
| Stride | Packed row |
| Input length | Exactly `offset + height * stride`, including final-row padding |
| Ignored bytes | Only the declared prefix and row padding |
| Output | Canonical gray, RGB or RGBA; lossless PNG export |

Missing or extra bytes fail. One-byte samples need no byte-order setting.
Ordinary input and image limits apply.

### Header comparison

`image raw --compare-header` and `Image.inspect_layout` report parsed header
dimensions separately from the raw candidate. Alternative geometry is a caller
hypothesis.

A malformed recognized header produces a separate error; unknown data has no
declared dimensions. Header evidence does not validate compressed data or repair
a file. Optional export creates a new PNG from the exact candidate layout.

## Sample transformations

| Transformation | Preservation |
| --- | --- |
| Rotate, mirror, permute channels | Reversible sample rearrangement |
| Invert | Alpha preserved unless selected |
| Crop | Discards pixels |
| Convert to grayscale | Loses color |

Saved PNGs preserve samples, palette and transparency while dropping other
metadata. Preserved samples do not imply byte-identical source files.

## Raster profiles

Image profiles take named ports. The [sample transformation guide](../../guides/images.md#run-an-image-profile)
shows the shared request shape in the CLI, Python and JavaScript. Each operation's
[reference](../operations.md#imageresize) lists its parameters and limits.

| Operation | Profile |
| --- | --- |
| `image.resize` | Nearest, bilinear or Catmull–Rom bicubic with half-pixel centers and clamped edges. |
| `image.warp` | An inverse affine or projective matrix maps destination pixel centers to source centers. Choose nearest or bilinear, with constant or clamped borders. |
| `image.color.convert` | Encoded sRGB to HSV or HSL, or linearized sRGB through Bradford adaptation to CIELAB D50. Alpha is reported separately. |
| `image.quality.measure` | MSE, PSNR or Gaussian 11 × 11 single-scale SSIM over the same region and selected physical channel. Exact matches have a null PSNR and a perfect-match flag. |
| `image.filter` | Select physical channels for Gaussian or median filtering. Indexed input needs explicit palette expansion. |
| `image.edge.response` / `image.edge.canny` | Signed Sobel, Prewitt or Laplacian samples, or a Canny mask with caller-provided low/high thresholds. |
| `image.mask.*` | Otsu threshold, rectangle or cross morphology, ordered components and selection from explicit flood seeds. Masks use 0 and 255. |
| `image.hough.lines` | Theta-major accumulator and peaks ordered by vote count, then bin position. |
| `image.template.match` | SSD or zero-mean NCC over every valid position. A zero-variance NCC window has a null score. |
| `image.stereo.separate` | Red/cyan channel separation or equal-width side-by-side views with explicit eye order. |
| `image.stereo.anaglyph` | March 2009 Dubois red–cyan projection of two equal opaque sRGB views. Clip the two linear projections before combining and converting back to sRGB. |
| `image.hash.perceptual` | `ahash8`, `dhash8` or `phash32-8-v1` return eight MSB-first bytes for comparing perceptual similarity. |

Resize and warp sample alpha independently of other channels. Warp defaults to
nearest and zero borders. Resize rounds clamped samples half up. Compare
normalized floating-point reports with absolute or relative tolerance of
`1e-9` and use `1e-6` absolute tolerance for Lab against external references.
Dubois output may differ by one sample level at a floating-point rounding
boundary. Integer counts, coordinates, bits and masks compare exactly.
[CSS Color 4, 26 October 2026](https://www.w3.org/TR/2026/CRD-css-color-4-20261026/),
[Wang et al. on SSIM](https://www.cns.nyu.edu/~lcv/ssim/index.html),
[Dubois, March 2009](https://www.site.uottawa.ca/~edubois/anaglyph/LeastSquaresHowToPhotoshop.pdf)

## Traversal and native samples

`image.traversal.order` lists coordinates using an explicit corner, order and
reversal. Serpentine alternates rows. Hilbert and Morton require square regions
with power-of-two sides. Hilbert reflects the mathematical bottom-left path
to the selected image corner. These three traversal names can also be used in
pixel export, LSB extraction and scalar-grid rendering.
`image.traversal.permute` checks a bijection over a rectangular region. Each
entry gives the row-major source index for that destination position.

`image.png.scanlines` requires a complete noninterlaced 8-bit PNG. The
`filtered` profile keeps each row's filter byte with the filtered bytes.
`reconstructed` returns the physical bytes after reversing the row filters.
Both report the filtered offsets and SHA-256 of the original source.
`image.gif.indices` reads frame zero of a complete single-frame GIF, returning
the deinterlaced indices, active palette, transparency index and frame offset.
Duplicate palette colors retain their original indices.
[PNG specification](https://www.w3.org/TR/png-3/),
[GIF89a specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt)

## Visual codes

`image.qr.encode` encodes QR Model 2 versions 1–40. Choose L, M, Q or H
correction and provide exact bytes or explicit numeric, alphanumeric and byte
segments. Supply mask 0–7 or omit it to evaluate all eight. The report includes
the four penalty terms for each evaluated mask. The lowest mask number breaks
ties. The `modules` port is one byte per module in row-major order, with 1 for
dark. An optional gray raster defaults to a four-module quiet zone and a scale
of one. [Nayuki QR Code generator 1.8.0](https://github.com/nayuki/QR-Code-generator/tree/v1.8.0)

`image.barcode.scan` scans only the supplied gray/RGB image rows or explicit
unsigned scanlines. Select `ean13`, `upca`, `code39` or `code128` plus
`forward` or `reverse`. EAN-13, UPC-A and Code 128 require a valid checksum.
Code 39 requires `none` or `mod43`. The report keeps positions in the caller's
sample coordinates and includes Code 128 codewords. The batch is bounded to
1,024 lines, 4,096 samples per line, 1 Mi total samples, 16 Mi work units and
64 decoded symbols.

`image.barcode.decode` takes opaque gray crops and a format of `data_matrix`,
`aztec` or `pdf417`. Supply a clockwise normalization angle as `"0"`,
`"90"`, `"180"` or `"270"`. Results give crop-relative points, decoded text,
and original binary material when the decoder exposes it. Data Matrix and
Aztec expose corrected bytes or bits and byte segments as available. PDF417
exposes corrected integer codewords, correction level and the number of
corrected errors. Read the crop status before using any payload.

### Optical profile qualification

The 21 November 2026 review selected [RXing 0.9.3](https://crates.io/crates/rxing/0.9.3)
(Apache-2.0, Rust 1.85 minimum) for all three formats. Each call accepts up to
eight candidates, up to 262,144 pixels and a 1,024-pixel side per crop, with
32 Mi work units and 64 KiB of decoded material across the batch.

| Format | Selected scope and work limit |
| --- | --- |
| Data Matrix ECC200 | One rectified symbol with a quiet zone. Use the pure-symbol reader and the pinned backend's ECC200 Reed–Solomon decoder. Structured-append control words are rejected. |
| Aztec | One compact or full symbol. Use the pinned detector and its mode and data error correction. Budget the crop passes and bounded layer/codeword work before detection. |
| Standard PDF417 | One rectified symbol with fixed horizontal module width and consistent row indicators. Use RXing's codeword table, prime-field error correction and compaction parser. Charge one correction attempt using the measured codeword count and parity before entering the backend. Macro and compact framing fall outside this profile. |

The qualification fixtures include original ZXing Java encodings and the
upstream compact Aztec and PDF417 samples. The tests exercise orientation,
binary payloads, correction evidence, blank and adversarial crops.
[ZXing 3.5.4](https://github.com/zxing/zxing/tree/zxing-3.5.4)

## Bit planes and LSB extraction

A bit plane returns one 0/1 byte per selected pixel. Optional packing is MSB-first;
visualization maps 0/1 to 0/255.

LSB extraction visits pixels, then selected channels, then selected bit positions.

| Option | Default or behavior |
| --- | --- |
| Bit | 0, the least significant bit |
| Traversal | Row-major, forward |
| Channels | Native order |
| Packing | MSB-first |
| Reverse | Reverse pixel order only |
| Incomplete final byte | Reject; explicit drop or zero-pad available |

An extracted bit stream is a deterministic candidate, not proof of a hidden message.

### Traversal order

| Traversal | Order from top left |
| --- | --- |
| `diagonal` | Increasing `x+y`; bottom-left to top-right within each anti-diagonal |
| `zigzag` | Same diagonals, reversing odd ones |
| `spiral_cw` | Inward clockwise, starting rightward |
| `spiral_ccw` | Inward counterclockwise, starting downward |

For a 3×3 image, zigzag visits row-major indices `0,1,3,6,4,2,5,7,8`.
`start_corner` defaults to `top_left`. Other corners reflect linear/diagonal
axes or rotate a spiral's starting edge while preserving its named direction.

Regions retain absolute coordinates. Reverse applies to the final pixel sequence.
Coordinates are generated with constant memory under image limits.

### Pixel export

Pixel export uses the same traversal and corner options, visiting each coordinate
before its selected channels. Raw output contains sample bytes; `--hex` displays
those exact bytes. Coordinate records retain source positions.

PNG channel visualization requires default row order. Other orders use bytes or
records. Exported bytes can enter ordinary byte transformations and pipelines;
encoded-image decoding and image results have no pipeline adapters.

## JPEG coefficient parity

`cube image jpeg-coefficients` reads the original encoded bytes of an
eight-bit baseline JPEG with one complete scan and one or three components.
Choose `--profile baseline_ac_parity` and the `--component-id` stored in its
frame header. The operation visits the selected plane's real blocks in
row-major order and reads the magnitude parity of nonzero AC coefficients
in JPEG zigzag order. Whole padded MCU blocks and DC coefficients are
outside the selected sequence.

Set `--start-bit` to an offset in that sequence and `--max-bits` to the
maximum desired count. The JSON result gives packed bits and an evidence
report with dimensions, block counts, the number of candidate positions
and the number of bits returned. Bits are packed most significant first.
Only the declared `bit_count` bits are valid in the last byte.

The input cap is 16 MiB and the traversal cap is 1,048,576 coefficients.
Each result contains at most 1,048,576 bits. The operation has no linear
pipeline adapter. Recompression or editing the JPEG can change the stored
coefficients.

## QR analysis

QR analysis accepts opaque supported images. It expands palettes in a temporary
view, applies integer luma conversion and uses Quircs' Otsu thresholding.
Transparent samples require explicit preprocessing; no background is guessed.

| Resource | Maximum |
| --- | --- |
| Pixels / side | 262,144 / 1,024 |
| Detector capstones / regions / grids | 32 / 65,534 / 8 |
| `max_symbols` | 1–8 |
| Total payload | 64 KiB |

Reports flag reached detector limits; detection is not guaranteed exhaustive.
Known fixtures cover 2, 4 and 8 pixels per module and quarter turns. Other scale,
perspective, damage or contrast conditions can fail.

### QR results

Reports distinguish no detection, detected but undecodable, decoded and partial
decode. Each symbol contains payload bytes or an error, plus available corners,
version, ECC, mask, type and ECI metadata.

Payload bytes are preserved without guessed text encoding. CLI JSON uses exact
hex; raw output requires exactly one decoded symbol. URLs are never followed and
payloads are never executed.

## Pixel CSV v1

`image.csv.export` and `image.csv.import` exchange complete, small sample grids.
The first record is `cube_pixels,1,width,height,model,8`.

| Model | Second record |
| --- | --- |
| `gray` | `x,y,gray` |
| `gray_alpha` | `x,y,gray,alpha` |
| `rgb` | `x,y,red,green,blue` |
| `rgba` | `x,y,red,green,blue,alpha` |

Coordinates are zero-based from the top left. Export is row-major; import permits
reordered rows but requires every coordinate exactly once.

```csv
cube_pixels,1,2,1,rgba,8
x,y,red,green,blue,alpha
0,0,1,2,3,0
1,0,4,5,6,255
```

### CSV grammar and limits

| Rule | Requirement |
| --- | --- |
| Lines | LF or CRLF; optional final newline; no blank records or BOM |
| Quoting | Whole fields may be double-quoted; no embedded quotes, commas or newlines |
| Numbers | Decimal only; no whitespace, signs, prefixes or fractions |
| Samples / dimensions | 0–255 / positive and within image limits |
| Pixel records | At most 10,000 |
| Input or output | At most 2 MiB |
| Physical row | At most 128 bytes, excluding line ending |
| Field / record | At most 16 unquoted bytes / six fields |

Bounds are checked before allocation. Incomplete images are never returned;
errors identify a row or field when applicable.

CSV preserves dimensions, model and samples, including RGB under zero alpha.
It does not preserve original encoding, EXIF, profiles or other metadata.
Indexed palettes and tRNS are rejected; explicitly expanded RGBA can be exported
as a new representation.

Import produces constructed 8-bit samples. CSV, image collections and other image
operations use direct APIs; the pipeline engine has no image sample value.
