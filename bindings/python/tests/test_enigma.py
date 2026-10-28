"""Native historical machine behavior and safe typed settings transport."""

import cube
import pytest


def settings():
    return {
        "profile": "enigma_i",
        "rotors": ["I", "II", "III"],
        "reflector": "B",
        "rings": [1, 1, 1],
        "windows": "AAA",
        "plugboard": [],
    }


def test_native_known_answer_inverse_and_final_state():
    p = settings()
    result = cube.enigma("AAAAA", p)
    assert result == {"text": "BDZGO", "final_windows": "AAF", "trace": []}
    assert cube.enigma(result["text"], p)["text"] == "AAAAA"
    p.update(rotors=["III", "II", "I"], windows="ADO", trace=True)
    assert [s["windows"] for s in cube.enigma("AAAAAA", p)["trace"]] == [
        "ADP",
        "ADQ",
        "AER",
        "BFS",
        "BFT",
        "BFU",
    ]


def test_native_invalid_settings_have_typed_safe_errors():
    p = settings()
    p["plugboard"] = ["AB", "AC"]
    with pytest.raises(ValueError) as caught:
        cube.enigma("SECRET", p)
    assert caught.value.code == "invalid_parameter"
    assert caught.value.operation == "cipher.enigma"
    assert caught.value.parameter == "plugboard"
    assert "SECRET" not in str(caught.value)
    p = settings()
    p["profile"] = "m4"
    with pytest.raises(ValueError):
        cube.enigma("AAAAA", p)


def test_native_preservation_and_limits():
    p = settings()
    with pytest.raises(ValueError):
        cube.enigma("a éA", p)
    p["text_mode"] = "preserve"
    assert cube.enigma("a éA", p)["text"] == "b éD"
    p["trace"] = True
    assert len(cube.enigma("A" * 65536, p)["trace"]) == 65536
    with pytest.raises(ValueError) as caught:
        cube.enigma("A" * 65537, p)
    assert caught.value.code == "resource_limit"
    with pytest.raises(ValueError):
        cube.enigma("A" * (1048576 + 1), settings())
