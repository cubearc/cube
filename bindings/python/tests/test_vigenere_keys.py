"""The new native key schedules execute independently verified profiles."""

import copy

import cube
import pytest


@pytest.mark.parametrize(
    ("encode", "decode", "parameters"),
    [
        (
            cube.interrupted_key_encode,
            cube.interrupted_key_decode,
            {"key": "KEY", "resets": [0, 3]},
        ),
        (
            cube.progressive_key_encode,
            cube.progressive_key_decode,
            {"key": "KEY", "increment": -(2**31)},
        ),
        (
            cube.vigenere_multiplicative_encode,
            cube.vigenere_multiplicative_decode,
            {"key": "KEY", "multiplier": 25},
        ),
    ],
)
def test_unicode_roundtrip_resets_and_caller_parameters_are_not_mutated(
    encode, decode, parameters
):
    text = "Aa é! e\u0301世界\0AbC"
    before = copy.deepcopy(parameters)
    encoded = encode(text, parameters)
    assert decode(encoded, parameters) == text
    assert encode(text, parameters) == encoded
    assert parameters == before


@pytest.mark.parametrize("resets", [[1, 1], [2, 1], [3], [-1], [2**32], [True]])
def test_invalid_reset_schedules_are_typed_failures(resets):
    with pytest.raises(ValueError) as caught:
        cube.interrupted_key_encode("ABC", {"key": "KEY", "resets": resets})
    assert caught.value.operation == "cipher.interrupted_key.encode"


@pytest.mark.parametrize("multiplier", [0, 2, 13, 26, 255, True, "7"])
def test_multiplicative_requires_an_explicit_unit_integer(multiplier):
    with pytest.raises(ValueError) as caught:
        cube.vigenere_multiplicative_encode(
            "", {"key": "KEY", "multiplier": multiplier}
        )
    assert caught.value.operation == "cipher.vigenere_multiplicative.encode"


def test_required_fields_unknown_fields_and_exact_native_limits():
    for function in [
        cube.interrupted_key_encode,
        cube.progressive_key_encode,
        cube.vigenere_multiplicative_encode,
    ]:
        with pytest.raises(ValueError):
            function("ABC", {"key": "KEY"})
    with pytest.raises(ValueError):
        cube.progressive_key_encode(
            "ABC", {"key": "KEY", "increment": 1, "unknown": True}
        )
    with pytest.raises(ValueError):
        cube.progressive_key_encode("", {"key": "A", "increment": 2**31})
    assert cube.progressive_key_encode("", {"key": "A" * 65536, "increment": 0}) == ""
    with pytest.raises(ValueError) as caught:
        cube.progressive_key_encode("", {"key": "A" * 65537, "increment": 0})
    assert caught.value.code == "resource_limit"
    assert cube.interrupted_key_encode("", {"key": "A", "resets": []}) == ""
    with pytest.raises(ValueError):
        cube.interrupted_key_encode("", {"key": "A", "resets": [0]})
    assert (
        cube.vigenere_multiplicative_encode("é" * 524288, {"key": "A", "multiplier": 1})
        == "é" * 524288
    )
    with pytest.raises(ValueError) as caught:
        cube.vigenere_multiplicative_encode(
            "é" * 524288 + "A", {"key": "A", "multiplier": 1}
        )
    assert caught.value.code == "resource_limit"
