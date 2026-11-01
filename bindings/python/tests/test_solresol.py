"""Installed native execution of the exact finite source-qualified lexicon."""

import json
from pathlib import Path

import cube
import pytest

P = {"profile": "gajewski_1902_monosyllables_v1"}


def test_native_solresol_matches_both_complete_canonical_fixtures():
    path = (
        Path(__file__).parents[3]
        / "crates/cube-core/src/operations/contracts/conformance.json"
    )
    cases = [
        r
        for r in json.loads(path.read_text())
        if r["operation"].startswith("symbol.solresol.")
    ]
    assert len(cases) == 2
    for r in cases:
        fn = (
            cube.solresol_encode
            if r["operation"].endswith("encode")
            else cube.solresol_decode
        )
        assert fn(**r["inputs"], parameters=r["parameters"]) == r["outputs"]


def test_negative_affirmative_conditional_and_aliases_keep_distinctions():
    encoded = cube.solresol_encode(["non", "oui", "à", "si (condition)"], P)
    assert encoded["tokens"] == ["do", "si", "fa", "sol"]
    decoded = cube.solresol_decode(["d", "si", "r", "ré", "do", "do"], P)
    assert decoded["tokens"] == ["d", "si", "r", "ré", "do", "do"]
    assert decoded["readings"][0]["alternatives"] == ["non", "pas", "point", "ne", "ni"]
    assert decoded["readings"][1]["alternatives"] == [
        "oui",
        "soit",
        "volontiers",
        "d'accord",
    ]
    assert decoded["readings"][5]["token_index"] == 5
    assert (
        decoded["provenance"]["definition_sha256"]
        == encoded["provenance"]["definition_sha256"]
    )


@pytest.mark.parametrize("form", ["re", "re\u0301", "rr", "do-ré", "C4"])
def test_unknown_or_altered_words_do_not_normalize_or_guess(form):
    with pytest.raises(ValueError) as failure:
        cube.solresol_decode([form], P)
    assert failure.value.code == "invalid_input"
    assert failure.value.operation == "symbol.solresol.decode"


def test_required_profile_and_typed_input_diagnostics():
    with pytest.raises(ValueError) as failure:
        cube.solresol_encode([], {})
    assert failure.value.code == "invalid_parameter"
    with pytest.raises(ValueError) as failure:
        cube.solresol_decode([0], P)
    assert failure.value.code == "invalid_input"
    assert failure.value.parameter == "tokens"
    assert cube.solresol_decode([], P)["readings"] == []
