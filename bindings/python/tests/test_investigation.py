import cube
import pytest


def test_triage_is_explained_deterministic_and_bounded():
    data = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
    result = cube.triage_input(data)
    assert result == cube.triage_input(data)
    assert result["inspected_bytes"] == 12
    assert result["input_size"] == len(data)
    suggestion = next(
        s for s in result["suggestions"] if s["operation"] == "image.png.inspect"
    )
    assert suggestion["rule"] == "triage.png_chunks.v1"
    assert suggestion["evidence"] == ["fact-2"]
    assert "alone does not validate" in suggestion["uncertainty"]
    assert "evidence_hex" not in str(result)
    with pytest.raises(ValueError) as error:
        cube.triage_input(data, max_read_bytes=11)
    assert error.value.code == "resource_limit"
    assert cube.triage_input(b"", max_read_bytes=0)["input_size"] == 0
