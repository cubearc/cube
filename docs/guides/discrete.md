# Count combinations and analyze simple models

Use a character inventory, formula or graph to ask a specific question.

## Check missing characters

```sh
cube text multiset subtract --text ab --requested aab
```

The requested text needs two copies of `a`, but the input has one.
The report identifies a deficit of one `a`.

## Count arrangements

```sh
cube number multinomial --groups 2,1,1 --json
cube number pascal --rows 4
```

The multinomial result is 12: the number of arrangements of four items when two
are identical. The second command returns the first four rows of Pascal's triangle.

## Evaluate a formula or graph

```sh
cube logic boolean analyze --text '{"variables":[],"formula":{"op":"const","value":true}}'
cube graph euler-trail --text '{"vertices":["a"],"directed":false,"edges":[]}'
```

The formula is true for its single assignment. The graph has one vertex and no
edges, so it has an empty Euler trail.

For larger inputs, start with the [model definitions](../discrete-foundations.md)
and [limits](../reference/topics/lexical-and-discrete-analysis.md).

<a id="analyze-inventories-logic-and-graphs"></a>
<a id="cli"></a>
<a id="lexical-and-discrete-foundations"></a>
<a id="lexical-and-discrete-investigations"></a>
<a id="python"></a>
<a id="reference"></a>
