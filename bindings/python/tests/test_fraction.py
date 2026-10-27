"""Native adapters share the independent Core/CLI known-answer corpus."""

import json
from pathlib import Path

import cube
import pytest

CASES = sum(
    (
        json.loads(
            (
                Path(__file__).resolve().parents[3]
                / f"crates/cube-core/src/ciphers/{domain}/vectors.json"
            ).read_text()
        )
        for domain in ("transposition", "fractionation")
    ),
    [],
)


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["function"])
def test_native_known_answers_and_contracts(case):
    function = getattr(cube, case["function"])
    actual = function(
        case["inputs"].get("text", case["inputs"].get("tokens")), case["parameters"]
    )
    expected = case["outputs"]
    assert actual == (next(iter(expected.values())) if len(expected) == 1 else expected)
    descriptor = cube.describe_operation(case["operation"], include_contracts=True)
    assert descriptor["contract"]["operation_version"] == 1


def test_native_scalar_permutations_padding_and_contextual_failures():
    text = "a😀β !\0e\u0301"
    parameters = {"key": {"kind": "keyword", "text": "BALLOON"}}
    encoded = cube.columnar_encode(text, parameters)
    assert cube.columnar_decode(encoded, parameters) == text
    result = cube.complete_columnar_encode(
        "AXXX", {"key": {"kind": "keyword", "text": "CAB"}, "padding": "X"}
    )
    assert result["original_length"] == 4 and result["padding_count"] == 2
    decoded = cube.complete_columnar_decode(
        result["text"],
        {
            "key": {"kind": "keyword", "text": "CAB"},
            "padding": "X",
            "original_length": 4,
        },
    )
    assert decoded["text"] == "AXXX"
    ambiguous = cube.morbit_decode("1", {"digits": "123456789"})
    assert ambiguous["text"] is None
    assert ambiguous["padding_ambiguous"] and ambiguous["morse"] == ".."
    with pytest.raises(ValueError) as caught:
        cube.skip_encode("ABCD", {"start": 0, "step": 2})
    assert caught.value.operation == "cipher.skip.encode"
    with pytest.raises(ValueError):
        cube.sequence_transposition_encode("AB", {"order": [0, 0]})
    with pytest.raises(ValueError):
        cube.scytale_encode("A" * 65537, {"columns": 1})
    with pytest.raises(ValueError):
        cube.nihilist_substitution_decode(
            [110] * 65537, {"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ", "key": "Z"}
        )


def test_exact_decimal_transport_agrees_with_native_integer_endpoints():
    assert cube.skip_encode("ABCDE", {"start": "0", "step": str(-(2**63))}) == "ACEBD"
    assert cube.skip_encode("ABCDE", {"start": 0, "step": -(2**63)}) == "ACEBD"
    assert cube.sequence_transposition_encode("AB", {"order": ["1", "0"]}) == "BA"
    for step in ["-0", "00", "+1", "9223372036854775808"]:
        with pytest.raises(ValueError):
            cube.skip_encode("ABCDE", {"start": 0, "step": step})
    assert (
        cube.nihilist_substitution_decode(
            ["110"], {"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ", "key": "Z"}
        )
        == "Z"
    )
