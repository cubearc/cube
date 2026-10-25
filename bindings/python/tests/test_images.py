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


def bmp_bytes():
    """Original 2x1 uncompressed BGR bitmap with two row-padding bytes."""
    pixels = bytes([3, 2, 1, 6, 5, 4, 0, 0])
    header = struct.pack("<2sIHHI", b"BM", 62, 0, 0, 54)
    dib = struct.pack("<IiiHHIIiiII", 40, 2, 1, 1, 24, 0, 8, 0, 0, 0, 0)
    return header + dib + pixels


def test_bmp_decoding_provenance_and_lossless_export(tmp_path):
    source = bmp_bytes()
    path = tmp_path / "original.bmp"
    path.write_bytes(source)
    image = cube.Image.open(path)
    assert image.samples == bytes([1, 2, 3, 4, 5, 6])
    assert image.info()["format"] == "bmp"
    assert image.provenance()["source_samples_preserved"] is False
    assert image.provenance()["color_management"] == "none"
    output = tmp_path / "decoded.png"
    image.save_png(output)
    assert cube.Image.open(output).samples == image.samples
    assert path.read_bytes() == source
    with pytest.raises(ValueError):
        cube.Image.decode(source[:-3])


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


def rgba_image():
    return cube.Image.from_samples(
        2,
        2,
        "rgba",
        bytes(
            [
                10,
                20,
                30,
                0,
                40,
                50,
                60,
                255,
                10,
                20,
                30,
                255,
                40,
                50,
                60,
                128,
            ]
        ),
    )


def test_pixels_channels_histograms_and_explicit_palette_views():
    image = rgba_image()
    assert image.pixel(1, 0) == [40, 50, 60, 255]
    assert image.channel("alpha")["data"] == bytes([0, 255, 255, 128])
    pixels = image.pixels(region=(1, 0, 1, 2), channels=["blue", "red"])
    assert pixels["data"] == bytes([60, 40, 60, 40])
    assert pixels["region"] == {"x": 1, "y": 0, "width": 1, "height": 2}
    assert len(image.histogram()["colors"]) == 4
    colors = image.histogram(channels=["red", "green", "blue"])["colors"]
    assert colors == [
        {"values": [10, 20, 30], "count": 2},
        {"values": [40, 50, 60], "count": 2},
    ]
    indexed = cube.Image.from_samples(
        2, 1, "indexed", b"\x00\x01", [[1, 2, 3], [1, 2, 3]]
    )
    assert indexed.channel("index")["data"] == b"\x00\x01"
    assert indexed.histogram(expand_palette=True)["colors"] == [
        {"values": [1, 2, 3, 255], "count": 2},
    ]
    for call in [
        lambda: image.pixel(2, 0),
        lambda: image.channel("index"),
        lambda: image.pixels(channels=["red", "red"]),
        lambda: image.pixels(region=(0, 0, 0, 1)),
        lambda: image.histogram(max_colors=1),
        lambda: indexed.channel("red"),
    ]:
        with pytest.raises(ValueError):
            call()


def test_comparison_preserves_alpha_and_bounded_differences():
    first = cube.Image.from_samples(2, 1, "rgba", bytes([1, 2, 3, 0, 4, 5, 6, 255]))
    second = cube.Image.from_samples(2, 1, "rgba", bytes([1, 2, 3, 255, 4, 50, 6, 255]))
    result = first.compare(second, max_results=1, mask=True)
    assert result["differing_pixels"] == 2
    assert result["complete"] is False
    assert result["mask"].samples == b"\xff\xff"
    assert first.compare(second, channels=["red", "blue"])["equal"] is True
    assert first.compare(first)["mask"] is None
    with pytest.raises(ValueError):
        first.compare(rgba_image())


def test_exact_geometry_color_transforms_and_immutable_sources(tmp_path):
    image = cube.Image.from_samples(2, 3, "gray", bytes([1, 2, 3, 4, 5, 6]))
    assert image.rotate(90).samples == bytes([5, 3, 1, 6, 4, 2])
    assert (image.rotate(90).width, image.rotate(90).height) == (3, 2)
    assert image.rotate(90).rotate(270).samples == image.samples
    assert image.mirror("horizontal").samples == bytes([2, 1, 4, 3, 6, 5])
    assert image.mirror("vertical").samples == bytes([5, 6, 3, 4, 1, 2])
    assert image.crop((1, 1, 1, 2)).samples == bytes([4, 6])
    for action in [
        lambda: image.rotate(45),
        lambda: image.mirror("other"),
        lambda: image.crop((1, 0, 2, 1)),
    ]:
        with pytest.raises(ValueError):
            action()
    image = cube.Image.from_samples(1, 1, "rgba", bytes([255, 0, 0, 7]))
    assert image.invert().samples == bytes([0, 255, 255, 7])
    assert image.invert().invert().samples == image.samples
    assert image.invert(channels=["alpha"]).samples[-1] == 248
    assert image.reorder(["blue", "green", "red", "alpha"]).samples == bytes(
        [0, 0, 255, 7]
    )
    assert image.grayscale().samples == bytes([77, 7])
    with pytest.raises(ValueError):
        image.reorder(["red", "red", "blue", "alpha"])
    path = tmp_path / "result.png"
    image.rotate(90).save_png(path)
    assert cube.Image.open(path).samples == image.samples
    with pytest.raises(OSError):
        image.invert().save_png(path)


def test_bit_planes_have_exact_values_packing_and_visualization():
    image = cube.Image.from_samples(3, 1, "gray", bytes([2, 0, 2]))
    plane = image.bit_plane("gray", 1, visualization=True)
    assert plane["pixels"]["data"] == b"\x01\x00\x01"
    assert plane["packed"] == b"\xa0"
    assert plane["valid_bits"] == 3
    assert plane["image"].samples == b"\xff\x00\xff"
    assert image.bit_plane("gray", 0)["image"] is None
    for channel, bit in [("alpha", 0), ("gray", 8)]:
        with pytest.raises(ValueError):
            image.bit_plane(channel, bit)


@pytest.mark.parametrize(
    "traversal,reverse,msb,lsb",
    [
        ("row", False, 0xA6, 0x65),
        ("row", True, 0x65, 0xA6),
        ("column", False, 0x9C, 0x39),
        ("column", True, 0x39, 0x9C),
    ],
)
def test_lsb_traversal_and_packing(traversal, reverse, msb, lsb):
    image = cube.Image.from_samples(4, 2, "gray", bytes([1, 0, 1, 0, 0, 1, 1, 0]))
    for bit_order, expected in [("msb", msb), ("lsb", lsb)]:
        result = image.lsb(traversal=traversal, reverse=reverse, bit_order=bit_order)
        assert result["data"] == bytes([expected])
        assert result["valid_bits"] == 8
    with pytest.raises(TypeError):
        image.lsb(unknown=True)


def test_lsb_binary_payload_selection_and_partial_byte_policies():
    payload = b"\x00\xff\x80"
    samples = bytes(
        0xF0 | ((byte >> bit) & 1) for byte in payload for bit in reversed(range(8))
    )
    image = cube.Image.from_samples(8, 3, "gray", samples)
    assert image.lsb()["data"] == payload
    assert cube.Image.decode(image.encode_png()).lsb()["data"] == payload
    partial = image.lsb(region=(0, 0, 1, 1), incomplete="pad")
    assert partial["valid_bits"] == 1
    assert partial["data"] == b"\x00"
    assert image.lsb(region=(0, 0, 1, 1), incomplete="drop")["data"] == b""
    for options in [
        dict(bits=[]),
        dict(bits=[0, 0]),
        dict(bits=[8]),
        dict(channels=[]),
        dict(channels=["alpha"]),
        dict(traversal="spiral"),
        dict(region=(0, 0, 1, 1)),
        dict(bit_order="other"),
        dict(incomplete="other"),
    ]:
        with pytest.raises(ValueError):
            image.lsb(**options)
    rgba = cube.Image.from_samples(1, 1, "rgba", bytes([0, 0, 0, 1]))
    assert (
        rgba.lsb(channels=["alpha"], bits=list(reversed(range(8))))["data"] == b"\x01"
    )
