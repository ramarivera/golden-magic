# Golden Magic Completion Audit

Date: 2026-06-15

This audit validates the previously claimed completion list against current repository state. A claim is only marked proven when source files and fresh command output directly cover it.

## Proven Claims

| Claim | Evidence |
| --- | --- |
| Rust parser core | `src/lib.rs` exposes `parse`, `parse_with_options`, `ParseReport`, `ParseOptions`; `cargo test --lib -- --nocapture` passed 13 tests. |
| Rust CLI binary `golden-magic` | `Cargo.toml` package/bin defaults and `src/main.rs`; `cargo build --bins` passed; manual stdin smoke through `target/debug/golden-magic` emitted rows JSON. |
| CLI aliases `gold`, `golden`, `magic`, `magia` | `src/bin/*.rs`; `cargo test --test cli -- --nocapture` passed 13 tests including alias coverage. |
| Native Rust Nushell plugin binary `nu_plugin_golden_magic` | `Cargo.toml` feature-gated bin and `src/bin/nu_plugin_golden_magic.rs`; `cargo test --features nu-plugin --test nu_plugin -- --nocapture` passed 3 tests. |
| Nushell wrapper module | `nu/golden-magic.nu`; `cargo test --test nu_wrapper -- --nocapture` passed 1 integration test. |
| `from golden-magic` aliases in wrapper and native plugin paths | Wrapper exports aliases in `nu/golden-magic.nu`; plugin command list in `src/bin/nu_plugin_golden_magic.rs`; Nu wrapper and native plugin integration tests passed. |
| Heuristic parsing: tabs, pipes, commas, semicolons, repeated-space tables, fallback lines | Rule constants and parser implementations are in `src/lib.rs`; `cargo test --lib -- --nocapture` passed 13 tests; `target/debug/golden-magic --list-rules` listed all expected rule ids. |
| Rule toggles: `--disable-rule`, `--only-rule`, `--list-rules` | `src/cli.rs` implements all three; `cargo test --test cli -- --nocapture` passed; manual `--only-rule detect.delimited.pipes --output trace-json` showed `options.only-rule` and pipe detection. |
| Output modes: report JSON, rows JSON, trace JSON | `src/cli.rs` implements `report-json`, `rows-json`, `trace-json`, and `--explain`; CLI tests passed. |
| Descriptor registry substrate | `src/descriptors.rs`; `cargo test --test descriptor_fixtures -- --nocapture` passed 3 tests. |
| XDG/config descriptor directory loading in CLI path | `src/cli.rs` and `src/config.rs`; CLI tests for XDG default and config override passed. |
| Descriptor/config loading inside native Nu plugin path | `src/bin/nu_plugin_golden_magic.rs` reuses `parser_options_from_descriptors`; native plugin integration tests passed. |
| Descriptor fixture tests | `tests/descriptor_fixtures.rs`; `cargo test --test descriptor_fixtures -- --nocapture` passed. |
| Property tests | `src/lib.rs` includes `proptest!`; `cargo test --lib -- --nocapture` passed. |
| CLI tests | `tests/cli.rs`; `cargo test --test cli -- --nocapture` passed 13 tests. |
| Nu wrapper tests | `tests/nu_wrapper.rs`; `cargo test --test nu_wrapper -- --nocapture` passed. |
| Native Nu plugin integration test | `tests/nu_plugin.rs`; `cargo test --features nu-plugin --test nu_plugin -- --nocapture` passed. |
| Optional Nix-backed fixture test exists | `tests/nix_fixture.rs` exists and `cargo test --test nix_fixture -- --nocapture` passed the default skip path. |
| Descriptor-driven Nix manifest harness exists | `tests/nix_fixture.rs` reads `nix.toml` manifests; `tests/fixtures/descriptors/generic-pipes/nix.toml` exists; default skip test passed; opt-in live execution passed in a disposable `nixos/nix:latest` container with `GOLDEN_MAGIC_RUN_NIX_FIXTURES=1`. |
| Known-tool descriptor corpus | `tests/fixtures/descriptors/*` contains representative descriptor packs; `docs/KNOWN-TOOLS.md`; descriptor fixture tests passed. |
| Extension-author SDK for descriptor packs | `docs/SDK.md`, `schemas/descriptor.schema.json`, `examples/descriptors/simple-pipes/*`, `--validate-descriptor-dir`; CLI tests and manual validation passed. |
| Prior-art research artifact | `docs/PRIOR-ART.md` exists. It still needs tree-sitter/backend expansion for the remaining parser-backend work. |
| Safe native runtime extension stance | `docs/EXTENSIONS.md` explicitly rejects arbitrary native runtime loading until separate review. |
| Docs, OpenSpec, beads, AGENTS/CLAUDE setup | `docs/*`, `openspec/golden-magic-core.md`, `.beads/issues.jsonl`, `AGENTS.md`, `CLAUDE.md` exist. |
| Public GitHub repo metadata | `git remote -v` points to `git@github.com:ramarivera/golden-magic.git`; `Cargo.toml` contains repository/homepage metadata. |
| crates.io metadata fixed | `Cargo.toml` contains description, license, repository, homepage, readme, keywords, and categories. |

## Contradicted Or Incomplete Claims

| Claim | Status |
| --- | --- |
| Arbitrary Rust runtime extension/plugin loading | Not implemented by design. Current docs reject it until separate security and portability review. |
| Tree-sitter backend | Not implemented. `docs/PARSER-BACKENDS.md` defers it until a named CLI grammar target and dependency approval justify adding tree-sitter runtime and grammar packages. |
| Full live Nix fixture execution | Proven through Docker-backed `nixos/nix:latest` run because host `nix` is not available on `PATH`; bead `golden-magic-714` can close with that evidence. |

## Weak Or Qualified Evidence

| Claim | Qualification |
| --- | --- |
| Performance benchmark + hard perf gate | The hard perf gate passed with `cargo test --test performance_gate -- --nocapture`. Criterion exits successfully, but `cargo bench --bench parser -- --sample-size 10` reported a small medium-TSV regression against untracked local `target/criterion` baseline data while large TSV and first-row headers improved. Treat Criterion comparison output as advisory until a checked-in baseline policy exists. |

## Audit Fixes Applied

- Descriptor loading now ignores reserved `nix.toml` fixture manifests, so a descriptor directory can contain optional Nix fixture metadata without breaking `--descriptor-dir` or `--validate-descriptor-dir`.
- Added a unit test for ignoring `nix.toml` in descriptor directories.
- Criterion regression was investigated and recorded in [`docs/PERFORMANCE.md`](PERFORMANCE.md). No baseline was updated.
- Descriptor parser backends now have explicit validation, `--list-backends` discovery, and core `ParseOptions` backend selection. `heuristic` and `sections` are implemented; `tree-sitter` remains a planned candidate and fails validation until implemented.
- Descriptor-driven Nix manifest fixtures were verified in a disposable `nixos/nix:latest` container with the repository mounted read-only and Cargo outputs redirected to `/tmp`.
