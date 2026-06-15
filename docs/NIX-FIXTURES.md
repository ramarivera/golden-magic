# Nix-backed Fixture Isolation

Golden Magic can test real third-party CLI behavior without installing those tools system-wide by running optional descriptor fixtures through `nix shell`.

This is intentionally opt-in. Normal `cargo test` should stay fast and should not require Nix, network access, or nixpkgs cache state.

## Run Optional Nix Fixtures

```bash
GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 cargo test --test nix_fixture -- --nocapture
```

If the environment variable is absent, the test skips itself. If `GOLDEN_MAGIC_RUN_NIX_FIXTURES=1` is set, `nix` must be on `PATH` and working; otherwise the test fails. This keeps normal local tests cheap while making opt-in live verification honest.

## Manifest Fixtures

Descriptor-driven Nix fixtures live beside descriptor fixtures:

```text
tests/fixtures/descriptors/<tool-or-pattern>/
  descriptor.toml
  nix.toml
  expected.rows.json
```

`nix.toml` declares the packages and deterministic command:

```toml
packages = ["nixpkgs#coreutils"]
command = "printf 'alpha|beta\\ngamma|delta\\n'"
expected_rows = "expected.rows.json"
parser_args = ["--headers", "first-row"]
```

Fields:

- `packages`: required list of `nix shell` installables.
- `command`: required shell command that emits hostile CLI text to stdout.
- `expected_rows`: optional expected rows JSON path relative to the fixture directory; defaults to `expected.rows.json`.
- `parser_args`: optional extra `golden-magic` parser arguments.

The harness automatically runs:

```text
<command> | golden-magic --no-default-descriptors --descriptor-dir <fixture> --output rows-json <parser_args...>
```

That means every manifest fixture exercises the descriptor in isolation and compares structured rows rather than terminal substrings.

## Why Opt-in

Nix can be slow on cold caches and can require network access. Hard-requiring it in every local test run would make parser development annoying as hell. The compromise is:

- core/unit/property/CLI tests run always
- optional real-tool fixtures run when explicitly requested
- opt-in real-tool fixture runs fail when `nix` is missing, rather than reporting a false pass
- CI can choose a Nix-enabled job later

## Fixture Rules

A Nix-backed fixture should:

1. declare only the packages needed for the fixture command
2. emit deterministic stdout
3. keep mutation out of global package state
4. assert JSON/structured rows, not terminal substrings
5. skip by default unless `GOLDEN_MAGIC_RUN_NIX_FIXTURES=1` is set

This is the policy floor; manifest fields can evolve as descriptor fixtures get richer.
