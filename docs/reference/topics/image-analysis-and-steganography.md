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
