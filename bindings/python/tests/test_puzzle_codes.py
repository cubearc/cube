"""Native book, keyboard and symbolic boundaries use the real Core implementation."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).resolve().parents[3]
CASES = sum(
    (
        json.loads((ROOT / f"crates/cube-core/src/{path}/vectors.json").read_text())
        for path in (
            "ciphers/book/encoding",
            "text/keyboard/coordinates",
            "symbols/tokens",
        )
    ),
    [],
)


@pytest.mark.parametrize("case", CASES, ids=lambda c: c["function"])
def test_all_native_reports_match_independent_core_cli_vectors(case):
    inputs = case["inputs"]
    arguments = (
        [inputs["source"], inputs["text"]]
        if "source" in inputs
        else [inputs.get("text", inputs.get("coordinates"))]
    )
    assert (
        getattr(cube, case["function"])(*arguments, case["parameters"])
        == case["outputs"]
    )
    assert (
        cube.describe_operation(case["op"], include_contracts=True)["contract"][
            "operation_version"
        ]
        == 1
    )


def test_source_hash_choices_and_legacy_decoder_agree():
    parameters = {
        "scheme": "word_character",
        "index_base": "1",
        "matching": "round_robin",
    }
    result = cube.book_encode("aa aa", "aaaaa", parameters)
    assert result["coordinates"] == "1:1,1:2,2:1,2:2,1:1"
    assert cube.book_decode("aa aa", result["coordinates"]) == "aaaaa"
    with pytest.raises(ValueError):
        cube.book_encode("a", "?", parameters)
    with pytest.raises(ValueError):
        cube.book_encode("a", "a" * 65537, parameters)


def test_native_definitions_are_strict_and_no_alias_inverse_is_guessed():
    parameters = next(
        c["parameters"] for c in CASES if c["op"] == "symbol.tokens.decode"
    )
    with pytest.raises(ValueError):
        cube.symbol_tokens_encode("A", parameters)
    with pytest.raises(ValueError):
        cube.symbol_tokens_decode("?", parameters)
    with pytest.raises(ValueError):
        cube.symbol_tokens_decode("", {**parameters, "unknown": True})
    parameters = next(
        c["parameters"] for c in CASES if c["op"] == "text.keyboard.coordinates.encode"
    )
    text = "a😀β"
    result = cube.keyboard_coordinates_encode(text, parameters)
    assert (
        cube.keyboard_coordinates_decode(result["coordinates"], parameters)["text"]
        == text
    )
    assert result["positions"] == [[0, 0], [1, 1], [5, 2]]
