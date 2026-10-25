import cube
import pytest


def test_signed_word_positions_and_omission_evidence():
    text = "Cube Uses Binary Evidence"
    assert (
        cube.extract_word_characters(text, positions=[1, 3])["extracted"] == "CbUeBnEi"
    )
    assert cube.extract_word_characters(text, positions=[-1])[
        "extracted"
    ] == cube.select_text(text, "word_last")
    assert (
        cube.extract_word_characters("ab cd", per_word=[[2, 1, 2], [-1]])["extracted"]
        == "babd"
    )
    report = cube.extract_word_characters("A bé", positions=[2], short_word="skip")
    assert report["extracted"] == "é"
    assert report["omitted_count"] == 1
    assert report["words"][0]["selections"][0]["scalar"] is None
    assert (
        cube.extract_word_characters("éa\u00a0🧊b", positions=[0], index_base=0)[
            "words"
        ][1]["selections"][0]["byte_offset"]
        == 5
    )
    assert cube.extract_word_characters("", per_word=[])["words"] == []
    for positions in [[0], [True], [1.0], [2**63], [-(2**63)]]:
        with pytest.raises(ValueError):
            cube.extract_word_characters("A", positions=positions)
    with pytest.raises(ValueError):
        cube.extract_word_characters("A", positions=[1], per_word=[[1]])
    with pytest.raises(ValueError):
        cube.extract_word_characters("A", per_word=[])
