"""Exact byte and fixed unsigned-unit vectors through the native core."""

import cube
import pytest


def test_logic_reverse_and_endian_are_binary_safe():
    data = bytes([0, 255, 0x55, 0xAA])
    assert cube.and_bytes(data, bytes([255, 15, 0xAA, 0x55])) == bytes([0, 15, 0, 0])
    assert cube.or_bytes(data, bytes([255, 15, 0xAA, 0x55])) == b"\xff" * 4
    assert cube.not_bytes(cube.not_bytes(data)) == data
    assert cube.reverse_bytes(cube.reverse_bytes(data)) == data
    assert cube.swap_endian(b"ABCD", unit_bytes=2) == b"BADC"
    assert cube.and_bytes(b"", b"") == b""
    with pytest.raises(ValueError):
        cube.and_bytes(data, b"x")


def test_fixed_width_unsigned_shifts_and_pipeline_preflight():
    assert cube.shift_left(b"\x01\xff", 1, unit_bytes=2) == b"\x03\xfe"
    assert cube.shift_right(b"\x01\x00", 1, unit_bytes=2) == b"\x00\x80"
    assert cube.shift_left(b"\xff", 8) == b"\0"
    assert cube.shift_right(b"\xff" * 8, 64, unit_bytes=8) == b"\0" * 8
    assert (
        cube.execute_pipeline(
            [
                {
                    "operation": "binary.swap_endian",
                    "version": 1,
                    "parameters": {"unit_bytes": 2},
                }
            ],
            b"ABCD",
        )["output"]
        == b"BADC"
    )
    for kwargs in [{"bits": 9}, {"bits": 1, "unit_bytes": 0}]:
        with pytest.raises(ValueError):
            cube.shift_left(b"x", **kwargs)
    with pytest.raises(ValueError):
        cube.swap_endian(b"x", unit_bytes=2)
