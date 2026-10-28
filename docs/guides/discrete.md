# Analyze inventories, logic and graphs

Supply the exact inventory, Boolean formula or finite graph. The reference explains the accepted model and what each result establishes.

## CLI

### Lexical and discrete investigations

```sh
cube text multiset subtract --text ab --requested aab
cube number binomial --n 128 --k 64 --json
cube number multinomial --groups 2,1,1 --json
cube number pascal --rows 4
cube logic boolean analyze --text '{"variables":[],"formula":{"op":"const","value":true}}'
cube graph euler-trail --text '{"vertices":["a"],"directed":false,"edges":[]}'
```


## Python

### Lexical and discrete foundations

```python
import cube

assert cube.multiset_compare("listen", "silent")["equal"]
assert cube.multiset_subtract("ab", "aab")["deficits"] == [{"scalar": "a", "count": 1}]
assert cube.binomial(128, 64) == 23951146041928082866135587776380551750
assert cube.permutations(5, 2) == 20
assert cube.multinomial([2, 1, 1]) == 12
assert cube.pascal_triangle(3) == [[1], [1, 1], [1, 2, 1]]
r = cube.boolean_analyze({"variables": [], "formula": {"op": "const", "value": True}})
assert r["tautology"] and r["assignments"] == 1
assert cube.graph_euler_trail({"vertices": [], "directed": False, "edges": []})["result"]["exists"]
```


## Reference

- [Lexical and discrete analysis](../reference/topics/lexical-and-discrete-analysis.md)

- [Detailed models and limits](../discrete-foundations.md)
