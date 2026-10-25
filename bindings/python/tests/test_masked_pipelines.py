"""Typed list parameters and masked step audits travel through the Rust engine."""

import json

import cube
import pytest


def step(**changes):
    parameters = dict(
        selector="fibonacci",
        exclude={"unsigned_list": [13, 21]},
        index_base=1,
        selected="atbash",
        otherwise="caesar",
        otherwise_shift=3,
    )
    parameters.update(changes)
    return dict(operation="text.transform.masked", version=1, parameters=parameters)


def recipe(steps):
    return json.dumps(
        dict(
            format_version=1,
            engine_api=1,
            input=dict(source="slot", name="text", type="text"),
            output=dict(type="text"),
            steps=steps,
        )
    )


def test_pipeline_recipe_and_direct_report_parity_with_repeated_overrides():
    original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    steps = [step(overrides={"text_list": ["1=雪", "1=🧊"]})]
    direct = cube.transform_masked(
        original,
        selector="fibonacci",
        exclude=[13, 21],
        index_base=1,
        selected="atbash",
        otherwise="caesar",
        otherwise_shift=3,
        overrides=[(1, "雪"), (1, "🧊")],
    )
    result = cube.execute_pipeline(steps, original, capture=[0])
    assert result["output"] == direct["final_text"]
    assert result["masked_audits"] == {0: direct}
    assert result["intermediates"] == {0: direct["final_text"]}
    assert cube.execute_pipeline(steps, original)["masked_audits"] == {0: direct}
    assert cube.validate_pipeline(steps, "text")["engine_api"] == 1
    document = recipe(steps)
    assert cube.execute_recipe(document, {"text": original})["masked_audits"] == {
        0: direct
    }
    checked = cube.check_recipe(document, {"text": original}, direct["final_text"])
    assert checked["passed"] is True
    assert checked["masked_audits"] == {0: direct}
    assert cube.describe_operation("text.transform.masked")["pipeline"] == {
        "version": 1
    }


def test_known_answer_and_composition_keep_the_step_audit():
    original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    steps = [step(), dict(operation="cipher.rot13", version=1)]
    result = cube.execute_pipeline(steps, original)
    report = result["masked_audits"][0]
    assert report["selected_positions"] == [1, 2, 3, 5, 8]
    assert report["deterministic_text"] == "ZYXGVIJSLMNOPQRSTUVWXYZABC"
    assert report["final_text"][12] == "P" and report["final_text"][20] == "X"
    assert result["output"] == cube.rot13(report["final_text"])


def test_empty_list_tags_both_bases_unicode_and_shifts():
    original = "A !éB e\u0301Z"
    for parameters, direct in [
        (
            dict(
                selector="indices",
                indices={"unsigned_list": []},
                selected="identity",
                overrides={"text_list": []},
            ),
            dict(selector="indices", indices=[], selected="identity"),
        ),
        (
            dict(
                selector="indices",
                indices={"unsigned_list": [4, 0, 0, 2]},
                selected="rot13",
            ),
            dict(selector="indices", indices=[4, 0, 0, 2], selected="rot13"),
        ),
        (
            dict(
                selector="every_nth",
                nth=2,
                index_base=1,
                selected="caesar",
                selected_shift=-(2**31),
                otherwise="atbash",
            ),
            dict(
                selector="every_nth",
                nth=2,
                index_base=1,
                selected="caesar",
                selected_shift=-(2**31),
                otherwise="atbash",
            ),
        ),
    ]:
        steps = [
            dict(operation="text.transform.masked", version=1, parameters=parameters)
        ]
        assert cube.execute_pipeline(steps, original)["masked_audits"][
            0
        ] == cube.transform_masked(original, **direct)
    assert cube.execute_pipeline([step()], "")["output"] == ""


@pytest.mark.parametrize(
    "value",
    [
        [],
        [1, 2],
        {"unsigned_list": [-1]},
        {"unsigned_list": [1.0]},
        {"unsigned_list": [True]},
        {"unsigned_list": ["1"]},
        {"unsigned_list": [[1]]},
        {"unsigned_list": [2**64]},
        {"unsigned_list": [], "unknown": 1},
        None,
    ],
)
def test_invalid_list_values_fail_in_shared_preflight(value):
    with pytest.raises(ValueError):
        cube.validate_pipeline([step(exclude=value)], "text")


def test_limits_and_context_are_preserved():
    for limit, boundary in [
        ("max_capture_bytes", 646),
        (
            "max_work_bytes",
            cube.execute_pipeline([step()], "ABCDEFGHIJKLMNOPQRSTUVWXYZ")[
                "work_bytes_upper_bound"
            ],
        ),
    ]:
        assert cube.execute_pipeline(
            [step()], "ABCDEFGHIJKLMNOPQRSTUVWXYZ", limits={limit: boundary}
        )
        with pytest.raises(ValueError) as error:
            cube.execute_pipeline(
                [step()], "ABCDEFGHIJKLMNOPQRSTUVWXYZ", limits={limit: boundary - 1}
            )
        assert error.value.code == "resource_limit"
        assert error.value.step == 0
    with pytest.raises(ValueError) as error:
        cube.validate_pipeline(
            [dict(operation="cipher.rot13", version=1), step(selected="caesar")], "text"
        )
    assert error.value.step == 1
    for overrides in [["0=x"], ["1="], ["1=e\u0301"], ["99=x"]]:
        with pytest.raises(ValueError):
            cube.execute_pipeline([step(overrides={"text_list": overrides})], "ABC")
    assert "masked_audits" not in cube.execute_pipeline([], "ABC")
