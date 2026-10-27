# Record and replay an investigation

Choose a local input and a bounded inspection profile. Suggestions remain observations; recording and replay require explicit actions and available input evidence.

## CLI

### Record an investigation

```sh
cube investigate triage sample.bin --json
cube investigate plan sample.bin --depth preview --json
cube investigate run sample.bin --depth preview --identify --save session.json --json
cube investigate report session.json
cube investigate replay session.json sample.bin --json
```


## Python

### Keep an investigation record

```python
import json
import cube

profile = json.dumps({"format_version": 1, "depth": "preview", "identify": True})
data = b"\x00original synthetic text\x00\xff"
plan = cube.plan_investigation(len(data), profile)
result = cube.investigate(data, profile)
session = result["session"]
record = session.to_json()
assert cube.InvestigationSession.from_json(record).replay(data)
```


```python
import cube

session = cube.InvestigationSession(b"\x00\xff", identify=True)
run_id = session.execute_pipeline(
    b"\x00\xff", [{"operation": "encoding.hex.encode", "version": 1}],
    retain_result=True,
)
session.add_hypothesis("A candidate worth inspecting")
selected = session.to_json(include_results=[run_id], include_hypotheses=True)
session.discard_result(run_id)
```


## Reference

- [Investigation profiles and records](../reference/topics/investigation-profiles-and-records.md)
