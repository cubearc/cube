"""Historical manual vectors through the actual native extension."""

import cube
import pytest

ORDER = [
    3,
    10,
    12,
    16,
    6,
    21,
    7,
    14,
    1,
    23,
    17,
    5,
    20,
    25,
    4,
    11,
    13,
    18,
    8,
    22,
    9,
    15,
    2,
    24,
    19,
]
PLAIN = (
    "HAVEJUSTREACHEDEASTERNEDGEOFWOODSALONGFIVEFIVETWOD"
    "ASHFIVENINETWOROADWILLREMAININOBSERVATION"
)
CIPHER = (
    "JUKLDYKITZIIVCYCVUYVPYWHJYUYEZDHVUZDBQPOZMCFNBJJIX"
    "EAPTHYOWHKWWTNYGMPRZJIFADMEQRHBPOJTYUQNTW"
)


def settings():
    return {"profile": "m94_v1", "order": ORDER, "row_offsets": [20, 10, 9, 4]}


def test_m94_manual_known_answers_and_exact_failures():
    assert cube.m94_encode(PLAIN, settings()) == {"text": CIPHER}
    assert cube.m94_decode(CIPHER, settings()) == {"text": PLAIN}
    p = settings()
    p["row_offsets"] = []
    assert cube.m94_encode("", p) == {"text": ""}
    with pytest.raises(ValueError) as caught:
        cube.m94_encode("SECRET", p)
    assert caught.value.code == "invalid_parameter"
    assert caught.value.operation == "cipher.m94.encode"
    assert "SECRET" not in str(caught.value)
    p["row_offsets"] = [1]
    with pytest.raises(ValueError) as caught:
        cube.m94_encode("a", p)
    assert caught.value.code == "invalid_input"


def test_m94_native_input_bound():
    p = settings()
    p["row_offsets"] = [1] * 41944
    assert len(cube.m94_encode("A" * 1048576, p)["text"]) == 1048576
    with pytest.raises(ValueError) as caught:
        cube.m94_encode("A" * 1048577, p)
    assert caught.value.code == "resource_limit"


def test_configured_wheel_and_strip_native_known_answers():
    p = {
        "profile": "jefferson_configured_v1",
        "wheels": [
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "AZYXWVUTSRQPONMLKJIHGFEDCB",
            "QWERTYUIOPASDFGHJKLZXCVBNM",
        ],
        "order": [1, 2, 3],
        "row_offsets": [1, 25],
    }
    assert cube.jefferson_encode("AAJAAA", p)["text"] == "BZKZBP"
    assert cube.jefferson_decode("BZKZBP", p)["text"] == "AAJAAA"
    p["order"] = [1, 1, 3]
    with pytest.raises(ValueError):
        cube.jefferson_encode("AAJAAA", p)
    p = {
        "profile": "m138a_configured_v1",
        "strip_series": "qualification:alternating-v1",
        "strips": [
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ" if i % 2 == 0 else "AZYXWVUTSRQPONMLKJIHGFEDCB"
            for i in range(100)
        ],
        "order": list(range(1, 31)),
        "row_offsets": [1, 2],
    }
    assert cube.m138a_encode("A" * 31, p)["text"] == "BZ" * 15 + "C"
    assert cube.m138a_decode("BZ" * 15 + "C", p)["text"] == "A" * 31
    p["strips"] = p["strips"][:99]
    with pytest.raises(ValueError):
        cube.m138a_encode("A" * 31, p)
