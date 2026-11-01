"""Execute the independent ACA worksheet through the real native extension."""

import cube
import pytest

PARAMETERS = {
    "alphabet": "abcdefghijklmnopqrstuvwxyz",
    "key": "I like ciphers.",
    "preparation": "ascii_lowercase_letters",
    "rotation": 18,
    "origin": "0",
    "width": 2,
    "choice": "round_robin",
}


def test_published_table_and_deterministic_choices():
    report = cube.numbered_key_encode("A ee iii", PARAMETERS)
    assert report["tokens"] == ["23", "15", "20", "11", "13", "17"]
    assert cube.numbered_key_decode(report["tokens"], PARAMETERS)["text"] == "aeeiii"
    assert cube.numbered_key_encode("A ee iii", PARAMETERS) == report
    assert len(report["definition_sha256"]) == 64


def test_first_choices_keep_full_decode_table():
    p = dict(PARAMETERS, choice="first")
    assert cube.numbered_key_encode("eee", p)["tokens"] == ["15"] * 3
    assert cube.numbered_key_decode(["20"], p)["text"] == "e"


@pytest.mark.parametrize("tokens", [["1"], ["999"], [1]])
def test_exact_token_errors(tokens):
    with pytest.raises((ValueError, TypeError)):
        cube.numbered_key_decode(tokens, PARAMETERS)


def test_non_ascii_preparation_and_checked_origin():
    with pytest.raises(ValueError):
        cube.numbered_key_encode("é", PARAMETERS)
    with pytest.raises(ValueError):
        cube.numbered_key_encode("a", dict(PARAMETERS, origin=str(2**128 - 1)))


def test_native_errors_identify_the_requested_operation():
    with pytest.raises(ValueError) as failure:
        cube.numbered_key_decode(["99"], PARAMETERS)
    assert failure.value.operation == "cipher.numbered_key.decode"
    with pytest.raises(ValueError) as failure:
        cube.numbered_key_encode("1", dict(PARAMETERS, preparation="exact", key="abc"))
    assert failure.value.operation == "cipher.numbered_key.encode"


def test_wrong_typed_tokens_use_input_diagnostics():
    with pytest.raises(ValueError) as failure:
        cube.numbered_key_decode([0], PARAMETERS)
    assert failure.value.code == "invalid_input"
    assert failure.value.operation == "cipher.numbered_key.decode"
    assert failure.value.parameter == "tokens"
