# Bounded puzzle analysis

Analyze a supplied dictionary, probability model or spatial input.
The [puzzle model reference](../../puzzle-analysis.md) defines the contracts,
budgets and examples.

## Supported analysis

| Family | Method | Interpretation |
| --- | --- | --- |
| Anagrams and word ladders | Scalar inventories and breadth-first graph search | Uses the supplied dictionary; word ladders allow substitutions only |
| Finite probability and dice | Exact reduced fractions | Retains the supplied assumptions |
| Spatial lag | Moments of physical samples | Makes no recognition claim |

All seven operations are direct APIs without pipeline adapters.
