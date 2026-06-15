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

Status: the core has an explicit parser backend selection path through `ParseOptions`, and descriptors can select the implemented `heuristic`, `sections`, and `tree-sitter-rust` backends.

## Why Tree-sitter Fits Some Cases

Tree-sitter is built around generated parsers that return concrete syntax trees. That is useful when Golden Magic needs to preserve nested structure, repeated sections, subcommands, or language-like output where splitting rows loses meaning.

The Rust API exposes a `Parser`, generated `Language` objects, syntax trees, nodes, and queries. That maps cleanly to an adapter layer where a backend parses input, walks selected nodes, and emits rows plus trace events.

Tree-sitter grammars also have a mature test/publishing workflow. That is better than inventing a private grammar format unless Golden Magic has requirements tree-sitter cannot satisfy.

## Why Tree-sitter Should Not Become The Default Parser

Most first-class Golden Magic inputs are hostile-but-table-ish CLI text. For tabs, pipes, commas, semicolons, repeated spacing, and fallback lines, the current heuristics are simpler, dependency-light, easier to explain, and easier to tune with property tests.

Tree-sitter requires generated language artifacts. That adds build tooling, grammar package selection, dependency policy, and distribution questions. It is overkill for rectangular text.

Tree-sitter is also strongest when there is a real grammar. Many CLI outputs are not languages; they are loosely aligned text with occasional headers. For those, descriptors plus existing heuristics are still the right center.

## Tree-sitter Scope Decision

Tree-sitter is accepted as a descriptor-selected backend surface after explicit dependency approval. The first implemented grammar target is `tree-sitter-rust`, using the `tree-sitter` runtime crate and the generated `tree-sitter-rust` grammar package.

The evidence:

- Official tree-sitter docs frame it as a parser generator plus incremental parsing library that produces concrete syntax trees for source files. Golden Magic's dominant inputs are one-shot CLI output streams, not editable source files.
- Rust tree-sitter usage requires the `tree-sitter` crate plus a language grammar crate such as `tree-sitter-rust`; this dependency surface is now explicit in `Cargo.toml`.
- Tree-sitter parser authoring uses generated grammars from `grammar.js`. The authoring docs call out a real grammar development workflow, including grammar DSL, parser writing, tests, and publishing. That is heavier than the current descriptor-pack SDK.
- The current repo has two non-default backends: `sections` for section/key-value blocks and `tree-sitter-rust` for Rust syntax declaration extraction.

New tree-sitter grammar targets should still require a named input family, fixtures that cannot be handled by `heuristic` or `sections`, and explicit approval to add the generated grammar package.

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

and:

```toml
[parser]
backend = "tree-sitter-rust"
```

`tree-sitter-rust` parses Rust syntax with tree-sitter and emits one row per supported declaration with `kind`, `name`, `start_line`, and `end_line`. It currently extracts modules, structs, and functions.

Direct core use can also select the implemented backend:

```rust
ParseOptions::new().backend("heuristic")
ParseOptions::new().backend("sections")
ParseOptions::new().backend("tree-sitter-rust")
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
- tree-sitter support is implemented for one named grammar target with evidence and future-entry criteria for additional grammars
- backend ids are listed or discoverable
- backend failures fall back safely or report clear diagnostics
- fixtures cover positive input, negative input, malformed input, and expected rows
- property tests cover backend invariants that matter for row emission
- parser core remains independent from Nushell plugin APIs
- dependency/build implications are documented

## Dependency Gate

The `tree-sitter` runtime and `tree-sitter-rust` grammar dependencies were explicitly approved before implementation. Adding more generated grammar crates remains a new dependency decision.
