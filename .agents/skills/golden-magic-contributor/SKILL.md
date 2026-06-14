---
name: golden-magic-contributor
description: Use when implementing Golden Magic parser heuristics, descriptors, Nushell adapters, fixture harnesses, or performance gates in this repository.
---

# Golden Magic Contributor

## Ground Rules

- Keep parser/domain logic independent from Nushell plugin APIs.
- Prefer explicit trace output over hidden debug channels.
- Add or update tests for every heuristic, descriptor, adapter, or config change.
- Use stable rule ids such as `detect.delimited.tabs`; reject unknown rule ids.
- Descriptor changes need isolated fixture coverage and full-registry conflict coverage.
- Native dynamic loading is not allowed without a security/portability design review.

## Verification

Run before claiming work is complete:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

For optional real-tool isolation:

```bash
GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 cargo test --test nix_fixture -- --nocapture
```

For performance baseline checks:

```bash
cargo bench --bench parser -- --sample-size 10
```

## Architecture Pointers

- Parser core: `src/lib.rs`
- CLI adapter: `src/main.rs`
- Native Nu plugin: `src/bin/nu_plugin_golden_magic.rs`
- Nu wrapper: `nu/golden-magic.nu`
- Descriptors: `src/descriptors.rs`, `docs/DESCRIPTORS.md`
- Debug policy: `docs/DEBUG-INSTRUMENTATION.md`
- Perf policy: `docs/PERFORMANCE.md`
- Nix fixtures: `docs/NIX-FIXTURES.md`
