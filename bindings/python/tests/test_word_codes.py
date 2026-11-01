"""Native SDK execution of source-known phrase and explicit page-word profiles."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).parents[3]


def test_every_word_code_contract_executes_through_native_exports():
    rows = json.loads(
        (
            ROOT / "crates/cube-core/src/operations/contracts/conformance.json"
        ).read_text()
    )
    cases = [
        r
        for r in rows
        if r["operation"].startswith(("cipher.key_phrase.", "cipher.arnold."))
    ]
    assert len(cases) == 4
    for row in cases:
        name = row["operation"].removeprefix("cipher.").replace(".", "_")
        assert (
            getattr(cube, name)(**row["inputs"], parameters=row["parameters"])
            == row["outputs"]
        )


def test_phrase_repeated_positions_are_not_guessed():
    p = {"phrase": "GIVEMELIBERTYORGIVEMEDEATH"}
    report = cube.key_phrase_decode("I", p)
    assert report["text"] is None and report["ambiguous"]
    assert report["records"] == [
        {
            "input": "I",
            "byte_offset": 0,
            "alternatives": ["B", "H", "Q"],
            "mapped": True,
        }
    ]
    assert cube.key_phrase_encode("bhq *", p)["text"] == "III *"
    assert cube.key_phrase_decode("", p)["text"] == ""
    with pytest.raises(ValueError) as failure:
        cube.key_phrase_decode("Z", p)
    assert failure.value.code == "invalid_input"
    assert failure.value.operation == "cipher.key_phrase.decode"


def test_arnold_pagination_words_and_source_provenance_are_exact():
    p = {
        "profile": "arnold_paged_words_v1",
        "source_id": "synthetic:edition-one",
        "index_base": 0,
        "choice": "round_robin",
    }
    pages = ["A B\nA 猫.", "A"]
    result = cube.arnold_encode(pages, ["A"] * 4, p)
    assert result["coordinates"] == [
        {"page": 0, "line": 0, "word": 0},
        {"page": 0, "line": 1, "word": 0},
        {"page": 1, "line": 0, "word": 0},
        {"page": 0, "line": 0, "word": 0},
    ]
    assert cube.arnold_decode(pages, result["coordinates"], p) == result
    changed = cube.arnold_encode([pages[0] + "\n", pages[1]], [], p)
    assert changed["source_sha256"] != result["source_sha256"]
    for word in ["a", "猫"]:
        with pytest.raises(ValueError) as failure:
            cube.arnold_encode(pages, [word], p)
        assert failure.value.operation == "cipher.arnold.encode"
        assert failure.value.code == "invalid_input"
    with pytest.raises(ValueError) as failure:
        cube.arnold_decode(pages, [{"page": 0, "line": 9, "word": 0}], p)
    assert failure.value.operation == "cipher.arnold.decode"


def test_structured_input_types_are_not_mislabeled_as_parameters():
    p = {
        "profile": "arnold_paged_words_v1",
        "source_id": "synthetic",
        "index_base": 1,
        "choice": "first",
    }
    with pytest.raises(ValueError) as failure:
        cube.arnold_encode([1], [], p)
    assert failure.value.code == "invalid_input"
    assert failure.value.parameter == "pages"
