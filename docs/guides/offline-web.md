# Inspect saved HTML and HAR

Read links and response metadata from files saved on your computer.
Cube does not run page scripts or fetch referenced URLs.

## Read links from HTML

```sh
printf '<a href="assets/challenge.png">Saved link</a>' > saved.html
cube web html saved.html --json
```

The report includes the link `assets/challenge.png` and the text `Saved link`.
The linked file is not opened.

## Extract a response from a HAR file

This small HAR contains one response body: the two bytes `00 ff`, encoded as Base64.
Create it with Python's standard library:

```sh
python3 - <<'PYTHON'
import json
from pathlib import Path

artifact = {
    "log": {
        "version": "1.2",
        "entries": [{
            "request": {"method": "GET", "url": "https://invalid.example/asset"},
            "response": {
                "status": 200,
                "content": {
                    "mimeType": "application/octet-stream",
                    "text": "AP8=",
                    "encoding": "base64",
                    "size": 2,
                },
            },
        }],
    },
}
Path("saved.har").write_text(json.dumps(artifact), encoding="utf-8")
PYTHON
cube web har saved.har --json
cube web asset har saved.har --index 0 --output selected.bin
cube file hex selected.bin
```

Entry 0 returns status 200. Extraction writes a new file containing `00 ff`;
it does not make the recorded HTTP request.

See [offline web formats and limits](../reference/topics/offline-web-artifacts.md)
for data URLs, missing bodies and extraction rules.

<a id="cli"></a>
<a id="inspect-offline-html-and-har"></a>
<a id="inspect-saved-html-and-har-1"></a>
<a id="python"></a>
<a id="reference"></a>
