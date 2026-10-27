"""Native monoalphabetic APIs: exact transport, canonicalization and budgets."""

import cube
import pytest

HOMOPHONIC = {
    "ascii_uppercase": True,
    "entries": [
        {"symbol": "A", "tokens": ["1", "22"]},
        {"symbol": "B", "tokens": ["3"]},
    ],
}
MONOME = {
    "digits": "6318927054",
    "prefixes": "63",
    "alphabet": "NOTARIESBCDFGHKLMPQUVWXY",
    "aliases": [{"from": "J", "to": "I"}, {"from": "Z", "to": "Y"}],
}
TRIDIGITAL = {
    "digits": "6703528149",
    "prefixes": "",
    "alphabet": "DRAGONFLYBCEHIJKMPQSTUVWXZ",
}
STRADDLING = {
    "digits": "0123456789",
    "prefixes": "37",
    "alphabet": "ABCDEFGHIJKLMNOPQRSTUVWXYZ #",
    "digit_escape": "#",
}
GRANDPRE = {
    "rows": [
        "LADYBUGS",
        "AZIMUTHS",
        "CALFSKIN",
        "QUACKISH",
        "UNJOVIAL",
        "EVULSION",
        "ROWDYISM",
        "SEXTUPLY",
    ],
    "row_labels": "12345678",
    "column_labels": "12345678",
}


def test_substitution_preserves_exact_scalar_identity_and_explicit_policy():
    assert (
        cube.substitution_encode(
            "ABCXYZ", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", keyword="BALLOON"
        )
        == "BALXYZ"
    )
    encoded = cube.substitution_encode(
        "Aé\0e\u0301", "Aé\0e\u0301", target="猫B\0\u0301e"
    )
    assert encoded == "猫B\0\u0301e"
    assert (
        cube.substitution_decode(encoded, "Aé\0e\u0301", target="猫B\0\u0301e")
        == "Aé\0e\u0301"
    )
    assert (
        cube.substitution_encode("ax!", "a", target="x", unmapped="preserve") == "xx!"
    )
    for kwargs in [
        {},
        {"target": "AB", "keyword": "A"},
        {"target": "AA"},
        {"target": "C"},
        {"keyword": "X"},
    ]:
        with pytest.raises(ValueError) as error:
            cube.substitution_encode("A", "AB", **kwargs)
        assert error.value.code == "invalid_parameter"
    with pytest.raises(ValueError) as error:
        cube.substitution_decode("x!", "a", target="x")
    assert error.value.operation == "cipher.substitution.decode"
    assert error.value.offset == 1


@pytest.mark.parametrize(
    "function,text,expected",
    [
        (cube.rot5, "0123456789 Aa 世界\0", "5678901234 Aa 世界\0"),
        (cube.rot18, "Hello 123!世界", "Uryyb 678!世界"),
        (cube.rot47, "Hello, World!", "w6==@[ (@C=5P"),
    ],
)
def test_rotations_are_exact_native_involutions(function, text, expected):
    assert function(text) == expected
    all_scalars = "".join(chr(i) for i in range(128)) + "é猫🦀\u0301"
    assert function(function(all_scalars)) == all_scalars
    assert len(function(all_scalars).encode()) == len(all_scalars.encode())
    assert len(function("A" * 1048576)) == 1048576
    with pytest.raises(ValueError) as error:
        function("A" * 1048577)
    assert error.value.code == "resource_limit"


def test_homophones_reset_counters_and_use_exact_structured_tokens():
    assert cube.homophonic_encode("aabA", HOMOPHONIC) == ["1", "22", "3", "1"]
    assert cube.homophonic_encode("aabA", HOMOPHONIC) == ["1", "22", "3", "1"]
    assert cube.homophonic_decode(["1", "22", "3"], HOMOPHONIC) == "AAB"
    assert cube.homophonic_encode(
        "AA", HOMOPHONIC, counters=[{"symbol": "A", "index": 65535}]
    ) == ["22", "1"]
    for index in [-1, 65536, True, 0.5, "1"]:
        with pytest.raises(ValueError):
            cube.homophonic_encode(
                "A", HOMOPHONIC, counters=[{"symbol": "A", "index": index}]
            )
    for tokens in [["122"], [True], [1], [["1"]]]:
        with pytest.raises(ValueError) as error:
            cube.homophonic_decode(tokens, HOMOPHONIC)
        assert error.value.operation == "cipher.homophonic.decode"
    assert len(cube.homophonic_encode("A" * 65536, HOMOPHONIC)) == 65536
    with pytest.raises(ValueError) as error:
        cube.homophonic_encode("A" * 65537, HOMOPHONIC)
    assert error.value.code == "resource_limit"
    escaped = {
        "ascii_uppercase": False,
        "entries": [{"symbol": "A", "tokens": ["\0" * 256]}],
    }
    with pytest.raises(ValueError) as error:
        cube.homophonic_encode("A" * 1000, escaped)
    assert error.value.code == "resource_limit"


def test_digit_profiles_preserve_ambiguity_and_reject_malformed_codes():
    assert cube.monome_dinome_encode("highf", MONOME) == "600676062"
    assert cube.monome_dinome_decode("600676062", MONOME) == "HIGHF"
    assert (
        cube.monome_dinome_decode(cube.monome_dinome_encode("jz", MONOME), MONOME)
        == "IY"
    )
    assert cube.tridigital_encode("the ides", TRIDIGITAL) == "03095607"
    assert cube.tridigital_decode("09", TRIDIGITAL) == [["A", "E", "T"], [" "]]
    assert (
        cube.straddling_decode(
            cube.straddling_encode("ABC 120XYZ", STRADDLING), STRADDLING
        )
        == "ABC 120XYZ"
    )
    for code in ["6", "60 0", "66", "600a"]:
        with pytest.raises(ValueError) as error:
            cube.monome_dinome_decode(code, MONOME)
        assert error.value.code == "invalid_input"
    for definition in [
        dict(TRIDIGITAL, digits="0000000000"),
        dict(TRIDIGITAL, alphabet="ABC"),
        dict(TRIDIGITAL, unexpected=True),
    ]:
        with pytest.raises(ValueError):
            cube.tridigital_encode("A", definition)


def test_grandpre_repeated_cells_labels_and_null_spaces_are_native():
    assert cube.grandpre_decode([[8, 4], [2, 7], [8, 2]], GRANDPRE) == "THE"
    assert cube.grandpre_encode("AA", GRANDPRE) == [[1, 2], [2, 1]]
    encoded = cube.grandpre_encode("the  keyword", GRANDPRE)
    assert cube.grandpre_decode(encoded, GRANDPRE) == "THE  KEYWORD"
    ten = {
        "rows": ["A" * 10] * 10,
        "row_labels": "1234567890",
        "column_labels": "1234567890",
    }
    assert cube.grandpre_encode("A" * 100, ten)[99] == [0, 0]
    for tokens in [[[True, 1]], [[1.5, 1]], [[1]], [[9, 9]], ["11"]]:
        with pytest.raises(ValueError):
            cube.grandpre_decode(tokens, GRANDPRE)
    with pytest.raises(ValueError) as error:
        cube.grandpre_decode([[1, 1]] * 65537, GRANDPRE)
    assert error.value.code == "resource_limit"


def test_homophonic_exact_serialized_output_ceiling():
    import json

    mapping = {
        "ascii_uppercase": False,
        "entries": [{"symbol": "A", "tokens": ["x" * 22]}],
    }
    encoded = cube.homophonic_encode("A" * 41943, mapping)
    assert len(json.dumps(encoded, separators=(",", ":")).encode()) == 1048576
    assert cube.homophonic_decode(encoded, mapping) == "A" * 41943
    for function, data in [
        (cube.homophonic_encode, "A" * 41944),
        (cube.homophonic_decode, encoded + ["x" * 22]),
    ]:
        with pytest.raises(ValueError) as error:
            function(data, mapping)
        assert error.value.code == "resource_limit"
