import random

import cube
import pytest


def test_native_fixed_width_independent_integer_oracles():
    rng = random.Random(19138)
    for width in [1, 2, 4, 8]:
        bits = width * 8
        mask = 2**bits - 1
        values = [0, 1, mask, mask - 1, 1 << (bits - 1)]
        values += [rng.getrandbits(bits) for _ in range(30)]
        data = b"".join(n.to_bytes(width, "big") for n in values)
        keys = values[::-1]
        operand = b"".join(n.to_bytes(width, "big") for n in keys)
        for count in [0, 1, bits - 1, bits, bits + 1, 2**64 - 1]:
            k = count % bits
            expected = b"".join(
                (((n << k) | (n >> ((bits - k) % bits))) & mask).to_bytes(width, "big")
                for n in values
            )
            left = cube.rotate_left(data, count, unit_bytes=width)
            assert left == expected
            assert cube.rotate_right(left, count, unit_bytes=width) == data
        for function, sign in [(cube.add_wrapping, 1), (cube.sub_wrapping, -1)]:
            assert function(data, operand, unit_bytes=width) == b"".join(
                ((a + sign * b) & mask).to_bytes(width, "big")
                for a, b in zip(values, keys, strict=True)
            )
    assert cube.xor_exact(b"\0\xff", b"\xff\x0f") == b"\xff\xf0"
    assert cube.xor_exact(b"", b"") == b""
    assert cube.xor(b"\0\0", b"\xff") == b"\xff\xff"
    assert cube.rotate_left(b"\x81", 1) == b"\x03"
    assert cube.rotate_right(b"\x03", 1) == b"\x81"
    assert cube.add_wrapping(b"\xff", b"\x01") == b"\0"
    assert cube.sub_wrapping(b"\0", b"\x01") == b"\xff"


@pytest.mark.parametrize("value", [-1, 2**64, True, 1.0, "1", None])
def test_native_counts_reject_coercion_and_narrowing(value):
    for function in [cube.rotate_left, cube.rotate_right]:
        with pytest.raises(ValueError) as exc:
            function(b"", value)
        assert exc.value.parameter == "count"


@pytest.mark.parametrize("width", [-1, 0, 3, 16, True, 1.0, 2**64])
def test_native_widths_are_explicit(width):
    for function in [cube.rotate_left, cube.rotate_right]:
        with pytest.raises(ValueError):
            function(b"", 0, unit_bytes=width)
    for function in [cube.add_wrapping, cube.sub_wrapping]:
        with pytest.raises(ValueError):
            function(b"", b"", unit_bytes=width)


def test_native_bounds_discovery_and_pipeline():
    for function in [cube.add_wrapping, cube.sub_wrapping, cube.xor_exact]:
        with pytest.raises(ValueError) as exc:
            function(b"A", b"")
        assert exc.value.code == "invalid_parameter"
    for name in [
        "rotate_left",
        "rotate_right",
        "xor_exact",
        "add_wrapping",
        "sub_wrapping",
    ]:
        descriptor = cube.describe_operation(f"binary.{name}")
        assert descriptor["pipeline"] == {"version": 1}
    too_big = bytes(16 * 1024**2 + 1)
    with pytest.raises(ValueError) as exc:
        cube.rotate_left(too_big, 0)
    assert exc.value.code == "resource_limit"
    with pytest.raises(ValueError):
        cube.add_wrapping(b"A", b"A", unit_bytes=2)


def test_native_new_adapters_compose_and_keep_pipeline_budgets():
    steps = [
        {
            "operation": "binary.rotate_left",
            "version": 1,
            "parameters": {"count": 17, "unit_bytes": 2},
        },
        {
            "operation": "binary.rotate_right",
            "version": 1,
            "parameters": {"count": 1, "unit_bytes": 2},
        },
        {
            "operation": "binary.add_wrapping",
            "version": 1,
            "parameters": {"key": {"base64": "AAE="}, "unit_bytes": 2},
        },
        {
            "operation": "binary.sub_wrapping",
            "version": 1,
            "parameters": {"key": {"base64": "AAE="}, "unit_bytes": 2},
        },
        {
            "operation": "binary.xor_exact",
            "version": 1,
            "parameters": {"key": {"base64": "//8="}},
        },
    ]
    r = cube.execute_pipeline(steps, b"\x80\x01", limits={"max_work_bytes": 22})
    assert r["output"] == b"\x7f\xfe"
    with pytest.raises(ValueError) as exc:
        cube.execute_pipeline(steps, b"\x80\x01", limits={"max_work_bytes": 21})
    assert exc.value.code == "resource_limit"
    with pytest.raises(ValueError):
        cube.execute_pipeline(steps, b"\x80")
