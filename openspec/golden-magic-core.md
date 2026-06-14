# Golden Magic Core Spec

## Purpose

Build a generic, Nushell-friendly parser engine that turns hostile table-ish CLI text into structured data without relying on upstream JSON.

## Beads Mapping

- Completed core slice: `golden-magic-hog`
- Native Nushell plugin follow-up: `golden-magic-6eh`
- Descriptor fixture harness follow-up: `golden-magic-4db`
- Nix fixture isolation follow-up: `golden-magic-jp7`
- Performance gates follow-up: `golden-magic-bz6`
- Debug instrumentation design follow-up: `golden-magic-f4n`

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
- [x] Support disabling specific heuristic rules through the CLI.
- [x] Support running only specific heuristic rules through the CLI.
- [x] Reject unknown rule ids instead of silently ignoring typos.
- [x] Include unit tests for current heuristics.
- [x] Include property-based tests for rectangular table invariants.
- [x] Include CLI integration tests for stdin parsing, output modes, header modes, rule listing, and invalid rule rejection.
- [x] Include a Nushell wrapper integration test.
- [x] Include a Criterion performance benchmark harness with initial parser baselines.
- [x] Include a generic TOML descriptor registry substrate with duplicate-id conflict tests.
- [x] Wire descriptor registry selection into CLI parser options with validation for descriptor rule ids.
- [x] Load default descriptors from XDG config with an opt-out for hermetic runs.
- [x] Support config-file descriptor directory overrides.
- [x] Keep parser core independent from Nushell plugin APIs.

## Deferred Criteria

- [ ] Implement native `from golden-magic` as a Nushell plugin binary instead of a CLI-backed wrapper.
- [ ] Add isolated Nix-backed fixture/runtime harness.
- [ ] Promote performance baselines into hard documented budgets/gates.
- [ ] Add descriptor isolation tests and full-registry conflict tests.
- [ ] Add hidden/debug instrumentation design only after security review.

## Evidence

Current verification commands:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench --bench parser -- --sample-size 10
```
