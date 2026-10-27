import cube
import pytest


def test_native_execution_exact_bytes_widths_and_eof():
    assert cube.brainfuck_execute("++++++++[>++++++++<-]>+.") == b"A"
    data = bytes(range(256))
    for bits in [8, 16, 32]:
        options = {"cell_bits": bits}
        assert cube.brainfuck_execute(",." * 256, data, options=options) == data
        report = cube.brainfuck_report(
            "-.", options=options, diagnostics={"tape_len": 1}
        )
        assert report["completed"]
        assert report["output"] == b"\xff"
        assert report["tape"] == [{"index": 0, "value": 2**bits - 1}]
        assert report["instructions"] == 2
    assert cube.brainfuck_execute("") == b""
    assert cube.brainfuck_execute("-,.", options={"eof": "zero"}) == b"\0"
    assert cube.brainfuck_execute("-,.", options={"eof": "unchanged"}) == b"\xff"


def test_native_failure_and_incomplete_reports():
    for code, parameter in [("[", "source"), ("<", "pointer"), (",", "input")]:
        with pytest.raises(ValueError) as exc:
            cube.brainfuck_execute(code)
        assert exc.value.parameter == parameter
        assert exc.value.code == "invalid_input"
        assert exc.value.offset == 0
    with pytest.raises(ValueError) as exc:
        cube.brainfuck_execute(".+[]", limits={"max_instructions": 4})
    assert exc.value.code == "resource_limit"
    r = cube.brainfuck_report(
        ".+[]",
        limits={"max_instructions": 4},
        diagnostics={"max_trace_records": 2, "max_trace_bytes": 1000, "tape_len": 1},
    )
    assert not r["completed"]
    assert r["stop_reason"] == "instruction_limit"
    assert r["error"]["parameter"] == "max_instructions"
    assert r["instructions"] == 4
    assert r["output"] == b"\0"
    assert not r["trace_complete"]
    assert [t["command"] for t in r["trace"]] == [".", "+"]
    assert r == cube.brainfuck_report(
        ".+[]",
        limits={"max_instructions": 4},
        diagnostics={"max_trace_records": 2, "max_trace_bytes": 1000, "tape_len": 1},
    )


def test_native_comparisons_budgets_and_discovery():
    c = cube.brainfuck_compare("-.", options={"widths": [32, 8, 16]})
    assert c["all_completed"]
    assert c["equal_completed_outputs"] is True
    assert [r["options"]["cell_bits"] for r in c["runs"]] == [8, 16, 32]
    assert all(r["output"] == b"\xff" for r in c["runs"])
    c = cube.brainfuck_compare("+" * 256 + "[.-]", options={"widths": [8, 16, 32]})
    assert c["all_completed"]
    assert c["equal_completed_outputs"] is False
    assert c["runs"][0]["output"] == b""
    assert c["runs"][1]["output"] == bytes([0] + list(range(255, 0, -1)))
    c = cube.brainfuck_compare(
        "+[]",
        options={"widths": [8, 16, 32]},
        limits={"max_instructions": 5},
        aggregate={"max_instructions": 7},
    )
    assert not c["all_completed"]
    assert c["equal_completed_outputs"] is None
    assert c["instructions"] == 7
    assert [r["instructions"] for r in c["runs"]] == [5, 2, 0]
    for operation in ["execute", "report", "compare"]:
        d = cube.describe_operation(f"esolang.brainfuck.{operation}")
        assert d["pipeline"] is None


@pytest.mark.parametrize("value", [-1, True, 1.5, "1", 2**64, None])
def test_native_configuration_rejects_integer_coercion(value):
    for function, kwargs in [
        (cube.brainfuck_execute, {"options": {"cell_bits": value}}),
        (cube.brainfuck_execute, {"limits": {"max_instructions": value}}),
        (cube.brainfuck_report, {"diagnostics": {"tape_len": value}}),
        (cube.brainfuck_compare, {"options": {"widths": [value]}}),
    ]:
        with pytest.raises(ValueError):
            function("", **kwargs)


@pytest.mark.parametrize(
    "kwargs",
    [
        {"options": {"cell_bits": 64}},
        {"options": {"eof": "guess"}},
        {"options": {"overflow": "checked"}},
        {"limits": {"max_output_bytes": 1024**2 + 1}},
        {"limits": {"max_memory_bytes": 0}},
    ],
)
def test_native_invalid_policies_and_limits(kwargs):
    with pytest.raises(ValueError):
        cube.brainfuck_execute("", **kwargs)


def test_comparisons_require_explicit_width_selection():
    with pytest.raises(TypeError):
        cube.brainfuck_compare("")
    with pytest.raises(ValueError) as exc:
        cube.brainfuck_compare("", options={})
    assert exc.value.parameter == "widths"
