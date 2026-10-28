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

Before expanding a dense passage, inspect its family reference, linked guides
and shared concepts. Classify the facts: reusable explanation belongs in Concepts;
exact defaults, ceilings, preparation choices, accounting rules and failure policy
belong with the operation; allocator layouts, fixture methods and design history
belong under Contributing. Reuse the canonical explanation with a descriptive link
instead of expanding the same paragraph on every page. Keep a short caveat beside
the action when it changes the reader's choice. Preserve generated contracts and
compatibility pointer pages; their repeated structure is intentional.

Use `docs/concepts/text.md`, `docs/concepts/resource-limits.md` and
`docs/concepts/results.md` for shared terminology, and
`docs/reference/text-preparation.md` only for operations using that exact profile.
Do not infer a global limit or preparation policy. Distinguish per-input from
aggregate limits, scratch from combined/process memory, defaults from hard maxima,
and work units from time. Preserve whether exhaustion fails, truncates only a
listing, stops execution or returns a search prefix; completion fields have
different meanings. Keep numeric units and zero/empty cases exact.

Rewrite mixed passages by subject and use short tables for lookup. Move long
algorithm descriptions out of table cells. Do not merely add headings or turn
prose into a checklist. Remove release-relative wording and hardcoded catalog or
adapter totals from user explanations. Check that a reader can follow links to
the actual limit or rule, rather than another broad index.

For a broad cleanup, inspect related pages for conceptual repetition as well as
copied text. Readability metrics and text matching can locate candidates; they
do not establish quality or justify deleting operation-specific facts. Review
changed references at desktop and mobile widths, and verify values, defaults,
result meanings and moved anchors against the original and implementation.

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
