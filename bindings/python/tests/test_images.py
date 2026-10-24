"""Original PNG fixtures built independently with Python's standard library."""

import struct
import zlib

import cube
import pytest
from cube import _native


def chunk(kind, data):
    return (
        struct.pack(">I", len(data))
        + kind
        + data
        + struct.pack(">I", zlib.crc32(kind + data))
    )


def png_bytes(width, height, color, samples, palette=None, transparency=None, depth=8):
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color]
    stride = width * channels * (2 if depth == 16 else 1)
    rows = b"".join(
        b"\0" + samples[y * stride : (y + 1) * stride] for y in range(height)
    )
    data = b"\x89PNG\r\n\x1a\n" + chunk(
        b"IHDR", struct.pack(">IIBBBBB", width, height, depth, color, 0, 0, 0)
    )
    if palette is not None:
        data += chunk(b"PLTE", palette)
    if transparency is not None:
        data += chunk(b"tRNS", transparency)
    return data + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


@pytest.mark.parametrize(
    ("model", "color", "samples", "palette", "trns"),
    [
        ("gray", 0, b"\0\x80\xff\x11", None, b"\0\x80"),
        ("gray_alpha", 4, bytes(range(8)), None, None),
        ("rgb", 2, bytes(range(12)), None, b"\0\0\0\x01\0\x02"),
        ("rgba", 6, bytes(range(16)), None, None),
        ("indexed", 3, b"\0\x01\x01\0", b"\x01\x02\x03\x04\x05\x06", b"\0\x80"),
    ],
)
def test_exact_png8_models_without_implicit_conversions(
    model, color, samples, palette, trns
):
    image = cube.Image.decode(png_bytes(2, 2, color, samples, palette, trns))
    assert image.width == image.height == 2
    assert image.samples == samples
    assert image.info()["color_model"] == model
    assert image.info()["transparency"] == list(trns or b"")
    assert cube.Image.decode(image.encode_png()).samples == samples
    assert image.exif() is None
    assert cube.Image is _native.Image
    with pytest.raises(AttributeError):
        image.width = 10


def test_palette_expansion_and_exclusive_outputs(tmp_path):
    image = cube.Image.from_samples(
        2, 1, "indexed", b"\0\x01", [[10, 20, 30], [40, 50, 60]], b"\0"
    )
    assert image.expand_palette().samples == bytes([10, 20, 30, 0, 40, 50, 60, 255])
    path = tmp_path / "image.png"
    image.save_png(path)
    assert cube.Image.open(path).samples == image.samples
    assert "2x1" in repr(image)
    with pytest.raises(OSError):
        image.save_png(path)
    for call in [
        lambda: cube.Image.from_samples(1, 1, "indexed", b"\x02", [[0, 0, 0]]),
        lambda: cube.Image.from_samples(0, 1, "gray", b""),
        lambda: cube.Image.from_samples(1, 1, "rgb", b"\0"),
    ]:
        with pytest.raises(ValueError):
            call()


def test_chunk_crc_evidence_and_unsupported_depth():
    original = png_bytes(1, 1, 0, b"\x7f")
    report = cube.png_chunks(original + b"tail")
    assert report["crc_valid"] is True
    assert report["boundary"]["trailing_bytes"] == 4
    assert report["chunks"][0]["offset"] == 8
    assert report["chunks"][0]["payload_hex"] is None
    corrupted = bytearray(original)
    corrupted[29] ^= 1
    with pytest.raises(ValueError):
        cube.png_chunks(bytes(corrupted))
    assert cube.png_chunks(bytes(corrupted), strict=False)["crc_valid"] is False
    with pytest.raises(ValueError) as error:
        cube.Image.decode(png_bytes(1, 1, 0, b"\x00\x01", depth=16))
    assert error.value.code == "unsupported_format"
