"""Exercise the bounded text profiles through the installed native module."""

import json
from pathlib import Path

import cube
import pytest

OPERATIONS = {
    "text.beider_morse",
    "text.daitch_mokotoff",
    "text.graphemes",
    "text.match_rating",
    "text.phonetic",
    "text.regex",
    "text.scalar_weave",
    "text.select_words",
    "text.sort",
    "text.tokenize",
    "text.unicode_transform",
}
FIXTURES = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)
CASES = [
    row
    for row in json.loads(FIXTURES.read_text(encoding="utf-8"))
    if row["operation"] in OPERATIONS
]


def test_exact_contract_fixture_coverage():
    assert {row["operation"] for row in CASES} == OPERATIONS


@pytest.mark.parametrize("case", CASES, ids=[row["operation"] for row in CASES])
def test_native_contract(case):
    operation = case["operation"].replace(".", "_")
    assert operation in cube.__all__
    assert (
        getattr(cube, operation)(case["inputs"]["request"]) == case["outputs"]["report"]
    )


def test_native_word_indices_and_regex_limits():
    minimum = -(2**63)
    result = cube.text_select_words(
        {"text": "word", "indices": [minimum], "out_of_range": "skip"}
    )
    assert result["omitted"] == [{"position": 0, "index": minimum}]
    for invalid in (
        {"text": "a", "pattern": "a", "max_results": 0, "action": "matches"},
        {"text": "a", "pattern": "(?=a)", "action": "matches"},
    ):
        with pytest.raises(ValueError):
            cube.text_regex(invalid)
