# Parser Backend Decision

Golden Magic should not treat "grammar engine" as a synonym for "build a custom grammar DSL." The parser core already handles generic table-ish CLI output with cheap heuristics. More machinery is only justified for output shapes where those heuristics cannot preserve structure.

## Sources Checked

- Tree-sitter introduction: https://tree-sitter.github.io/
- Tree-sitter parser authoring docs: https://tree-sitter.github.io/tree-sitter/creating-parsers/
- Tree-sitter grammar DSL docs: https://tree-sitter.github.io/tree-sitter/creating-parsers/2-the-grammar-dsl.html
- Rust `tree-sitter` API docs: https://docs.rs/tree-sitter
- Existing Golden Magic prior art: [`docs/PRIOR-ART.md`](PRIOR-ART.md)

## Current Decision

Use tree-sitter as the first serious candidate for grammar-like parser backends, but do not replace the current table heuristics with tree-sitter.

The implemented slice is a narrow backend experiment, not a general grammar engine:

1. Backend abstraction returns rows, trace events, confidence, and stable rule ids.
2. Descriptors opt into a backend by id.
3. The `sections` backend prototypes one structured output family whose shape is awkward for delimiter or repeated-space parsing.
4. Fallback heuristics remain the default path for ordinary CLI tables.

Status: the core has an explicit parser backend selection path through `ParseOptions`, and descriptors can select the implemented `heuristic` and `sections` backends. Tree-sitter is evaluated below and deferred for this scope.

## Why Tree-sitter Fits Some Cases

Tree-sitter is built around generated parsers that return concrete syntax trees. That is useful when Golden Magic needs to preserve nested structure, repeated sections, subcommands, or language-like output where splitting rows loses meaning.

The Rust API exposes a `Parser`, generated `Language` objects, syntax trees, nodes, and queries. That maps cleanly to an adapter layer where a backend parses input, walks selected nodes, and emits rows plus trace events.

Tree-sitter grammars also have a mature test/publishing workflow. That is better than inventing a private grammar format unless Golden Magic has requirements tree-sitter cannot satisfy.

## Why Tree-sitter Should Not Become The Default Parser

Most first-class Golden Magic inputs are hostile-but-table-ish CLI text. For tabs, pipes, commas, semicolons, repeated spacing, and fallback lines, the current heuristics are simpler, dependency-light, easier to explain, and easier to tune with property tests.

Tree-sitter requires generated language artifacts. That adds build tooling, grammar package selection, dependency policy, and distribution questions. It is overkill for rectangular text.

Tree-sitter is also strongest when there is a real grammar. Many CLI outputs are not languages; they are loosely aligned text with occasional headers. For those, descriptors plus existing heuristics are still the right center.

## Tree-sitter Scope Decision

Tree-sitter is rejected for the current `golden-magic-2mf` implementation slice and kept as a future backend candidate.

The evidence:

- Official tree-sitter docs frame it as a parser generator plus incremental parsing library that produces concrete syntax trees for source files. Golden Magic's dominant inputs are one-shot CLI output streams, not editable source files.
- Rust tree-sitter usage requires the `tree-sitter` crate plus a language grammar crate such as `tree-sitter-rust`; that is at least one new parser dependency and one grammar dependency per supported grammar.
- Tree-sitter parser authoring uses generated grammars from `grammar.js`. The authoring docs call out a real grammar development workflow, including grammar DSL, parser writing, tests, and publishing. That is heavier than the current descriptor-pack SDK.
- The current repo now has a real non-table backend, `sections`, with stable trace IDs, fixture coverage, property coverage, malformed-input diagnostics, and descriptor integration. That satisfies the immediate need to prove backend routing without taking on tree-sitter packaging yet.

Tree-sitter should be reconsidered when there is a named CLI output family with an actual grammar, fixtures that cannot be handled by `heuristic` or `sections`, and explicit approval to add both the tree-sitter runtime crate and a generated grammar package.

## Rejected Default

Do not start by building a custom Golden Magic grammar DSL.

A custom DSL would create parser semantics, grammar authoring docs, diagnostics, compatibility rules, and tooling from scratch. That may become necessary later, but it is not the first move while tree-sitter and parser-combinator approaches remain viable.

## Candidate Backend Contract

A backend should be selected by descriptors, not by hidden global behavior.

```toml
id = "example.structured"
name = "Example Structured Output"

[matches]
required_substrings = ["section:"]

[parser]
backend = "tree-sitter"
grammar = "golden-magic-example"
query = "rows.scm"
```

This TOML shape is a design target, not implemented API yet.

The implemented descriptor surface currently accepts:

```toml
[parser]
backend = "heuristic"
```

and:

```toml
[parser]
backend = "sections"
```

Descriptors that request `tree-sitter` fail validation until a future tree-sitter backend exists.

Direct core use can also select the implemented backend:

```rust
ParseOptions::new().backend("heuristic")
ParseOptions::new().backend("sections")
```

Inspect implemented backend ids with:

```bash
golden-magic --list-backends
```

Backend results must include:

- stable rule id, such as `backend.tree-sitter.example`
- rows as native parser records
- confidence
- trace event explaining descriptor selection and backend selection
- diagnostics when parsing fails or produces errors

## Prototype Acceptance Criteria

This slice proves:

- a descriptor can select a backend explicitly
- tree-sitter support is rejected for this scope with evidence and future-entry criteria
- backend ids are listed or discoverable
- backend failures fall back safely or report clear diagnostics
- fixtures cover positive input, negative input, malformed input, and expected rows
- property tests cover backend invariants that matter for row emission
- parser core remains independent from Nushell plugin APIs
- dependency/build implications are documented

## Open Dependency Gate

Adding the `tree-sitter` crate or a generated grammar crate is a new dependency. That requires explicit approval before code changes add it.
