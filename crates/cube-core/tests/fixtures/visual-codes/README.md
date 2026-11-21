# Visual code known answers

The compact Aztec and PDF417 images are from Apache-2.0 ZXing 3.5.4 black-box
fixtures converted to grayscale PNG.

| Source fixture in `core/src/test/resources/blackbox` | Exact decoded text |
| --- | --- |
| `aztec-1/abc-19x19C.png` | `abcdefghijklmnopqrstuvwxyz` |
| `pdf417-1/01.png` | `This is PDF417` |

ZXing 3.5.4 Java writers generated the other lossless rasters with these inputs.

| Fixture | Writer input and settings |
| --- | --- |
| `datamatrix.png` | `0123456789`, ECC200, four pixels per module and six pixels of white margin. |
| `datamatrix-eci.png` | `43 75 62 65 00 ff` as ISO-8859-1, compact writer, four pixels per module and six pixels of margin. |
| `aztec-full.png` | `43 75 62 65 00 ff`, explicit ISO-8859-1, full layer 5, 33% correction, three pixels per module and six pixels of margin. |
| `upca.png` | `036000291452`, width-zero writer rendering, margin 24, doubled samples. |
| `code39.png` | `CODE39W`, where W is the Mod 43 check character, margin 24, doubled samples. |
| `code128.png` | `AB123456` then FNC1 then `Z`, compact writer, margin 24, doubled samples. |

`qr-masks-v1.json` uses eight independently encoded matrices from Python
qrcode 8.2 for version 1-L, payload `00 ff`, and masks 0–7. Its penalty terms
come from independent run grouping over those matrices.

[ZXing 3.5.4 source corpus](https://github.com/zxing/zxing/tree/zxing-3.5.4/core/src/test/resources/blackbox)
