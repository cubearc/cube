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
