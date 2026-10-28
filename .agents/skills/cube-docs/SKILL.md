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

For reference, use the shared guide's reference pattern: a short overview,
descriptive sentence-case headings and concise tables for operations, defaults
and limits. Separate behavior, boundaries and result interpretation; regroup dense
prose rather than putting headings over it. Verify exact facts against code when
rewriting, preserve old anchors, and keep implementation history out of the lookup
path. Review the rendered outline and mobile tables; a reader should find a
default, limit or error rule without reading the entire page.

For behavior changes, inspect the implementation and affected examples. Regenerate
contract reference from its owning Rust definitions when required. Update
`docs/SUMMARY.md`, preserve old links and run relevant example/link/reference checks.

Include the sibling `cubearc/docs` repository in product-documentation delivery.
Follow the shared guide's portal-delivery section: adopt the validated Cube
commit in `sources.lock.json`, update affected shared pages and readable operation
navigation names, and verify the locked portal build and authorized publication.
Product Markdown changes alone do not update the published portal. Report the
reason when no portal update is required.

When a change crosses Cube and Play execution, verify the relevant Play Docker
Runner/Lab/runtime checks in Actions against the delivered Core revision. Reuse
existing jobs; ensure a Core-side integration check tests the current Core commit
against a reviewed Play revision when that boundary changes.

Respect review-only and planning-only requests. Report the pages and validation
actually changed or performed.
