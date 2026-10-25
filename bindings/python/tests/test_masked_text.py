"""Native masked transformations share the Rust position and cipher implementations."""

import cube
import pytest


def test_fibonacci_known_answer_and_auditable_overrides():
    options = dict(
        selector="fibonacci",
        exclude=[13, 21],
        index_base=1,
        selected="atbash",
        otherwise="caesar",
        otherwise_shift=3,
    )
    text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    report = cube.transform_masked(text, **options)
    assert report["original_input"] == text
    assert report["selected_positions"] == [1, 2, 3, 5, 8]
    assert report["deterministic_text"] == "ZYXGVIJSLMNOPQRSTUVWXYZABC"
    assert report["final_text"][12] == "P"
    assert report["final_text"][20] == "X"
    assert report["overrides"] == []
    assert cube.transform_masked(text, **options) == report
    edited = cube.transform_masked(text, overrides=[(1, "雪"), (1, "🧊")], **options)
    assert edited["deterministic_text"] == report["deterministic_text"]
    assert edited["final_text"] == "🧊" + report["final_text"][1:]
    assert edited["overrides"] == [
        dict(position=1, previous="Z", replacement="雪"),
        dict(position=1, previous="雪", replacement="🧊"),
    ]


def test_selectors_and_unicode_count_every_scalar_without_normalization():
    assert (
        cube.transform_masked("", selector="fibonacci", selected="atbash")["final_text"]
        == ""
    )
    assert cube.transform_masked("AB", selector="fibonacci", selected="atbash")[
        "selected_positions"
    ] == [0, 1]
    assert cube.transform_masked(
        "ABC", selector="fibonacci", selected="atbash", index_base=1
    )["selected_positions"] == [1, 2, 3]
    report = cube.transform_masked(
        "A !éB e\u0301Z",
        selector="every_nth",
        nth=2,
        index_base=1,
        selected="atbash",
        otherwise="caesar",
        otherwise_shift=3,
    )
    assert report["selected_positions"] == [2, 4, 6, 8]
    assert report["final_text"] == "D !éE h\u0301C"
    report = cube.transform_masked(
        "ABCDEF", selector="indices", indices=[5, 1, 1, 2], selected="atbash"
    )
    assert report["selected_positions"] == [1, 2, 5]
    assert report["final_text"] == "AYXDEU"
    assert cube.extract_characters("ABCDEF", indices=[5, 1, 1, 2]) == "FBBC"
    report = cube.transform_masked(
        "ABCDEFGHI", selector="fibonacci", exclude=[0, 3, 21], selected="identity"
    )
    assert report["selected_positions"] == [1, 2, 5, 8]


@pytest.mark.parametrize("shift", [-(2**31), -53, 0, 2**31 - 1])
def test_cipher_parity_and_large_shifts(shift):
    text = "AaZz é世界🧊e\u0301\0\r\n!?"
    for selector in [
        dict(selector="every_nth", nth=1),
        dict(selector="indices", indices=[]),
    ]:
        report = cube.transform_masked(
            text,
            selected="caesar",
            selected_shift=shift,
            otherwise="caesar",
            otherwise_shift=shift,
            **selector,
        )
        assert report["final_text"] == cube.caesar_encode(text, shift)
    assert cube.transform_masked(text, selector="every_nth", nth=1, selected="rot13")[
        "final_text"
    ] == cube.rot13(text)


@pytest.mark.parametrize(
    "options",
    [
        dict(selector="guess"),
        dict(selector="every_nth", nth=0),
        dict(selector="every_nth"),
        dict(selector="indices"),
        dict(selector="indices", indices=[0], nth=2),
        dict(selector="fibonacci", exclude=[4]),
        dict(index_base=2),
        dict(selector="indices", indices=[3]),
        dict(index_base=1, overrides=[(0, "x")]),
        dict(overrides=[(3, "x")]),
        dict(overrides=[(0, "")]),
        dict(overrides=[(0, "ab")]),
        dict(overrides=[(0, "e\u0301")]),
        dict(selected="caesar"),
        dict(selected_shift=3),
        dict(otherwise="caesar"),
        dict(otherwise_shift=1),
    ],
)
def test_invalid_configuration_is_rejected(options):
    arguments = dict(selector="fibonacci", selected="atbash") | options
    with pytest.raises(ValueError) as error:
        cube.transform_masked("ABC", **arguments)
    assert error.value.code == "invalid_parameter"
    assert error.value.operation == "text.transform.masked"


def test_types_limits_and_operation_discovery():
    for options in [dict(indices=[-1]), dict(indices=[2**100])]:
        with pytest.raises(OverflowError):
            cube.transform_masked(
                "ABC", selector="indices", selected="atbash", **options
            )
    with pytest.raises(TypeError):
        cube.transform_masked(b"ABC", selector="fibonacci", selected="atbash")
    for text, options in [
        ("A" * (1024 * 1024 + 1), dict(selector="fibonacci")),
        ("A", dict(selector="indices", indices=[0] * 65537)),
        ("A" * 65537, dict(selector="every_nth", nth=1)),
        ("A", dict(selector="fibonacci", overrides=[(0, "x")] * 65537)),
    ]:
        with pytest.raises(ValueError) as error:
            cube.transform_masked(text, selected="atbash", **options)
        assert error.value.code == "resource_limit"
    descriptor = cube.describe_operation("text.transform.masked")
    assert descriptor["pipeline"] is None
    with pytest.raises(ValueError):
        cube.validate_pipeline(
            [dict(operation="text.transform.masked", version=1)], "text"
        )


def test_optional_defaults_and_unknown_keyword_rejection():
    options = dict(selector="fibonacci", selected="identity")
    assert cube.transform_masked("ABC", **options) == cube.transform_masked(
        "ABC",
        indices=None,
        nth=None,
        exclude=None,
        overrides=None,
        selected_shift=None,
        otherwise_shift=None,
        **options,
    )
    with pytest.raises(TypeError):
        cube.transform_masked("ABC", unknown=True, **options)
    for options in [dict(index_base=-1), dict(nth=-1), dict(selected_shift=2**31)]:
        with pytest.raises(OverflowError):
            cube.transform_masked(
                "ABC", selector="fibonacci", selected="atbash", **options
            )
