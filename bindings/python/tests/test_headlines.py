"""All five source-known Headlines results through the installed native SDK."""

import json
from pathlib import Path

import cube
import pytest


def test_published_headlines_and_independent_alphabet():
    rows = json.loads(
        (
            Path(__file__).parents[3]
            / "crates/cube-core/src/operations/contracts/conformance.json"
        ).read_text()
    )
    cases = [r for r in rows if r["operation"].startswith("cipher.headlines.")]
    assert len(cases) == 2
    for row in cases:
        fn = (
            cube.headlines_decode
            if row["operation"].endswith("decode")
            else cube.headlines_encode
        )
        result = fn(row["inputs"]["headlines"], row["parameters"])
        assert result == row["outputs"]
        assert result["alphabet"] == "CFUAPTOSNZILYEJWHGVBQMKXDR"


def test_headlines_reject_unsupported_variants_and_ambiguous_punctuation():
    p = {
        "profile": "aca_base",
        "hat": "APOTHECARY",
        "key": "CHEMIST",
        "setting": "DRUGS",
        "punctuation": "aca_printed",
    }
    assert cube.headlines_encode([""] * 5, p)["headlines"] == [""] * 5
    for headlines in [[], ["é", "", "", "", ""], ["=", "", "", "", ""]]:
        with pytest.raises(ValueError):
            cube.headlines_encode(headlines, p)
    with pytest.raises(ValueError):
        cube.headlines_encode([""] * 5, dict(p, profile="up"))
