import hashlib
import json

import cube
import pytest


def har_document(text="AP8=", encoding="base64"):
    return json.dumps(
        {
            "log": {
                "version": "1.2",
                "entries": [
                    {
                        "request": {
                            "method": "GET",
                            "url": "https://invalid.example/asset",
                        },
                        "response": {
                            "status": 200,
                            "content": {
                                "mimeType": "application/octet-stream",
                                "text": text,
                                "encoding": encoding,
                                "size": 2,
                            },
                        },
                    }
                ],
            }
        }
    )


def test_offline_html_references_and_inline_spans_do_not_become_actions(tmp_path):
    marker = tmp_path / "must-not-exist"
    document = (
        f"雪<script>fetch('file://{marker}');<img src=hidden></script>"
        "<a href='../secret?x=1&amp;y=2'>A &amp; B</a>"
        "<img src='https://invalid.example/x'>"
    )
    r = cube.inspect_html(document)
    assert len(r["references"]) == 2
    assert r["references"][0]["value"] == "../secret?x=1&y=2"
    assert r["text"].endswith("A & B")
    start, end = r["embedded"][0]["source_range"]
    assert document.encode()[start:end].startswith(b"fetch(")
    assert not marker.exists()
    with pytest.raises(ValueError, match="128"):
        cube.inspect_html("<div>" * 129)


def test_har_metadata_preserves_unknowns_and_rejects_duplicate_or_excessive_json():
    r = cube.inspect_har(har_document())
    assert r["entries"][0]["body_text_bytes"] == 4
    assert r["entries"][0]["body_encoding"] == "base64"
    assert r["entries"][0]["started_date_time"] is None
    assert "AP8=" not in json.dumps(r)
    for document in [
        '{"log":{"version":"1.2","version":"1.2","entries":[]}}',
        "[" * 20,
        "x" * (256 * 1024 + 1),
    ]:
        with pytest.raises(ValueError) as e:
            cube.inspect_har(document)
        assert e.value.operation == "web.har.inspect"


def test_selected_assets_keep_binary_data_provenance_and_existing_pipeline_types():
    document = "雪<script>é\r\n&amp;</script><img src='data:;base64,AP8='>"
    inline = cube.extract_html_asset(document, 0)
    assert inline["data"] == "é\r\n&amp;".encode()
    p = inline["provenance"]
    start, end = p["source_range"]
    assert document.encode()[start:end] == inline["data"]
    assert p["artifact_sha256"] == hashlib.sha256(document.encode()).hexdigest()
    assert p["sha256"] == hashlib.sha256(inline["data"]).hexdigest()
    assert (
        cube.extract_html_asset(document, 0, selector="data_url")["data"] == b"\0\xff"
    )
    selected = cube.extract_har_asset(har_document(), 0)
    assert selected["data"] == b"\0\xff"
    assert selected["provenance"]["representation"] == "har_base64_decoded_bytes"
    steps = [{"operation": "encoding.hex.encode", "version": 1}]
    assert cube.execute_pipeline(steps, selected["data"])["output"] == "00ff"
    steps.insert(
        0, {"operation": "web.har.asset", "version": 1, "parameters": {"index": 0}}
    )
    assert cube.execute_pipeline(steps, har_document(), capture=[0])[
        "intermediates"
    ] == {0: b"\0\xff"}
    steps[0] = {
        "operation": "web.html.asset",
        "version": 1,
        "parameters": {"index": 0, "selector": "data_url"},
    }
    assert cube.execute_pipeline(steps, document)["output"] == "00ff"
    assert cube.describe_operation("web.html.asset")["pipeline"] == {"version": 1}


def test_asset_selection_requires_explicit_supported_embedded_data(tmp_path):
    source = tmp_path / "secret.bin"
    source.write_bytes(b"private")
    for reference in [
        str(source),
        source.as_uri(),
        "https://invalid.example/x",
        "data:,x",
    ]:
        with pytest.raises(ValueError):
            cube.extract_html_asset(f"<img src='{reference}'>", 0, selector="data_url")
    assert source.read_bytes() == b"private"
    for document, index, selector in [
        ("<script>x", 0, "inline"),
        ("<script>x</script>", 1, "inline"),
        ("<script>x</script>", 0, "automatic"),
        ("<img src='data:;base64,AB=='>", 0, "data_url"),
    ]:
        with pytest.raises(ValueError):
            cube.extract_html_asset(document, index, selector=selector)
    with pytest.raises(ValueError):
        cube.extract_har_asset(har_document(encoding="gzip"), 0)
    artifact = json.loads(har_document("é\0", "base64"))
    content = artifact["log"]["entries"][0]["response"]["content"]
    del content["encoding"]
    selected = cube.extract_har_asset(json.dumps(artifact), 0)
    assert selected["data"] == "é\0".encode()
    assert selected["provenance"]["representation"] == "har_logical_text_utf8"
    del content["text"]
    content["_file"] = str(source)
    with pytest.raises(ValueError, match="absent"):
        cube.extract_har_asset(json.dumps(artifact), 0)
    with pytest.raises(ValueError):
        cube.validate_pipeline(
            [{"operation": "web.html.asset", "version": 1, "parameters": {"index": 0}}],
            "bytes",
        )
