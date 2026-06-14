# Nix-backed Fixture Isolation

Golden Magic can test real third-party CLI behavior without installing those tools system-wide by running optional fixtures through `nix shell`.

This is intentionally opt-in. Normal `cargo test` should stay fast and should not require Nix, network access, or nixpkgs cache state.

## Run Optional Nix Fixtures

```bash
GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 cargo test --test nix_fixture -- --nocapture
```

If the environment variable is absent, the test skips itself. If `nix` is not on `PATH`, the test also skips itself with a diagnostic.

## Pattern

A Nix-backed fixture should:

1. build `golden-magic` through Cargo as usual
2. launch `nix shell nixpkgs#<tool> --command ...`
3. pipe deterministic fixture output into `golden-magic`
4. assert JSON/structured rows, not terminal substrings
5. avoid mutating global package state
6. skip by default unless `GOLDEN_MAGIC_RUN_NIX_FIXTURES=1` is set

## Why Opt-in

Nix can be slow on cold caches and can require network access. Hard-requiring it in every local test run would make parser development annoying as hell. The compromise is:

- core/unit/property/CLI tests run always
- optional real-tool fixtures run when explicitly requested
- CI can choose a Nix-enabled job later

## Future Extension Harness

For vendored tool descriptors, prefer a fixture layout like:

```text
fixtures/<tool>/<case>/
  descriptor.toml
  nix.toml
  input.command
  expected.rows.json
```

A future harness can read `nix.toml`, enter a short-lived Nix shell for the listed packages, run `input.command`, and assert `expected.rows.json` against `golden-magic --output rows-json`.

This doc is the policy floor; implementation details can evolve as descriptor fixtures get richer.
