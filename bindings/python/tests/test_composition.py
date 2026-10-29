"""Ordered composition through the rebuilt native Rust extension."""

import cube
import pytest


def test_text_and_binary_order_preserves_exact_content():
    assert (
        cube.concat_text("e\u0301\x00", "世界\r\n", " | ") == "e\u0301\x00 | 世界\r\n"
    )
    assert cube.concat_text("", "", ":") == ":"
    assert cube.concat_text("A", "B") == "AB"
    assert cube.concat_bytes(b"\x00\xff", b"\x80\x0a") == b"\x00\xff\x80\x0a"
    assert cube.concat_bytes(b"", b"") == b""


def test_composition_refuses_coercion_and_aggregate_overflow():
    with pytest.raises(TypeError):
        cube.concat_text(b"A", "B")
    with pytest.raises(TypeError):
        cube.concat_bytes("A", b"B")
    maximum = 1024 * 1024
    assert len(cube.concat_text("A" * (maximum - 1), "B")) == maximum
    assert len(cube.concat_bytes(b"A" * (maximum - 1), b"B")) == maximum
    for operation, args, identity in [
        (cube.concat_text, ("A" * maximum, "B"), "text.concat"),
        (cube.concat_text, ("", "", "A" * (maximum + 1)), "text.concat"),
        (cube.concat_bytes, (b"A" * maximum, b"B"), "bytes.concat"),
    ]:
        with pytest.raises(ValueError) as error:
            operation(*args)
        assert error.value.code == "resource_limit"
        assert error.value.operation == identity
