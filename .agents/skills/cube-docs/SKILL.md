---
name: cube-docs
description: Write or review Cube user and contributor documentation, including documentation affected by operation or interface changes.
---

# Cube documentation

Read applicable instructions and the
[shared authoring guide](https://github.com/cubearc/docs/blob/main/content/contributing/documentation.md).
With a sibling docs checkout, prefer its `content/contributing/documentation.md`.

Find the existing canonical page for the reader task. Keep operation semantics in
reference, interface-specific behavior in its interface page, and working examples
in task guides. Update the README only for positioning or first use.

Choose one main interface for a task. Keep Python integration in its SDK guide;
add a second interface example only when it solves a different need. Explain what
the reader does and what the result means. Do not turn guides into test assertions
or repeat contributor process instructions in product pages.

For behavior changes, inspect the implementation and affected examples. Regenerate
contract reference from its owning Rust definitions when required. Update
`docs/SUMMARY.md`, preserve old links and run relevant example/link/reference checks.

Respect review-only and planning-only requests. Report the pages and validation
actually changed or performed.
