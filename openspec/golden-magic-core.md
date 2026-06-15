# Golden Magic Core Spec

## Purpose

Build a generic, Nushell-friendly parser engine that turns hostile table-ish CLI text into structured data without relying on upstream JSON.

## Beads Mapping

- Completed core slice: `golden-magic-hog`
- Completed native Nushell plugin: `golden-magic-6eh`
- Completed descriptor fixture harness: `golden-magic-4db`
- Completed Nix fixture isolation: `golden-magic-jp7`
- Completed performance gates: `golden-magic-bz6`
- Completed debug instrumentation design: `golden-magic-f4n`
- Completed native plugin descriptor/config parity: `golden-magic-9e1`
- Completed descriptor-driven Nix manifest harness: `golden-magic-714`
- Completed prior-art research artifact: `golden-magic-4re`
- Completed safe runtime extension architecture design: `golden-magic-by0`
- Completed known-tool descriptor corpus for current descriptor schema: `golden-magic-9pu`
- Completed extension-author SDK for descriptor packs: `golden-magic-euf`
- Completed parser-backend/grammar-like parsing slice: `golden-magic-2mf`

## Acceptance Criteria

- [x] Parse rectangular tab-delimited input into rows and generated columns.
- [x] Parse rectangular comma, semicolon, and pipe-delimited input into rows and generated columns.
- [x] Parse simple repeated-space fixed-width-ish input into rows and generated columns.
- [x] Fall back to one-column line records when table inference is unsafe.
- [x] Emit parser confidence and trace events with stable rule ids.
- [x] Expose heuristic rule listing through the CLI.
- [x] Support full-report, rows-only, and trace-only JSON output modes through the CLI.
- [x] Support generated and first-row header modes through the CLI and parser core.
- [x] Provide a Nushell wrapper module exporting `from golden-magic` over the CLI adapter.
- [x] Provide a native Nushell plugin binary exporting `from golden-magic` over the same parser core.
- [x] Support disabling specific heuristic rules through the CLI.
- [x] Support running only specific heuristic rules through the CLI.
- [x] Reject unknown rule ids instead of silently ignoring typos.
- [x] Include unit tests for current heuristics.
- [x] Include property-based tests for rectangular table invariants.
- [x] Include CLI integration tests for stdin parsing, output modes, header modes, rule listing, and invalid rule rejection.
- [x] Include a Nushell wrapper integration test.
- [x] Include a Criterion performance benchmark harness with initial parser baselines.
- [x] Include hard parser performance regression gates in `cargo test`.
- [x] Include a generic TOML descriptor registry substrate with duplicate-id conflict tests.
- [x] Wire descriptor registry selection into CLI parser options with validation for descriptor rule ids.
- [x] Load default descriptors from XDG config with an opt-out for hermetic runs.
- [x] Support config-file descriptor directory overrides.
- [x] Support descriptor/config loading inside the native Nushell plugin path.
- [x] Include descriptor fixture harness tests for isolated matching, negative inputs, expected rows, and duplicate registry ids.
- [x] Include optional descriptor-driven Nix manifest fixture harness and docs.
- [x] Include extension-author SDK docs, descriptor schema, validation command, example descriptor pack, and fixture guidance.
- [x] Include descriptor-selected parser backend support with `heuristic` and `sections` backends.
- [x] Include backend listing, validation, fixture coverage, malformed-input diagnostics, and property tests for current backend behavior.
- [x] Evaluate tree-sitter before inventing a custom grammar DSL and document why tree-sitter is deferred for this scope.
- [x] Document debug instrumentation threat model and explicit no-hidden-channel default.
- [x] Keep parser core independent from Nushell plugin APIs.

## Deferred Criteria

- Arbitrary Rust runtime extension/plugin loading is not implemented. `docs/EXTENSIONS.md` explicitly rejects native runtime loading until a separate security and portability review; current design prefers descriptor packs, subprocesses, and WASM/WASI boundaries.
- Tree-sitter backend is not implemented. `docs/PARSER-BACKENDS.md` defers it until a named CLI grammar target and dependency approval justify adding the runtime and generated grammar package.
- Native runtime extension SDK is not implemented. The current SDK is descriptor-pack authoring, validation, schema, examples, and fixture guidance.
- Live execution of descriptor-driven Nix manifest fixtures is not verified in environments without `nix`; the harness skips unless `GOLDEN_MAGIC_RUN_NIX_FIXTURES=1` and `nix` is available.

## Evidence

Current verification commands:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench --bench parser -- --sample-size 10
```
