"""Exercise all numeric analysis operations through the rebuilt native wheel."""

import json
from pathlib import Path

import cube
import pytest

FIXTURES = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)
CASES = [
    row
    for row in json.loads(FIXTURES.read_text(encoding="utf-8"))
    if row["operation"].startswith("number.")
    and "request" in row["inputs"]
    and "report" in row["outputs"]
]

EXACT_FIELDS = {
    "number.gray": {"value"},
    "number.factoradic": {"value"},
    "number.balanced_ternary": {"value"},
    "number.lehmer": {"rank"},
    "number.lexicographic_rank": {"rank", "total"},
    "number.partitions": {"count"},
    "number.permutation_list": {"count"},
    "number.entropy": {"samples"},
    "number.matrix": {"numerator", "denominator"},
    "number.interpolation": {"numerator", "denominator"},
    "number.letter_values": {"value"},
}


def native_values(value, exact, key=None):
    if isinstance(value, dict):
        return {k: native_values(v, exact, k) for k, v in value.items()}
    if isinstance(value, list):
        return [native_values(v, exact, key) for v in value]
    if isinstance(value, str) and key in exact:
        return int(value)
    return value


def test_all_numeric_fixtures_are_present():
    assert len(CASES) == 23


@pytest.mark.parametrize("case", CASES, ids=[row["operation"] for row in CASES])
def test_numeric_contract(case):
    operation = case["operation"]
    native = operation.replace(".", "_")
    assert native in cube.__all__
    report = getattr(cube, native)(case["inputs"]["request"])
    assert report == native_values(
        case["outputs"]["report"], EXACT_FIELDS.get(operation, set())
    )


def test_native_wide_integers_and_text():
    maximum = 2**128 - 1
    encoded = cube.number_gray({"direction": "encode", "width": 128, "value": maximum})
    assert encoded["value"] == 2**127
    assert isinstance(encoded["value"], int)
    assert (
        cube.number_balanced_ternary({"direction": "encode", "value": -maximum})[
            "value"
        ]
        == -maximum
    )
    assert (
        cube.number_recurrence({"sequence": "fibonacci", "count": 94})["values"][-1]
        == 12200160415121876738
    )
    assert cube.number_matrix(
        {
            "action": "determinant",
            "matrix": {
                "rows": 1,
                "cols": 1,
                "cells": [{"numerator": -(2**127), "denominator": maximum}],
            },
        }
    )["value"] == {"numerator": -(2**127), "denominator": maximum}
    assert cube.number_markov({"states": ["123"], "sequence": ["123"]})["states"] == [
        "123"
    ]


def test_native_limits_and_invalid_inputs():
    with pytest.raises(ValueError):
        cube.number_gray({"direction": "encode", "width": 128, "value": 2**128})
    with pytest.raises(ValueError):
        cube.number_roman({"direction": "decode", "text": "IIII"})
    with pytest.raises(ValueError):
        cube.number_correlation({"method": "lag", "x": [float("nan")], "lags": [0]})
    maximum = 2**20
    assert (
        cube.number_runs({"input": "binary", "values": [0] * maximum})["n0"] == maximum
    )
    with pytest.raises(ValueError):
        cube.number_runs({"input": "binary", "values": [0] * (maximum + 1)})
