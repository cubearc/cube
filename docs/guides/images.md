# Inspect and transform an image

Create an original input with the fixture guide, then inspect exact samples, channels or selected bits. Keep output paths new; use the reference for supported formats and pixel/bit traversal rules.

Prepare a small input with [original media fixtures](fixtures.md). Examples with collections, a second image or FLAC require those explicit caller-supplied inputs.

## CLI

### Analyze an image

```sh
cube image inspect sample.png --json
cube image histogram sample.png --channels red,green,blue --json
cube image colors sample.png --colors 020000 --output exact-mask.png
cube image lsb sample.png --channels red --traversal zigzag --start-corner top_right --stdout > zigzag.bin
cube image pixels sample.png --channels red --traversal spiral_cw --hex
cube image pixels sample.png --channels red --stdout | cube pipeline run --steps '[{"operation":"encoding.hex.encode","version":1}]'
cube image channel sample.png --channel red --output red.bin
cube image rotate sample.png --degrees 90 --output rotated.png
cube image lsb sample.png --channels red --bits 0 --stdout > extracted.bin
```


```sh
cube image mirror sample.png --axis horizontal --output mirrored.png
cube image bitwise sample.png mirrored.png --operation xor --output combined.png --json
```


```sh
python -c 'from pathlib import Path; Path("pixels.bin").write_bytes(bytes([3,2,1,0,6,5,4,0]))'
cube image raw pixels.bin --width 1 --height 2 --layout bgr --row-stride 4 --output reconstructed.png
```


```sh
cube image raw pixels.bin --width 2 --height 1 --layout rgba --compare-header
```


```sh
cube image csv export sample.png --output pixels.csv
cube image csv import --input pixels.csv --output restored.png --json
cube image csv export sample.png --stdout | cube image csv import --output copied.png
```


```sh
cube image pairs first.png second.png third.png --operation xor --plan
mkdir pairs
cube image pairs first.png second.png third.png --operation xor --output-dir pairs --json
```


```sh
mkdir layers
cube image color-layers sample.png --output-dir layers
```


## Python

### Analyze an image

```python
import cube

image = cube.Image.from_samples(2, 1, "rgb", b"\xff\x00\x00\x00\x00\xff")
assert image.pixel(0, 0) == [255, 0, 0]
assert image.channel("red")["data"] == b"\xff\x00"
histogram = image.histogram(channels=["red", "green", "blue"])
assert histogram["pixels"] == 2
assert len(histogram["colors"]) == 2
rotated = image.rotate(90)
assert (rotated.width, rotated.height) == (1, 2)
assert (image.width, image.height) == (2, 1)
image.save_png("two-pixels.png")
assert cube.Image.open("two-pixels.png").samples == image.samples
```


```python
import cube

left = cube.Image.from_samples(1, 1, "rgba", bytes([10, 20, 30, 0]))
right = cube.Image.from_samples(1, 1, "rgba", bytes([10, 20, 30, 255]))
combined = left.bitwise(right, "xor")
assert combined.samples == bytes([0, 0, 0, 255])
assert left.bitwise(right, "and").samples == left.samples
assert left.bitwise(right, "or").samples == right.samples
combined.save_png("combined.png")
```


```python
raw = cube.Image.from_raw(bytes([3, 2, 1, 0, 6, 5, 4, 0]), 1, 2, "bgr", row_stride=4)
assert raw.pixel(0, 1) == [4, 5, 6]
mask = raw.exact_colors([[4, 5, 6]])
assert mask["selected_pixels"] == 1
assert mask["image"].samples == b"\x00\xff"
stream = raw.lsb(bits=list(range(7, -1, -1)), traversal="spiral_ccw")
assert stream["data"] == raw.samples
ordered = raw.pixels(channels=["blue", "red"], traversal="spiral_cw")
assert ordered["data"] == bytes([3, 1, 6, 4])
encoded = raw.encode_png()
geometry = cube.Image.inspect_layout(encoded, len(encoded), 1, "gray")
assert geometry["declared"]["width"] == 1
assert geometry["declared"]["height"] == 2
assert geometry["candidate"]["width"] == len(encoded)
assert raw.qr()["status"] == "no_detection"
```


```python
from cube import Image

source = Image.from_samples(2, 1, "rgba", bytes([1, 2, 3, 0, 4, 5, 6, 255]))
csv_text = source.to_csv()
restored = Image.from_csv(csv_text)
assert restored.samples == source.samples
```


## Reference

- [Image analysis and steganography](../reference/topics/image-analysis-and-steganography.md)
