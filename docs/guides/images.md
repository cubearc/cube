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

## Run an image profile

The named request below generates a QR symbol for two exact bytes. Save it as
`request.json`. Base64 represents the bytes in the CLI's JSON transport.

```json
{
  "inputs": {"data": {"base64": "AP8="}},
  "parameters": {"ecc": "L", "version": 1, "mask": 0, "raster": {"scale": 4}}
}
```

```sh
cube image profile qr-encode --input request.json --port raster --output qr.png
```

The command prints named outputs as JSON and creates a new PNG. In Python,
the binary port accepts `bytes` and the raster port returns a native `Image`.

```python
from cube import execute_image_profile

result = execute_image_profile(
    "image.qr.encode",
    {"data": bytes([0, 255])},
    {"ecc": "L", "version": 1, "mask": 0, "raster": {"scale": 4}},
)
assert len(result["modules"]) == 441
result["raster"].save_png("qr-python.png")
```

JavaScript uses `Uint8Array` for binary ports. The same request works in Node.js
and in browsers loading the Wasm SDK through `@cubearc/cube/browser`.

```typescript
import { createCube } from "@cubearc/cube";

const cube = await createCube();
const qr = cube.execute({
  operation: "image.qr.encode",
  version: 1,
  inputs: { data: Uint8Array.of(0, 255) },
  parameters: { ecc: "L", version: 1, mask: 0, raster: { scale: 4 } },
});
if (qr.modules.length !== 441 || qr.raster?.width !== 116) {
  throw new Error("Unexpected QR geometry");
}
```

See the [image profiles reference](../reference/topics/image-analysis-and-steganography.md#raster-profiles)
for other profiles and their input requirements.

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
