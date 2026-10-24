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


def test_unicode_normalization_and_scalar_reversal():
    assert cube.UNICODE_VERSION == (18, 0, 0)
    assert cube.normalize_text("e\u0301", "NFC") == "é"
    assert cube.normalize_text("é", "NFD") == "e\u0301"
    assert cube.normalize_text("ﬀ Ａ ①", "NFKC") == "ff A 1"
    assert cube.normalize_text("Å", "NFKD") == "A\u030a"
    assert cube.reverse_text("e\u0301\r\n") == "\n\r\u0301e"
    for form in ["NFC", "NFD", "NFKC", "NFKD"]:
        result = cube.normalize_text("é ﬀ 각", form)
        assert cube.normalize_text(result, form) == result
    for text in ["", "é 世界🧊e\u0301\x00\r\n"]:
        assert cube.reverse_text(cube.reverse_text(text)) == text
    with pytest.raises(ValueError):
        cube.normalize_text("", "nfc")


def test_positional_selection_keeps_duplicates_and_validates_modes():
    text = "Aé🧊e\u0301Z"
    assert cube.extract_characters(text, indices=[5, 1, 1, 2]) == "Zéé🧊"
    assert cube.extract_characters(text, indices=[]) == ""
    assert cube.extract_characters(text, stride=2) == "A🧊\u0301"
    assert cube.extract_characters(text, start=1, stride=2) == "éeZ"
    assert cube.extract_characters(text, start=6, stride=1) == ""
    for options in [
        dict(),
        dict(indices=[6]),
        dict(stride=0),
        dict(stride=1, stop=7),
        dict(indices=[], start=0),
    ]:
        with pytest.raises(ValueError):
            cube.extract_characters(text, **options)
    with pytest.raises(OverflowError):
        cube.extract_characters(text, indices=[-1])
    with pytest.raises(ValueError) as error:
        cube.extract_characters("🧊", indices=[0] * 65537)
    assert error.value.code == "resource_limit"
