import cube
import pytest


def test_keyed_and_josephus_records_and_options():
    assert cube.keyed_skip("ABCDE", [2], 5)["extracted"] == "BDACE"
    assert cube.josephus_extract("ABCDE", [2])["extracted"] == "BDAEC"
    assert (
        cube.keyed_skip("ABCDE", [1], 3, options={"index_base": 1, "start": 2})[
            "extracted"
        ]
        == "BCD"
    )
    report = cube.keyed_skip("é🧊a\u0301", [2], 4)
    assert report["records"][1] == {
        "scalar_index": 3,
        "byte_offset": 7,
        "scalar": "\u0301",
    }
    assert cube.keyed_skip("", [1], 0)["records"] == []
    for steps in [[], [0], [-1], [True], [1.5], [2**64]]:
        with pytest.raises(ValueError):
            cube.keyed_skip("A", steps, 1)
    with pytest.raises(ValueError):
        cube.josephus_extract("ABC", [2], options={"wrap": False})
    with pytest.raises(ValueError):
        cube.keyed_skip("ABC", [2], 3, options={"wrap": False})
    with pytest.raises(ValueError):
        cube.keyed_skip("ABC", [1], 1, options={"unknown": 1})
    for key in ["max_work", "max_memory", "max_output"]:
        with pytest.raises(ValueError):
            cube.keyed_skip("ABC", [1], 1, options={key: 0})
