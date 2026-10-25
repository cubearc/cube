import cube
import pytest


def test_numeric_views_keep_exact_boundaries_and_presentations():
    assert [w["scalar_length"] for w in cube.word_lengths("A é e\u0301")] == [1, 1, 2]
    for presentation in ["decimal", "hex", "unicode"]:
        assert [
            c["value"] for c in cube.text_codepoints("A é", format=presentation)
        ] == [65, 32, 233]
    assert cube.text_codepoints("é", format="unicode")[0]["presentation"] == "U+00E9"
    assert [w["sum"] for w in cube.word_values("A é", sums=True)] == [65, 233]
    report = cube.word_values(
        "A?", table=[("A", 2**64 - 1)], unmapped="skip", sums=True
    )
    assert report[0]["sum"] == 2**64 - 1
    assert isinstance(report[0]["sum"], int)
    assert report[0]["values"][1]["value"] is None
    assert report[0]["omitted_count"] == 1
    with pytest.raises(ValueError):
        cube.word_values("AA", table=[("A", 2**64 - 1)], sums=True)
    with pytest.raises(ValueError):
        cube.word_values("A", sums=True, max_sum=64)
    for table in [[("A", True)], [("A", -1)], [("A", 1.0)], [("A", 1), ("A", 2)]]:
        with pytest.raises(ValueError):
            cube.word_values("A", table=table)
    with pytest.raises(ValueError):
        cube.text_codepoints("A", format="unknown")
