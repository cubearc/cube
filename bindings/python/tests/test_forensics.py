"""Use only original synthetic files and exercise the actual Rust extension."""

import io

import cube
import pytest


def test_signatures_scan_and_inspect(tmp_path):
    path = tmp_path / "misleading.jpg"
    data = b"abc\x00\xff%PDF-1.7\x00PK\x03\x04"
    path.write_bytes(data)
    assert cube.file_signatures(b"\x89PNG\r\n\x1a\n")[0]["format"] == "png"
    assert cube.file_signatures(b"unknown") == []
    info = cube.inspect_file(path)
    assert info["size"] == len(data)
    assert info["regular_file"] is True
    assert info["signatures"] == []
    scan = cube.scan_file(path)
    assert [c["offset"] for c in scan["candidates"]] == [5, 14]
    assert scan["complete"] is True
    assert cube.scan_file(path, format="zip")["candidates"][0]["offset"] == 14
    assert cube.scan_file(path, max_results=1)["complete"] is False
    assert path.read_bytes() == data
    with pytest.raises(ValueError) as error:
        cube.scan_file(path, format="missing")
    assert error.value.code == "invalid_parameter"
    assert error.value.operation == "file.signature.scan"
    with pytest.raises(OSError) as error:
        cube.inspect_file(tmp_path / "missing")
    assert error.value.code == "io"
    with pytest.raises(ValueError):
        cube.inspect_file(tmp_path)


def test_exact_range_extraction_and_streaming(tmp_path):
    path = tmp_path / "source"
    data = b"\x00\xff\x80\n" * 20000
    path.write_bytes(data)
    target = tmp_path / "target"
    assert cube.extract_bytes(data, 1, 3) == b"\xff\x80\n"
    assert cube.extract_bytes(data, len(data), 0) == b""
    assert cube.extract_file_range(path, target, 1, 3) == 3
    assert target.read_bytes() == b"\xff\x80\n"
    sink = io.BytesIO()
    assert cube.copy_file_range(path, sink, 0, len(data)) == len(data)
    assert sink.getvalue() == data
    with pytest.raises(OSError):
        cube.extract_file_range(path, target, 0, 1)
    with pytest.raises(OSError):
        cube.extract_file_range(path, path, 0, 1)
    assert path.read_bytes() == data
    with pytest.raises(ValueError):
        cube.extract_bytes(data, 0, len(data) + 1)
    with pytest.raises(OverflowError):
        cube.extract_bytes(data, -1, 1)


class InvalidWriter:
    def write(self, data):
        return len(data) + 1

    def flush(self):
        pass


def test_stream_writer_failure_is_explicit(tmp_path):
    path = tmp_path / "input"
    path.write_bytes(b"data")
    with pytest.raises(OSError):
        cube.copy_file_range(path, InvalidWriter(), 0, 4)


def test_hex_strings_and_comparison_preserve_exact_values(tmp_path):
    first = tmp_path / "first"
    second = tmp_path / "second"
    first.write_bytes(b"\x00hello world\xfftail")
    second.write_bytes(b"\x00hello World\xff")
    rows = cube.file_hex(first, offset=1, length=12, width=5)
    assert rows[0]["offset"] == 1
    assert rows[0]["bytes"] == b"hello"
    assert rows[0]["hex"] == "68656c6c6f"
    assert b"".join(r["bytes"] for r in rows) == b"hello world\xff"
    result = cube.file_strings(first)
    assert result["complete"] is True
    assert result["strings"] == [
        {"offset": 1, "length": 11, "text": "hello world"},
        {"offset": 13, "length": 4, "text": "tail"},
    ]
    diff = cube.compare_files(first, second, max_results=1)
    assert diff["equal"] is False
    assert diff["differing_bytes"] == 5
    assert diff["complete"] is False
    assert diff["differences"][0] == {"offset": 7, "left": 119, "right": 87}
    tail = cube.compare_files(first, second, offset=13)
    assert tail["differences"][0]["right"] is None
    for call in [
        lambda: cube.file_hex(first, width=0),
        lambda: cube.file_strings(first, min_length=0),
        lambda: cube.compare_files(first, second, max_results=0),
    ]:
        with pytest.raises(ValueError):
            call()


def synthetic_png(trailer=b""):
    import struct
    import zlib

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload))
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(b"\x00\x00\x00\x00"))
        + chunk(b"IEND", b"")
        + trailer
    )


def synthetic_tiff():
    import struct

    return (
        b"II\x2a\x00"
        + struct.pack("<I", 8)
        + struct.pack("<H", 1)
        + struct.pack("<HHI", 0x010F, 2, 4)
        + b"Cube"
        + struct.pack("<I", 0)
    )


def test_exif_is_explicit_and_preserves_typed_values(tmp_path):
    data = synthetic_tiff()
    path = tmp_path / "input.tiff"
    path.write_bytes(data)
    assert "tags" not in cube.inspect_file(path)
    metadata = cube.file_exif(path)
    assert metadata == cube.inspect_exif(data)
    assert metadata["present"] is True
    assert metadata["tags"][0]["tag"] == 0x010F
    assert metadata["tags"][0]["value"] == {
        "type": "ascii",
        "values": [[67, 117, 98, 101]],
    }
    assert cube.inspect_exif(b"\xff\xd8\xff\xd9")["present"] is False
    with pytest.raises(ValueError) as error:
        cube.inspect_exif(b"unknown")
    assert error.value.code == "unsupported_format"
    with pytest.raises(ValueError) as error:
        cube.inspect_exif(b"II\x2a\x00\xff\xff\xff\x7f")
    assert error.value.code == "invalid_input"
    assert path.read_bytes() == data


def test_appended_data_requires_valid_png_boundaries(tmp_path):
    path = tmp_path / "input.png"
    image = synthetic_png()
    path.write_bytes(image + b"\x00\xffhidden")
    result = cube.appended_data(path)
    assert result["logical_end"] == len(image)
    assert result["trailing_bytes"] == 8
    assert result["trailing_offset"] == len(image)
    corrupted = bytearray(image)
    corrupted[29] ^= 1
    path.write_bytes(corrupted)
    with pytest.raises(ValueError) as error:
        cube.appended_data(path)
    assert error.value.code == "invalid_input"
    assert error.value.operation == "file.appended.detect"
    path.write_bytes(b"not a png file")
    with pytest.raises(ValueError) as error:
        cube.appended_data(path)
    assert error.value.code == "unsupported_format"


def test_appended_gif_framing_and_malformed_inputs(tmp_path):
    gif = (
        b"GIF89a\x01\0\x01\0\x80\0\0\0\0\0\xff\xff\xff"
        b"\x2c\0\0\0\0\x01\0\x01\0\0\x02\x02\x44\x01\0\x3b"
    )
    path = tmp_path / "artifact"
    path.write_bytes(gif + b"tail")
    result = cube.appended_data(path)
    assert result["format"] == "gif"
    assert result["logical_end"] == len(gif)
    assert result["trailing_offset"] == len(gif)
    assert result["trailing_bytes"] == 4
    assert "not decoded" in result["evidence"]
    assert path.read_bytes() == gif + b"tail"
    for data in [gif[:-1], b"\xff\xd8\xff\xd9", b"unknown"]:
        path.write_bytes(data)
        with pytest.raises(ValueError) as error:
            cube.appended_data(path)
        assert error.value.operation == "file.appended.detect"


def test_embedded_zip_structure_uses_exact_candidate_origin(tmp_path):
    import zipfile

    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("sample.bin", b"Cube\0\xff")
        archive.comment = b"decoy PK\x05\x06 comment"
    encoded = buffer.getvalue()
    source = b"host" + encoded + b"tail"
    path = tmp_path / "artifact"
    path.write_bytes(source)
    report = cube.validate_zip(source, offset=4)
    assert cube.validate_zip_file(path, offset=4) == report
    assert report["length"] == len(encoded)
    assert report["offset"] == 4
    assert report["trailing_bytes"] == 4
    assert report["payload_verified"] is False
    assert report["entries"][0]["name_hex"] == b"sample.bin".hex()
    assert report["entries"][0]["method"] == 8
    assert path.read_bytes() == source
    for bad in [encoded[:-1], b"PK\x03\x04false", b"unknown"]:
        with pytest.raises(ValueError) as error:
            cube.validate_zip(bad)
        assert error.value.operation == "file.zip.validate"
    assert cube.describe_operation("file.zip.validate")["pipeline"] is None


def test_explicit_carving_provenance_and_no_overwrite(tmp_path):
    archive = b"PK\x05\x06" + b"\0" * 18
    source = tmp_path / "host"
    target = tmp_path / "recovered"
    source.write_bytes(b"host" + archive + b"tail")
    report = cube.carve_file(source, target, 4, max_bytes=22)
    assert target.read_bytes() == archive
    assert report["offset"] == 4
    assert report["length"] == report["bytes_written"] == 22
    assert report["source_size"] == 30
    assert report["selection"] == "candidate"
    assert report["format"] == "zip"
    assert report["sha256"] == cube.sha256(archive)
    with pytest.raises(OSError):
        cube.carve_file(source, target, 4)
    assert source.read_bytes() == b"host" + archive + b"tail"
    for offset, length, budget in [(30, 1, 100), (4, 22, 21)]:
        missing = tmp_path / "missing"
        with pytest.raises(ValueError) as error:
            cube.carve_file(source, missing, offset, length=length, max_bytes=budget)
        assert error.value.operation == "file.carve"
        assert not missing.exists()
    raw = cube.carve_file(source, tmp_path / "raw", 1, length=2)
    assert raw["format"] is None
    assert raw["selection"] == "range"
    assert cube.describe_operation("file.carve")["pipeline"] is None


def test_documented_forensic_recovery_workflow(tmp_path):
    import zipfile

    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        info = zipfile.ZipInfo("note.txt", (1980, 1, 1, 0, 0, 0))
        info.create_system = 3
        info.external_attr = 0o600 << 16
        archive.writestr(info, b"Cube\x00\xff")
    encoded = buffer.getvalue()
    assert len(encoded) == 120
    source = tmp_path / "mystery.bin"
    original = b"CubeArc\x00" + encoded + b"\x00tail"
    source.write_bytes(original)
    assert cube.inspect_file(source)["signatures"] == []
    assert cube.file_strings(source)["strings"][0]["text"] == "CubeArc"
    candidates = cube.scan_file(source, format="zip")
    assert any(item["offset"] == 8 for item in candidates["candidates"])
    archive = cube.validate_zip_file(source, offset=8)
    assert archive["length"] == 120
    assert archive["payload_verified"] is False
    recovered = tmp_path / "python-recovered.zip"
    report = cube.carve_file(source, recovered, 8, max_bytes=archive["length"])
    assert recovered.read_bytes() == source.read_bytes()[8:128] == encoded
    assert report["sha256"] == cube.sha256(recovered.read_bytes())
    result = cube.execute_pipeline(
        [
            {"operation": "encoding.hex.decode", "version": 1},
            {"operation": "file.signature.detect", "version": 1},
        ],
        "504b0304",
        capture=[0],
    )
    assert result["intermediates"][0] == b"PK\x03\x04"
    assert result["output"]["data"]["matches"][0]["format"] == "zip"
    assert source.read_bytes() == original
    with pytest.raises(OSError):
        cube.carve_file(source, recovered, 8)
