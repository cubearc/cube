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
    alternatives = row.get("alternatives", [[unit] for unit in row["units"]])
    assert decoded["readings"] == [
        {"token_index": i, "alternatives": readings}
        for i, readings in enumerate(alternatives)
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


def test_published_aliases_retain_exact_form_and_all_readings():
    aliases = json.loads(
        (
            Path(__file__).parents[3]
            / "crates/cube-core/src/symbols/alphabet/aliases.json"
        ).read_text()
    )
    assert len(aliases) == 187
    for row in aliases:
        r = cube.alphabet_decode([row["token"]], {"profile": row["profile"]})
        assert r["tokens"] == [row["token"]]
        assert r["readings"] == [
            {"token_index": 0, "alternatives": row["alternatives"]}
        ]


@pytest.mark.parametrize(
    "count", [65536, (1048576 - 1) // (len("upright:left:flat:raised") + 3)]
)
def test_output_budget_diagnostics_do_not_blame_the_input(count):
    with pytest.raises(ValueError) as e:
        cube.alphabet_encode(
            ["H"] * count, {"profile": "birds_nrtech_four_features_v1"}
        )
    assert e.value.code == "resource_limit"
    assert e.value.operation == "symbol.alphabet.encode"
    assert e.value.parameter == "output"


def test_hylian_chart_duplicates_are_ambiguous_in_the_native_sdk():
    p = {"profile": "hylian_skyward_dcode_v1"}
    assert cube.alphabet_encode(["D", "W"], p)["tokens"] == [
        "hylian:r1c4",
        "hylian:r1c4",
    ]
    assert cube.alphabet_decode(["hylian:r3c5"], p)["readings"] == [
        {"token_index": 0, "alternatives": ["D", "W"]}
    ]
    with pytest.raises(ValueError) as e:
        cube.alphabet_decode(["sheikah:r1c1"], p)
    assert e.value.code == "invalid_input"
