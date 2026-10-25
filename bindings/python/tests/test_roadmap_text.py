"""Original exact text fixtures shared through the compiled Rust SDK."""

import cube
import pytest


def test_word_line_selection_scalar_semantics_and_existing_extraction():
    assert cube.select_text("Cube Uses Binary Evidence", "word_first") == "CUBE"
    assert (
        cube.select_text("\tAlpha,\u2003βeta!\r\n\n🧊cube \n", "line_first") == "\t🧊"
    )
    assert cube.select_text("e\u0301 🧊雪", "word_last") == "\u0301雪"
    assert cube.select_text("", "line_last") == ""
    assert cube.extract_characters("abcdef", start=1, stride=2) == "bdf"
    assert len(cube.WORD_UNICODE_VERSION) == 3
    assert (
        cube.execute_pipeline(
            [
                {
                    "operation": "text.select",
                    "version": 1,
                    "parameters": {"selector": "word_first"},
                }
            ],
            "Cube Uses Binary Evidence",
        )["output"]
        == "CUBE"
    )
    with pytest.raises(ValueError):
        cube.select_text("text", "guess")


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
