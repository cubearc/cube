"""Native adapters share the independent Core/CLI known-answer corpus."""

import json
from pathlib import Path

import cube
import pytest

CASES = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/poly/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["function"])
def test_native_known_answers_and_contracts(case):
    function = getattr(cube, case["function"])
    actual = function(case["inputs"]["text"], case["parameters"])
    expected = case["outputs"]
    assert actual == (expected["text"] if len(expected) == 1 else expected)
    descriptor = cube.describe_operation(case["operation"], include_contracts=True)
    assert descriptor["contract"]["operation_version"] == 1


@pytest.mark.parametrize("key", ["", "ABC", "12 3"])
def test_gronsfeld_rejects_bad_digits_without_echoing_private_data(key):
    with pytest.raises(ValueError) as caught:
        cube.gronsfeld_encode("ABC", {"key": key})
    assert caught.value.code == "invalid_parameter"


def test_native_parameters_are_strict_and_preservation_is_exact():
    with pytest.raises(ValueError):
        cube.trithemius_encode("A", {"start": 0, "step": 1, "extra": True})
    text = "Aa !\0é e\u0301世界"
    parameters = {"seed": "KEY", "feedback": "ciphertext"}
    encoded = cube.autokey_encode(text, parameters)
    assert cube.autokey_decode(encoded, parameters) == text
    with pytest.raises(ValueError):
        cube.running_key_encode("AB", {"key": "A"})


def test_padding_is_visible_and_state_resets():
    result = cube.playfair_encode("XX", {"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ"})
    assert result["prepared"] == "XQXQ"
    assert result["fillers"] == [1, 3]
    recovered = cube.playfair_decode(
        result["text"], {"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ"}
    )
    assert recovered["text"] == "XQXQ"
    assert recovered["fillers"] == []
    parameters = {"deck": list(range(1, 55))}
    assert cube.solitaire_encode("AAAAAAAAAA", parameters) == "EXKYIZSGEH"
    assert cube.solitaire_encode("AAAAAAAAAA", parameters) == "EXKYIZSGEH"
    with pytest.raises(ValueError) as caught:
        cube.solitaire_encode("A", {**parameters, "max_steps": 0})
    assert caught.value.code == "resource_limit"


def test_published_slidefair_wraps_the_key_per_digraph():
    plain = "THESLIDEFAIRCANBEUSEDWITHVIGENEREVARIANTORBEAUFORT"
    cipher = "EWKMCRNUAFCXTJYQMMYYFUTIGWZPKHJMPKBSAIECKVCFMIILCI"
    parameters = {"key": "DIGRAPH"}
    assert cube.slidefair_encode(plain, parameters) == cipher
    assert cube.slidefair_decode(cipher, parameters) == plain
