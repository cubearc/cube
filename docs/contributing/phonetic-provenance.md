# Phonetic profile provenance

Cube uses pinned, offline spelling rules. The published operation schemas define
the supported alphabets, profiles and error types. See the
[phonetic reference](../reference/topics/phonetic-text-profiles.md) for requests.

## Pinned implementation sources

| Profile | Provenance |
| --- | --- |
| Census Soundex | [National Archives rule description](https://www.archives.gov/research/census/soundex) |
| NYSIIS, Metaphone, Double Metaphone and Caverphone | [rphonetic 4.0.0](https://docs.rs/crate/rphonetic/4.0.0), a Rust port of Apache Commons Codec 1.15, with explicit Cube input admission |
| Cologne and Match Rating | [Apache Commons Codec 1.15](https://github.com/apache/commons-codec/tree/rel/commons-codec-1.15), with the declared ASCII and German-letter profiles |
| Daitch–Mokotoff and Beider–Morse | Apache Commons Codec 1.15 rule corpus under Apache-2.0, with distribution notices included |

The rphonetic package identifies commit
`f5dd1e8ce22ac211df557f7b41a51e5df02dbb57`, declares Rust 1.80 and uses portable
Rust dependencies. It was released September 2026. Compare its output with the
Apache 1.15 JAR before changing a pin. The Cologne comparison identified a
difference in suppressed-code state, so Cube applies the Apache one-pass
semantics in Core. Match Rating's Cube ASCII profile collapses a complete
consonant run and treats equal, nonempty one-letter names as matches.

## Beider–Morse integration decision

The [research issue](https://github.com/cubearc/cube/issues/320) accepts the
bounded profile tracked in [the implementation issue](https://github.com/cubearc/cube/issues/531).

Cube executes all three Apache name types and both final-rule families in Core.
The full selected tables include named languages, language inference and the
common final passes. The Rust candidate's bundled Beider–Morse data covers only
`any` and `common`. Its alternative cap does not provide Cube's rule-state and
end-to-end storage limits. Cube therefore imports the Apache data and uses one
bounded rewrite engine for Rust, CLI, native Python, Wasm and Play.

Reproduce the bundled corpus with
`python3 crates/cube-core/data/commons-codec-1.15/import-bm.py <source-jar> <output>`.
The importer verifies the Apache source SHA-256 and resolves includes by
first-character bucket. The source contains 127 rule/language resources totaling
230,286 bytes. The resulting 132,134-byte corpus has 114 reachable tables,
4,565 effective ordered rules and 497 language hints. Its SHA-256 is
`834d0ae444e2aea76b808c242d7541ff1585c1fc29ae2ba47cd758974ca9a4a3`.
Original notices credit Alexander Beider and Stephen P. Morse. Conformance inputs
are synthetic or documented standard examples.

The fixed language grammar uses literals, anchors and finite character sets.
No user-supplied rule compiler runs. Admission validates the corpus byte count
and bounded row counts. Each rule probe, language hint transition, branch
candidate and code copy is metered. Two rewrite phases may coexist temporarily.
Each frontier reserves at most 2 MiB for its live strings and ordered index.
The corpus, frontiers, partial groups and capped encoding fit the 32 MiB scratch
ceiling. The engine returns a typed resource error at its first exceeded limit,
including a live or aggregate alternative cap.

The profile evaluates an explicit language set throughout every branch. Generic
prefixes use a fixed order, and unmapped input scalars produce typed errors.
These choices keep a language selection and an input error reproducible across
hosts. A fresh language inference pass runs for each subname in guess mode.

A deterministic stress sweep used 12 repeated seeds at lengths 4 through 256,
all name types and both rule families with the unrestricted language profile.
Of 504 cases, 272 completed and 232 hit a typed resource limit. The largest
observed successful work count was 928,893, and the highest successful frontier
held 2,592 alternatives. The two maxima came from different requests. The
configured limits are 1,048,576 work units and 4,096 alternatives.
