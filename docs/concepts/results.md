# Complete and partial results

Before interpreting a count, match or missing path, check what the operation
finished. Reaching a resource limit can mean an error, omitted rows or an
interrupted computation, depending on the operation.

## Complete answer with a shorter listing

Some operations compute the whole answer but retain only selected rows.
For example, Unicode inspection keeps exact counts when its record listing is
truncated. Boolean analysis counts every assignment even when it lists few rows.

Read the listing-completeness field together with the totals. Omitting rows does
not make those totals estimates. The operation reference names the field and
defines the ordering and listing budget.

## Interrupted search or execution

A search can return the prefix it actually explored. Anagram search identifies
work or result-limit stops; classical-analysis reports can also identify
cancellation or pruning. No solution in an incomplete search does not establish
that no solution exists.

Brainfuck report APIs can retain partial execution evidence. Their `completed`
flag describes execution; `trace_complete` separately describes the diagnostic
trace. Execution can finish even after the trace fills up.

## Failure without a result

Other operations require a complete answer. Multiset comparison and graph queries
fail when processing cannot stay within their budgets. A resource error is not
evidence of unequal text, a disconnected graph or an impossible Euler trail.

Preprocessing can fail even for an operation that supports interrupted execution.
For example, anagram preparation raises an error before search, while a search
that reaches its work limit can return an incomplete prefix.

## Error categories and interpretation

Typed Core errors distinguish `invalid_input`, `invalid_parameter`,
`unsupported_format`, `resource_limit`, `cancelled` and `io`. The operation may
attach its ID, a parameter or an original input offset. Each interface specifies
how it exposes these errors, and each operation specifies which cases apply.

Completeness fields qualify their own report. `complete`, `listing_complete`,
`completed` and `trace_complete` are not interchangeable. A complete measurement
also does not establish an interpretation beyond that operation: a language score
is a heuristic, and a character-property flag is not a security verdict.
