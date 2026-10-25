"""Native Rust puzzle operations and exact structured evidence."""

import cube
import pytest


def test_bacon_variants_and_notations():
    assert cube.bacon_encode("AB", "extended26") == "AAAAA AAAAB"
    assert cube.bacon_encode("Z", "classical24") == "BABBB"
    for variant, canonical in [("extended26", "IJ UV"), ("classical24", "II UU")]:
        for notation in ["ab", "binary", "binary_inverted"]:
            encoded = cube.bacon_encode(" ij  uv ", variant, notation=notation)
            assert cube.bacon_decode(encoded, variant, notation=notation) == canonical
    assert cube.bacon_encode("", "extended26") == ""
    for value in ["!", "é", "A\tB"]:
        with pytest.raises(ValueError):
            cube.bacon_encode(value, "extended26")
    for value in ["AAAA", "AAAAAA", "BBBBB", "AAAAA /", " AAAAA"]:
        with pytest.raises(ValueError):
            cube.bacon_decode(value, "extended26")
    with pytest.raises(ValueError):
        cube.bacon_encode("A", "unknown")
    with pytest.raises(ValueError):
        cube.bacon_decode("00000", "extended26")


def test_polybius_presets_custom_tokens_and_validation():
    assert cube.polybius_encode("AIJZ") == [[1, 1], [2, 4], [2, 4], [5, 5]]
    assert cube.polybius_encode("CK", preset="tap") == [[1, 3], [1, 3]]
    assert cube.polybius_decode([[1, 3]], preset="tap") == "C"
    kwargs = {"alphabet": "αβγδ", "side": 2, "aliases": [("a", "α")]}
    for base in [0, 1]:
        tokens = cube.polybius_encode(" δβα  γ ", index_base=base, **kwargs)
        assert cube.polybius_decode(tokens, index_base=base, **kwargs) == " δβα  γ "
    assert cube.polybius_encode("a", **kwargs) == [[1, 1]]
    assert cube.polybius_encode("") == []
    for tokens in [[[0, 1]], [[1, 6]], [[1]], [[1, 2, 3]], [[-1, 0]], [[True, True]]]:
        with pytest.raises((ValueError, OverflowError)):
            cube.polybius_decode(tokens)
    with pytest.raises(ValueError):
        cube.polybius_encode("A", alphabet="A", side=1, preset="tap")
    with pytest.raises(ValueError):
        cube.polybius_encode("A", alphabet="AAAA", side=2)
    with pytest.raises(ValueError):
        cube.polybius_encode("A", alphabet="ABCD", side=2, aliases=[("A", "B")])


def test_keyboard_native_custom_layouts_and_presets():
    assert cube.keyboard_remap("yYzZ", "qwerty", "qwertz") == "zZyY"
    assert cube.keyboard_remap("qwerty", "qwerty", "dvorak") == "',.pyf"
    assert cube.keyboard_remap("QWERTY", "qwerty", "dvorak") == '"<>PYF'
    source = {
        "geometry": "pair",
        "entries": [
            {"key": "one", "layer": "base", "symbol": "α"},
            {"key": "one", "layer": "shift", "symbol": "β"},
        ],
    }
    target = {
        "geometry": "pair",
        "entries": [
            {"key": "one", "layer": "base", "symbol": "🧊"},
            {"key": "one", "layer": "shift", "symbol": "!"},
        ],
    }
    assert cube.keyboard_remap("αβ?", source, target) == "🧊!?"
    assert cube.keyboard_remap("🧊!", target, source, unmapped="error") == "αβ"
    with pytest.raises(ValueError) as caught:
        cube.keyboard_remap("α?", source, target, unmapped="error")
    assert caught.value.offset == 2
    with pytest.raises(ValueError):
        cube.keyboard_remap("", source, {**target, "geometry": "other"})
    with pytest.raises(ValueError):
        cube.keyboard_remap("", {**source, "entries": source["entries"] * 2}, target)


def test_keypad_telephone_alternatives_and_explicit_groups():
    assert cube.keypad_encode("HELLO", "telephone") == "43556"
    assert cube.keypad_decode("43556", "telephone") == {
        "text": None,
        "alternatives": ["GHI", "DEF", "JKL", "JKL", "MNO"],
        "reversible": False,
    }
    assert cube.keypad_encode("HELLO", "multi_tap") == "44|33|555|555|666"
    assert cube.keypad_decode("44|33|555|555|666", "multi_tap") == {
        "text": "HELLO",
        "alternatives": [],
        "reversible": True,
    }
    custom = {"ascii_fold": False, "keys": [{"digit": "0", "symbols": " 0🧊"}]}
    assert cube.keypad_encode("0 🧊", "multi_tap", mapping=custom) == "00|0|000"
    assert cube.keypad_decode("00|0|000", "multi_tap", mapping=custom)["text"] == "0 🧊"
    for value in ["4444", "4433", "44||33", "0", "1", "|44", "44|"]:
        with pytest.raises(ValueError):
            cube.keypad_decode(value, "multi_tap")
    with pytest.raises(ValueError):
        cube.keypad_encode("1", "telephone")
    with pytest.raises(ValueError):
        cube.keypad_encode(
            "",
            "telephone",
            mapping={"ascii_fold": True, "keys": [{"digit": "2", "symbols": "Aa"}]},
        )


def test_classes_provenance_filtering_and_native_definitions():
    assert cube.map_character_classes("AEIOU")["mapped_text"] == "00000"
    assert cube.map_character_classes("bCd")["mapped_text"] == "111"
    report = cube.map_character_classes("éa!B", unmapped="filter")
    assert report["records"] == [
        {"scalar_index": 1, "byte_offset": 2, "class": "vowel", "emitted": "0"},
        {"scalar_index": 3, "byte_offset": 4, "class": "consonant", "emitted": "1"},
    ]
    assert report["filtered_count"] == 2
    assert report["filtered"] == [
        {"scalar_index": 0, "byte_offset": 0},
        {"scalar_index": 2, "byte_offset": 3},
    ]
    for y, expected in [("vowel", "0"), ("consonant", "1"), ("unmapped", "Y")]:
        assert (
            cube.map_character_classes("Y", y=y, unmapped="preserve")["mapped_text"]
            == expected
        )
    custom = {
        "ascii_fold": False,
        "classes": [{"name": "greek", "characters": "αβ", "symbol": "🧊"}],
    }
    assert cube.map_character_classes("αβ", definition=custom)["mapped_text"] == "🧊🧊"
    for kwargs in [{}, {"unmapped": "filter"}]:
        with pytest.raises(ValueError):
            cube.map_character_classes("Y", **kwargs)
    with pytest.raises(ValueError):
        cube.map_character_classes("1")
    with pytest.raises(ValueError):
        cube.map_character_classes("", y="vowel", definition=custom)
    with pytest.raises(ValueError):
        cube.map_character_classes(
            "",
            definition={
                "ascii_fold": True,
                "classes": [{"name": "x", "characters": "aA", "symbol": "0"}],
            },
        )


def test_interface_limits_and_pipeline_discovery():
    with pytest.raises(ValueError):
        cube.bacon_encode("A" * (1024 * 1024 + 1), "extended26")
    for operation in [
        lambda s: cube.polybius_encode(s),
        lambda s: cube.keypad_encode(s, "telephone"),
        lambda s: cube.map_character_classes(s),
    ]:
        with pytest.raises(ValueError):
            operation("A" * 65537)
    for name in [
        "cipher.bacon.encode",
        "cipher.polybius.decode",
        "symbol.keypad.decode",
        "text.keyboard.remap",
        "text.class.map",
    ]:
        assert cube.describe_operation(name)["pipeline"] is None


def test_bounded_native_definition_and_pair_transport():
    with pytest.raises(ValueError):
        cube.polybius_decode([None] * 65537)
    with pytest.raises(ValueError):
        cube.polybius_encode("", alphabet="A", side=1, aliases=[("a", "A")] * 4097)
    with pytest.raises(ValueError):
        cube.keypad_encode(
            "",
            "telephone",
            mapping={
                "ascii_fold": False,
                "keys": [{"digit": "0", "symbols": "x" * 65537}],
            },
        )
    with pytest.raises(ValueError):
        cube.map_character_classes(
            "",
            definition={
                "ascii_fold": False,
                "classes": [{"name": "x", "characters": "A", "symbol": "x" * 65537}],
            },
        )
    with pytest.raises(ValueError):
        cube.map_character_classes(
            "",
            definition={
                "ascii_fold": False,
                "classes": [
                    {"name": "x", "characters": "A", "symbol": "0"},
                    {"name": "y", "characters": "B", "symbol": "01"},
                ],
            },
        )
