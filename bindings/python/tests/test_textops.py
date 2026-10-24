"""Original known answers and boundary behavior for text operations backed by Rust."""

import cube
import pytest


def test_classical_known_answers_unicode_and_round_trips():
    assert cube.atbash("ABC xyz") == "ZYX cba"
    assert cube.vigenere_encode("ATTACKATDAWN", "LEMON") == "LXFOPVEFRNHR"
    assert cube.vigenere_decode("LXFOPVEFRNHR", "lemon") == "ATTACKATDAWN"
    assert cube.vigenere_encode("A!éA🧊A", "BCD") == "B!éC🧊D"
    assert cube.beaufort("ABC xyz", "B") == "BAZ edc"
    assert cube.affine_encode("AFFINECIPHER", 5, 8) == "IHHWVCSWFRCP"
    for text in ["", "AaZz", "e\u0301 世界 🧊\x00\r\n"]:
        assert cube.atbash(cube.atbash(text)) == text
        assert (
            cube.vigenere_decode(cube.vigenere_encode(text, "LeMoN"), "LeMoN") == text
        )
        assert cube.beaufort(cube.beaufort(text, "LeMoN"), "LeMoN") == text
        assert cube.affine_decode(cube.affine_encode(text, 5, 8), 5, 8) == text


@pytest.mark.parametrize("key", ["", "A B", "é", "A1", "A\x00"])
def test_classical_invalid_keys_have_structured_errors(key):
    for operation in [cube.vigenere_encode, cube.vigenere_decode, cube.beaufort]:
        with pytest.raises(ValueError) as error:
            operation("", key)
        assert error.value.code == "invalid_parameter"
        assert error.value.parameter == "key"
    with pytest.raises(ValueError):
        cube.affine_encode("", 13, 0)


def test_classical_limits_types_and_discovery():
    with pytest.raises(TypeError):
        cube.atbash(b"binary")
    with pytest.raises(ValueError) as error:
        cube.atbash("a" * (1024 * 1024 + 1))
    assert error.value.code == "resource_limit"
    assert cube.describe_operation("cipher.affine.encode")["reversible"] is True
