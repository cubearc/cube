"""Native execution of dated symbolic alphabets, including every assignment."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3] / "crates/cube-core/src/symbols/alphabet/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("row", VECTORS)
def test_all_published_assignments_and_exact_positions(row):
    p = {"profile": row["profile"]}
    encoded = cube.alphabet_encode(row["units"], p)
    assert encoded["tokens"] == row["tokens"]
    decoded = cube.alphabet_decode(row["tokens"], p)
    assert decoded["tokens"] == row["tokens"]
    assert decoded["readings"] == [
        {"token_index": i, "alternatives": [unit]}
        for i, unit in enumerate(row["units"])
    ]
    assert encoded["provenance"] == decoded["provenance"]
    assert cube.alphabet_decode([], p)["readings"] == []


@pytest.mark.parametrize("unit", ["a", "Ch", "CHAT", "0", "é", ""])
def test_no_casefolding_segmentation_or_unsourced_digits(unit):
    with pytest.raises(ValueError) as e:
        cube.alphabet_encode([unit], {"profile": "aurebesh_ucsur_2016_v1"})
    assert e.value.code == "invalid_input"
    assert e.value.operation == "symbol.alphabet.encode"


def test_parameter_input_and_limits_remain_typed():
    p = {"profile": "aurebesh_ucsur_2016_v1"}
    with pytest.raises(ValueError) as e:
        cube.alphabet_encode([], {})
    assert e.value.code == "invalid_parameter"
    with pytest.raises(ValueError) as e:
        cube.alphabet_decode([0], p)
    assert e.value.code == "invalid_input"
    assert e.value.parameter == "tokens"
    with pytest.raises(ValueError) as e:
        cube.alphabet_encode(["A"] * 65537, p)
    assert e.value.code == "resource_limit"
    with pytest.raises(ValueError) as e:
        cube.alphabet_decode(["\ue8d0"], p)
    assert e.value.code == "invalid_input"


def test_published_aliases_retain_exact_orientation_and_bird_code():
    aliases = json.loads(
        (
            Path(__file__).parents[3]
            / "crates/cube-core/src/symbols/alphabet/aliases.json"
        ).read_text()
    )
    assert len(aliases) == 147
    for row in aliases:
        r = cube.alphabet_decode([row["token"]], {"profile": row["profile"]})
        assert r["tokens"] == [row["token"]]
        assert r["readings"] == [
            {"token_index": 0, "alternatives": row["alternatives"]}
        ]
