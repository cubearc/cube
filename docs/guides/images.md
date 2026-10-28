# Inspect and transform an image

Inspect an image's pixels, extract a channel, or read selected bits as bytes.
Create `sample.png` with the [sample files guide](fixtures.md) first. It is a
4 × 2 RGB image with eight pixels.

## Inspect pixels and colors

```sh
cube image inspect sample.png --json
cube image histogram sample.png --channels red,green,blue --json
cube image pixels sample.png --channels red --hex
```

The first report gives the dimensions and sample layout. The histogram counts
colors and channel values; the last command lists the red samples in pixel order.

## Read the least significant bits

The sample image stores the byte `A` in the lowest bit of each red sample:

```sh
cube image lsb sample.png --channels red --bits 0 --stdout > extracted.bin
cube file hex extracted.bin
```

The result is one byte, `41` in hexadecimal. A different image may produce arbitrary
bytes. Choose channels, bit positions and traversal order to match what you want
to inspect.

## Save a transformed image

```sh
cube image rotate sample.png --degrees 90 --output rotated.png
cube image mirror sample.png --axis horizontal --output mirrored.png
cube image bitwise sample.png mirrored.png --operation xor --output combined.png --json
```

Rotation changes the dimensions to 2 × 4. The XOR comparison produces a new
image from the original and its mirror. These commands leave `sample.png` unchanged
and reject existing output paths.

## Exchange pixels as CSV

```sh
cube image csv export sample.png --output pixels.csv
cube image csv import --input pixels.csv --output restored.png --json
```

The restored image has the same samples. The PNG file itself may have different
encoded bytes.

For raw pixel layouts, color masks, image collections and extraction limits, see
[image analysis and steganography](../reference/topics/image-analysis-and-steganography.md).

<a id="analyze-an-image"></a>
<a id="analyze-an-image-1"></a>
<a id="cli"></a>
<a id="python"></a>
<a id="reference"></a>
