"""Native typed analysis uses the same reviewed known answers as Core."""

import json
from pathlib import Path

import cube
import pytest


def test_native_analysis_matches_reviewed_contracts():
    source = (
        Path(__file__).parents[3]
        / "crates/cube-core/src/operations/contracts/conformance.json"
    )
    vectors = json.loads(source.read_text())
    mapping = {
        "analysis.language.score": "language_score",
        "analysis.period.estimate": "period_estimate",
    }
    for v in vectors:
        if v["operation"] in mapping:
            assert (
                getattr(cube, mapping[v["operation"]])(v["inputs"]["request"])
                == v["outputs"]["report"]
            )


def test_model_validation_reaches_native_typed_errors():
    with pytest.raises(ValueError):
        cube.language_score({"candidates": ["A"], "model": {"kind": "unknown"}})
