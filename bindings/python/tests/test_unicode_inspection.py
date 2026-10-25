import cube
import pytest


def test_unicode_counts_properties_and_limited_listings():
    text = "Aaǅé中\0\u200b\u200c\u200d\u00a0\u202e\ufe0f"
    report = cube.inspect_unicode(text)
    assert report["counts"]["scalars"] == 12
    assert report["records"][2]["case"] == "titlecase"
    assert report["records"][4]["byte_offset"] == 6
    assert report["records"][5]["control"]
    assert report["records"][8]["zero_width_candidate"]
    assert report["records"][9]["whitespace"]
    assert report["records"][10]["bidi_control"]
    assert report["category_unicode_version"] == [16, 0, 0]
    limited = cube.inspect_unicode(text, options={"max_results": 0})
    assert limited["counts"]["scalars"] == 12
    assert limited["records"] == [] and not limited["complete"]
    assert cube.inspect_unicode("")["complete"]


def test_original_and_normalized_offsets_are_separate():
    report = cube.compare_unicode("é", "e\u0301", normalization="NFC")
    assert not report["exact_equal"] and report["normalized_equal"]
    assert report["original"]["total_differences"] == 2
    assert report["original"]["records"][1]["left"] is None
    assert report["original"]["records"][1]["right"]["byte_offset"] == 1
    assert report["normalized"]["coordinate_space"] == "normalized_scalar_index"
    assert report["normalized"]["records"] == []
    report = cube.compare_unicode("ﬀX", "ffY", normalization="NFKC")
    assert report["normalized"]["records"][0]["left"]["byte_offset"] == 2
    assert report["original"]["records"][1]["left"]["byte_offset"] == 3
    report = cube.compare_unicode(
        "AAA", "aaa", normalization="NFC", options={"max_results": 1}
    )
    assert not report["complete"]
    assert len(report["original"]["records"]) == 1
    assert report["normalized"]["records"] == []
    assert report["normalized"]["total_differences"] == 3
    assert cube.compare_unicode("", "")["exact_equal"]
    for options in [
        {"max_results": 65537},
        {"max_work": 0},
        {"max_output": 0},
        {"unknown": True},
    ]:
        with pytest.raises(ValueError):
            cube.inspect_unicode("A", options=options)
    with pytest.raises(ValueError):
        cube.compare_unicode("", "", normalization="bad")
