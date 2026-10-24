import cube
import pytest


def test_triage_is_explained_deterministic_and_bounded():
    data = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
    result = cube.triage_input(data)
    assert result == cube.triage_input(data)
    assert result["inspected_bytes"] == 12
    assert result["input_size"] == len(data)
    suggestion = next(
        s for s in result["suggestions"] if s["operation"] == "image.png.inspect"
    )
    assert suggestion["rule"] == "triage.png_chunks.v1"
    assert suggestion["evidence"] == ["fact-2"]
    assert "alone does not validate" in suggestion["uncertainty"]
    assert "evidence_hex" not in str(result)
    with pytest.raises(ValueError) as error:
        cube.triage_input(data, max_read_bytes=11)
    assert error.value.code == "resource_limit"
    assert cube.triage_input(b"", max_read_bytes=0)["input_size"] == 0


def test_native_sessions_record_private_provenance_and_verified_replay():
    import json

    data = b"\x00\xffprivate"
    session = cube.InvestigationSession(data, identify=True)
    steps = [{"operation": "encoding.hex.encode", "version": 1}]
    assert session.execute_pipeline(data, steps, retain_result=True) == 2
    session.add_hypothesis("Maybe \x1b[31m\nconcealed", [2])
    assert "User hypothesis" in session.report()
    assert "\x1b" not in session.report()
    default = session.to_json()
    assert "Maybe" not in default
    assert json.loads(default)["session"]["runs"][1]["retained"] is None
    selected = session.to_json(include_results=[2], include_hypotheses=True)
    assert (
        json.loads(selected)["session"]["runs"][1]["retained"]["data"]
        == "00ff70726976617465"
    )
    loaded = cube.InvestigationSession.from_json(default)
    assert "Succeeded" in loaded.replay(data).report()
    with pytest.raises(ValueError, match="identity changed"):
        loaded.replay(b"\x01\xffprivate")
    session.discard_result(2)
    with pytest.raises(ValueError, match="not retained"):
        session.to_json(include_results=[2])
    with pytest.raises(ValueError):
        session.add_hypothesis("bad", [999])


def test_native_session_redactions_and_failures_are_explicit():
    import json

    data = b"hello"
    session = cube.InvestigationSession(data, identify=True)
    keyed = [
        {
            "operation": "binary.xor",
            "version": 1,
            "parameters": {"key": {"base64": "aw=="}},
        }
    ]
    session.execute_pipeline(data, keyed)
    record = session.to_json()
    assert "redacted" in record
    with pytest.raises(ValueError, match="redacted"):
        session.replay(data)
    with pytest.raises(ValueError):
        session.to_json(include_parameters=[2])
    explicit = cube.InvestigationSession(data, identify=True)
    explicit.execute_pipeline(data, keyed, record_parameters=True)
    saved = explicit.to_json(include_parameters=[2])
    assert cube.InvestigationSession.from_json(saved).replay(data)
    explicit.redact_parameters(2)
    with pytest.raises(ValueError):
        explicit.replay(data)
    session = cube.InvestigationSession(b"zz")
    session.run(b"zz", '{"kind":"triage","max_read_bytes":12}')
    session.execute_pipeline(
        b"zz",
        [
            {"operation": "text.utf8.decode", "version": 1},
            {"operation": "encoding.hex.decode", "version": 1},
        ],
    )
    value = json.loads(session.to_json())
    assert value["status"] == "partial"
    assert value["session"]["runs"][1]["state"] == "failed"
    with pytest.raises(ValueError):
        cube.InvestigationSession.from_json('{"schema_version":1,"schema_version":1}')
