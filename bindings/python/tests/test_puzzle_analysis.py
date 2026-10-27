import collections
import itertools
import json
from fractions import Fraction

import cube
import pytest


def fraction(value):
    assert type(value["numerator"]) is int
    assert type(value["denominator"]) is int
    return Fraction(value["numerator"], value["denominator"])


def test_anagram_unordered_oracle_aliases_and_shared_subtraction():
    dictionary = ["ab", "a", "b", "ba", "aa"]
    for source, reuse in itertools.product(
        ["", "ab", "aa", "aab", "abab", "x"], [False, True]
    ):
        r = cube.anagram_search(
            source, dictionary, options={"reuse": reuse, "max_words": 3}
        )
        expected = set()
        for count in range(4):
            combinations = (
                itertools.combinations_with_replacement
                if reuse
                else itertools.combinations
            )
            for path in combinations(range(len(dictionary)), count):
                if collections.Counter(
                    "".join(dictionary[i] for i in path)
                ) == collections.Counter(source):
                    expected.add(path)
        assert r["complete"]
        assert {tuple(path) for path in r["solutions"]} == expected
        for path in r["solutions"]:
            result = "".join(r["dictionary"][i]["text"] for i in path)
            assert cube.multiset_subtract(source, result)["equal"]
    r = cube.anagram_search(
        "éa",
        ["e\u0301", "é", "A", "a"],
        profile={"normalization": "NFC", "case": "ascii_lower"},
    )
    assert r["dictionary"] == [
        {"text": "é", "aliases": [0, 1]},
        {"text": "a", "aliases": [2, 3]},
    ]
    assert r["solutions"] == [[0, 1]]


def test_anagram_complete_no_solution_and_explicit_incomplete_prefixes():
    d = ["ab", "a", "b"]
    r = cube.anagram_search("ab", d)
    assert r["solutions"] == [[0], [1, 2]]
    partial = cube.anagram_search("ab", d, options={"max_results": 1})
    assert partial["solutions"] == r["solutions"][:1]
    assert not partial["complete"] and partial["stop_reason"] == "result_limit"
    assert cube.anagram_search("ab", d, options={"max_work": r["work"]})["complete"]
    partial = cube.anagram_search("ab", d, options={"max_work": r["work"] - 1})
    assert not partial["complete"] and partial["stop_reason"] == "work_limit"
    assert cube.anagram_search("x", d)["complete"]
    assert cube.anagram_search("x", d)["solutions"] == []
    assert cube.anagram_search("", ["a"])["solutions"] == [[]]
    assert cube.anagram_search("aa", ["a"])["solutions"] == []
    assert cube.anagram_search("aa", ["a"], options={"reuse": True})["solutions"] == [
        [0, 0]
    ]
    for key in ["memory_bytes_upper_bound", "report_bytes_upper_bound"]:
        option = "max_memory_bytes" if key.startswith("memory") else "max_report_bytes"
        with pytest.raises(ValueError):
            cube.anagram_search("ab", d, options={option: r[key] - 1})


@pytest.mark.parametrize(
    "dictionary", [[""], ["a" * 65], ["a"] * 10001, [False], {"a": 1}]
)
def test_anagram_bad_dictionaries(dictionary):
    with pytest.raises(ValueError):
        cube.anagram_search("a", dictionary)


def test_ladder_paths_ties_and_validation_evidence():
    d = ["dog", "cog", "dot", "cot", "cat", "cot"]
    r = cube.word_ladder_shortest(d, "cat", "dog")
    assert [r["dictionary"][i]["text"] for i in r["path"]] == [
        "cat",
        "cot",
        "cog",
        "dog",
    ]
    assert r["edge_count"] == 5 and r["complete"]
    assert r["dictionary"][2]["aliases"] == [3, 5]
    assert cube.word_ladder_validate(d, ["cat", "cot", "cog", "dog"])["valid"]
    for path, reason in [
        (["cat", "dog"], "not_one_substitution"),
        (["cat", "cat"], "not_one_substitution"),
        (["cat", "hat"], "not_in_dictionary"),
    ]:
        failure = cube.word_ladder_validate(d, path)["failure"]
        assert failure == {"position": 1, "step": 0, "reason": reason}
    assert (
        cube.word_ladder_shortest(["cat", "dog"], "cat", "dog")["status"] == "no_path"
    )
    assert len(cube.word_ladder_shortest(d, "cat", "cat")["path"]) == 1
    r = cube.word_ladder_shortest(
        ["é!", "e\u0301x", "éx"], "éx", "é!", profile={"normalization": "NFC"}
    )
    assert len(r["path"]) == 2
    # Numeric graph IDs allow words longer than the graph's ID-byte limit.
    r = cube.word_ladder_shortest(
        ["🧊" * 32, "🧊" * 31 + "雪"], "🧊" * 32, "🧊" * 31 + "雪"
    )
    assert len(r["path"]) == 2


@pytest.mark.parametrize(
    "call",
    [
        lambda: cube.word_ladder_shortest(["a", "ab"], "a", "ab"),
        lambda: cube.word_ladder_shortest(["cat"], "cat", "dog"),
        lambda: cube.word_ladder_validate(["a"], []),
        lambda: cube.word_ladder_shortest(["a"], "a", "a", limits={"max_work": 0}),
        lambda: cube.word_ladder_shortest(
            ["a"], "a", "a", profile={"filter": "ascii_letters"}
        ),
    ],
)
def test_ladder_invalid_models_and_limits(call):
    with pytest.raises(ValueError):
        call()


def test_finite_joint_conditioning_and_signed_expectation():
    model = {
        "outcomes": [
            {"id": str(i), "weight": 1, "value": v} for i, v in enumerate([-4, 1, 1, 1])
        ],
        "event_a": ["0", "1"],
        "event_b": ["0", "2"],
    }
    r = cube.finite_probability(model)
    assert fraction(r["probability_a"]) == Fraction(1, 2)
    assert fraction(r["intersection"]) == Fraction(1, 4)
    assert fraction(r["conditional_a_given_b"]) == Fraction(1, 2)
    assert [fraction(p["probability"]) for p in r["posterior"]] == [
        Fraction(1, 2),
        0,
        Fraction(1, 2),
        0,
    ]
    assert r["expected_value"] == {"negative": True, "numerator": 1, "denominator": 4}
    assert r["total_mass"] == 4 and type(r["total_mass"]) is int
    assert r["assumptions"]["outcomes"][0]["id"] == "0"
    assert r == cube.finite_probability(json.dumps(model))
    model["outcomes"][0]["value"] = -3
    assert cube.finite_probability(model)["expected_value"] == {
        "negative": False,
        "numerator": 0,
        "denominator": 1,
    }


def die(values):
    return [{"value": value, "weight": 1} for value in values]


def test_dice_independent_cartesian_oracle_weighted_faces_and_exact_u128():
    d = {"dice": [die(range(1, 7)), die(range(1, 7))]}
    r = cube.dice_sum(d)
    assert r["total_mass"] == 36
    expected = collections.Counter(
        sum(v) for v in itertools.product(range(1, 7), repeat=2)
    )
    assert {p["value"]: p["mass"] for p in r["distribution"]} == expected
    assert sum(fraction(p["probability"]) for p in r["distribution"]) == 1
    assert r["expected_value"] == {"negative": False, "numerator": 7, "denominator": 1}
    r = cube.dice_sum(
        {
            "dice": [
                [
                    {"value": -1, "weight": 3},
                    {"value": -1, "weight": 2},
                    {"value": 2, "weight": 1},
                    {"value": 999, "weight": 0},
                ]
            ]
        }
    )
    assert [(v["value"], v["mass"]) for v in r["distribution"]] == [(-1, 5), (2, 1)]
    assert r["expected_value"] == {"negative": True, "numerator": 1, "denominator": 2}
    weight = 2**64 - 1
    r = cube.dice_sum({"dice": [[{"value": 0, "weight": weight}]] * 2})
    assert r["total_mass"] == weight**2 and type(r["total_mass"]) is int
    assert cube.dice_sum({"dice": []})["distribution"][0]["probability"] == {
        "numerator": 1,
        "denominator": 1,
    }


def test_dice_strict_win_tie_loss_and_nontransitive_relations():
    a, b, c = [
        {"dice": [die(v)]}
        for v in [[2, 2, 4, 4, 9, 9], [1, 1, 6, 6, 8, 8], [3, 3, 5, 5, 7, 7]]
    ]
    for left, right in [(a, b), (b, c), (c, a)]:
        r = cube.dice_compare({"left": left, "right": right})
        assert fraction(r["win"]) == Fraction(5, 9)
        assert fraction(r["win"]) + fraction(r["tie"]) + fraction(r["loss"]) == 1
    assert fraction(cube.dice_compare({"left": a, "right": a})["tie"]) == Fraction(1, 3)


@pytest.mark.parametrize("weight", [True, -1, 1.5, 2**64, "01", "-1", "1.0", None])
def test_finite_and_dice_weights_reject_invalid_native_types(weight):
    for model, function in [
        (
            {"outcomes": [{"id": "x", "weight": weight}], "event_a": []},
            cube.finite_probability,
        ),
        ({"dice": [[{"value": 0, "weight": weight}]]}, cube.dice_sum),
    ]:
        with pytest.raises(ValueError):
            function(model)


def test_probability_bad_events_zero_mass_overflow_and_budget_failures():
    for model in [
        {"outcomes": [{"id": "x", "weight": 0}], "event_a": []},
        {"outcomes": [{"id": "x", "weight": 1}], "event_a": ["unknown"]},
        {"outcomes": [{"id": "x", "weight": 1}], "event_a": ["x", "x"]},
        {"outcomes": [{"id": "x", "weight": 1}], "event_a": [], "event_b": []},
    ]:
        with pytest.raises(ValueError):
            cube.finite_probability(model)
    with pytest.raises(ValueError):
        cube.dice_sum({"dice": [[{"value": 0, "weight": 2**64 - 1}]] * 3})
    model = {"dice": [die([1, 2, 3])]}
    r = cube.dice_sum(model)
    assert cube.dice_sum(model, limits={"max_work": r["work"]}) == r
    for key, value in [
        ("max_work", r["work"] - 1),
        ("max_report_bytes", r["report_bytes_upper_bound"] - 1),
        ("max_memory_bytes", r["memory_bytes_upper_bound"] - 1),
    ]:
        with pytest.raises(ValueError):
            cube.dice_sum(model, limits={key: value})
    cyclic = []
    cyclic.append(cyclic)
    with pytest.raises(ValueError):
        cube.dice_sum({"dice": cyclic})
    with pytest.raises(ValueError):
        cube.finite_probability('{"outcomes":[],"event_a":[],"event_a":[]}')


def test_spatial_exact_physical_moments_correlation_and_limits():
    image = cube.Image.from_samples(4, 1, "gray", bytes([0, 255, 0, 255]))
    r = image.autocorrelation(
        "gray",
        [{"dx": 0, "dy": 0}, {"dx": 1, "dy": 0}, {"dx": -1, "dy": 0}],
        region=(0, 0, 4, 1),
    )
    assert [m["correlation"] for m in r["measurements"]] == [1.0, -1.0, -1.0]
    assert r["measurements"][1]["squared_difference"] == 3 * 255**2
    assert r["measurements"][1]["sum_xy"] == 0
    assert r["measurements"][1]["source_overlap"] == {
        "x": 0,
        "y": 0,
        "width": 3,
        "height": 1,
    }
    assert image.autocorrelation("gray", [], region=(0, 0, 4, 1))["measurements"] == []
    constant = cube.Image.from_samples(1, 1, "gray", bytes([9]))
    assert (
        constant.autocorrelation("gray", [{"dx": 0, "dy": 0}], region=(0, 0, 1, 1))[
            "measurements"
        ][0]["correlation"]
        is None
    )
    for limits in [
        {"max_work": r["work"] - 1},
        {"max_report_bytes": r["report_bytes_upper_bound"] - 1},
        {"max_memory_bytes": r["memory_bytes_upper_bound"] - 1},
    ]:
        with pytest.raises(ValueError):
            image.autocorrelation(
                "gray",
                r'[ {"dx":0,"dy":0},{"dx":1,"dy":0},{"dx":-1,"dy":0}]',
                region=(0, 0, 4, 1),
                limits=limits,
            )
    for channel, lags in [
        ("red", [{"dx": 0, "dy": 0}]),
        ("gray", [{"dx": 4, "dy": 0}]),
        ("gray", [{"dx": True, "dy": 0}]),
        ("gray", [{"dx": 0, "dy": 0}] * 257),
    ]:
        with pytest.raises(ValueError):
            image.autocorrelation(channel, lags, region=(0, 0, 4, 1))


def test_builtin_transport_does_not_invoke_custom_collection_iterators():
    class HostileList(list):
        def __iter__(self):
            raise AssertionError("custom iterator must not run")

    class HostileDict(dict):
        def items(self):
            raise AssertionError("custom items must not run")

    for value in [HostileList(["a"]), HostileDict(dice=[])]:
        with pytest.raises(ValueError):
            cube.anagram_search("a", value)
    for value in [1, "true", None]:
        with pytest.raises(ValueError):
            cube.anagram_search("a", ["a"], options={"reuse": value})
