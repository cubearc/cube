"""Native typed analysis uses the same reviewed known answers as Core."""

import json
from pathlib import Path

import cube
import pytest


def test_native_frequency_distance_and_invalid_totals():
    request = {
        "bins": [
            {"label": "A", "observed": "20", "expected": 25.0},
            {"label": "B", "observed": 30, "expected": 25.0},
            {"label": "C", "observed": 50, "expected": 50.0},
        ],
        "expected_policy": "counts",
    }
    report = cube.frequency_compare(request)
    assert report["distance"] == {"kind": "finite", "value": 2.0}
    assert report["degrees_of_freedom"] == 2
    request["bins"][0]["expected"] = 20.0
    with pytest.raises(ValueError) as error:
        cube.frequency_compare(request)
    assert error.value.operation == "analysis.frequency.compare"


def test_native_analysis_matches_reviewed_contracts():
    source = (
        Path(__file__).parents[3]
        / "crates/cube-core/src/operations/contracts/conformance.json"
    )
    vectors = json.loads(source.read_text())
    mapping = {
        "analysis.language.score": "language_score",
        "analysis.period.estimate": "period_estimate",
        "analysis.caesar.rank": "caesar_rank",
        "analysis.affine.rank": "affine_rank",
        "analysis.vigenere.rank": "vigenere_rank",
        "analysis.transposition.rank": "transposition_rank",
        "analysis.homophonic.inspect": "homophonic_inspect",
        "analysis.crib.verify": "crib_verify",
        "analysis.frequency.compare": "frequency_compare",
        "analysis.substitution.solve": "substitution_solve",
        "analysis.family.suggest": "family_suggest",
    }
    assert set(mapping) == {
        v["operation"] for v in vectors if v["operation"].startswith("analysis.")
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


def test_existing_cancellation_token_stops_native_analysis():
    token = cube.CancellationToken()
    token.cancel()
    report = cube.caesar_rank(
        {
            "text": "B",
            "model": {
                "kind": "dictionary",
                "entries": [{"text": "A", "weight": 1}],
                "unseen_weight": 0,
            },
        },
        cancellation=token,
    )
    assert report["status"]["stop"] == "cancelled"
    assert report["attempted"] == 0


def test_probability_mass_cannot_exceed_one_even_inside_sum_tolerance():
    with pytest.raises(ValueError):
        cube.language_score(
            {
                "candidates": ["A"],
                "model": {
                    "kind": "probabilities",
                    "n": 1,
                    "entries": [{"text": "A", "weight": 1.0000000005}],
                    "unseen_probability": 1e-300,
                    "normalization": "exact",
                },
            }
        )


def test_substitution_exact_classes_aliases_and_cancellation():
    import math

    request = {
        "profile": "observed_bijection_v1",
        "alphabet": "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "ciphertext": "αBBα",
        "outside": "preserve",
        "aliases": [{"alias": "α", "symbol": "A"}],
        "dictionary": {
            "segmentation": "unicode_whitespace",
            "words": ["NOON", "DEED", "NOON"],
        },
    }
    result = cube.substitution_solve(request)
    assert [c["plaintext"] for c in result["candidates"]] == ["NOON", "DEED"]
    assert result["candidates"][0]["key_count"] == str(math.factorial(24))
    assert result["dictionary_classes"][0]["input_indices"] == [0, 2]
    token = cube.CancellationToken()
    token.cancel()
    result = cube.substitution_solve(request, cancellation=token)
    assert result["status"]["stop"] == "cancelled"
    assert not result["contradiction"]
    request["limits"] = {"max_work": 1048577}
    with pytest.raises(ValueError) as error:
        cube.substitution_solve(request)
    assert error.value.operation == "analysis.substitution.solve"


def test_family_uncertainty_and_native_cancellation():
    request = {"profile": "ascii_classical_evidence_v1", "text": "AB" * 128}
    result = cube.family_suggest(request)
    assert result["suggestions"] == []
    assert "fewer_than_8_distinct_letters" in result["eligibility_failures"]
    token = cube.CancellationToken()
    token.cancel()
    assert (
        cube.family_suggest(request, cancellation=token)["status"]["stop"]
        == "cancelled"
    )
