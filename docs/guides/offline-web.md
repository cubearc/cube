# Inspect saved HTML and HAR

Inspect saved artifacts without opening referenced resources. For a first input, save the literal HTML below in a scratch directory; use caller-selected assets for subsequent transformations.

```sh
printf '<a href="assets/challenge.png">Saved link</a>' > saved.html
cube web html saved.html --json
```

## CLI

### Inspect saved HTML and HAR

```sh
python - <<'PYTHON'
import json
from pathlib import Path
Path("saved.html").write_text("<a href='../not-opened'>Link</a><script>fetch('https://invalid.example')</script>")
artifact = {"log": {"version": "1.2", "entries": [{"request": {"method": "GET", "url": "https://invalid.example/asset"}, "response": {"status": 200, "content": {"mimeType": "application/octet-stream", "text": "AP8=", "encoding": "base64", "size": 2}}}]}}
Path("saved.har").write_text(json.dumps(artifact))
PYTHON
cube web html saved.html --json
cube web har saved.har --json
```


```sh
cube web asset har saved.har --index 0 --json
cube web asset har saved.har --index 0 --output selected.bin
cube web asset har saved.har --index 0 --stdout | cube pipeline run --steps '[{"operation":"encoding.hex.encode","version":1}]' --format raw
# 00ff (without an added newline)
```


## Python

### Inspect offline HTML and HAR

```python
import json
import cube

html = cube.inspect_html("<a href='../not-opened'>A &amp; B</a><script>inert()</script>")
assert html["references"][0]["value"] == "../not-opened"
assert html["text"] == "A & B"
artifact = {"log": {"version": "1.2", "entries": [{"request": {"method": "GET", "url": "https://invalid.example/asset"}, "response": {"status": 200}}]}}
har = cube.inspect_har(json.dumps(artifact))
assert har["entries"][0]["status"] == 200
assert har["entries"][0]["body_text_bytes"] is None
```


```python
import cube

document = "<img src='data:application/octet-stream;base64,AP8='>"
asset = cube.extract_html_asset(document, 0, selector="data_url")
assert asset["data"] == b"\x00\xff"
assert asset["provenance"]["bytes"] == 2
steps = [{"operation": "encoding.hex.encode", "version": 1}]
assert cube.execute_pipeline(steps, asset["data"])["output"] == "00ff"

steps.insert(0, {"operation": "web.html.asset", "version": 1,
                 "parameters": {"index": 0, "selector": "data_url"}})
assert cube.execute_pipeline(steps, document, capture=[0])["intermediates"][0] == b"\x00\xff"
```


## Reference

- [Offline web artifacts](../reference/topics/offline-web-artifacts.md)
