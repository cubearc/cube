"""Original exact text fixtures shared through the compiled Rust SDK."""

import cube
import pytest


def test_a1z26_ascii_whitespace_case_and_pipeline_contracts():
    assert cube.a1z26_encode("Hi Cube\r\n") == "8-9 3-21-2-5\r\n"
    assert cube.a1z26_decode("8-9 3-21-2-5\r\n") == "HI CUBE\r\n"
    for text in ["", "AZ", " \t\n\v\f\r", "THE QUICK BROWN FOX"]:
        assert cube.a1z26_decode(cube.a1z26_encode(text)) == text
    result = cube.execute_pipeline(
        [{"operation": "symbol.a1z26.encode", "version": 1}], "Cube"
    )
    assert result["output"] == "3-21-2-5"
    for value in ["0", "27", "-1", "01", "1--2", "1-", "é"]:
        with pytest.raises(ValueError):
            cube.a1z26_decode(value)
    with pytest.raises(ValueError):
        cube.a1z26_encode("Hi!")
