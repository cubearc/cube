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
