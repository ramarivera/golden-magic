# Golden Magic

Golden Magic is an experimental parser for turning hostile, table-ish CLI text into structured data for Nushell-oriented workflows without requiring upstream JSON.

It currently provides a Rust core, a small CLI adapter, and a Nushell wrapper module:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic
```

```nu
use ./nu/golden-magic.nu *
"name\tstatus\nalpha\tok\n" | from golden-magic --headers first-row
```

By default, the output is a JSON `ParseReport` containing:

- `kind`: selected parser family
- `confidence`: heuristic confidence
- `columns`: generated column names
- `rows`: parsed records
- `trace`: rule ids and reasons for parser selection

For Nushell-friendly pipelines, emit rows only:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic --output rows-json
```

Emit trace only:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic --output trace-json
```

Use the first parsed row as headers:

```bash
printf 'name\tstatus\nalpha\tok\n' | golden-magic --headers first-row --output rows-json
```

Inspect available heuristic rules:

```bash
golden-magic --list-rules
```

Explain parser selection without returning parsed rows:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic --explain
```

Disable a specific heuristic:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic --disable-rule detect.delimited.tabs
```

Run only one heuristic:

```bash
printf 'name     status\nalpha    ok\n' | golden-magic --only-rule detect.fixed-width.gaps
```

Load descriptor packs:

```bash
cat output.txt | golden-magic --descriptor-dir ./descriptors
```

By default, Golden Magic also checks:

```text
$XDG_CONFIG_HOME/golden-magic/descriptors
# or ~/.config/golden-magic/descriptors when XDG_CONFIG_HOME is unset
```

Disable default descriptor discovery:

```bash
cat output.txt | golden-magic --no-default-descriptors
```

Override descriptor directories in config:

```toml
# $XDG_CONFIG_HOME/golden-magic/config.toml
descriptor_dirs = ["/path/to/descriptors"]
```

Or pass a config explicitly:

```bash
cat output.txt | golden-magic --config ./golden-magic.toml
```

## Current Scope

Implemented generic heuristics:

- rectangular tab-delimited detection
- rectangular pipe/comma/semicolon-delimited detection
- repeated-space fixed-width-ish splitting
- safe fallback to one-column lines
- rule listing with `--list-rules`
- rule toggles with `--disable-rule` and `--only-rule`
- row-only output with `--output rows-json`
- trace-only output with `--output trace-json` or `--explain`
- generated or first-row header modes with `--headers generated|first-row`
- Nushell wrapper command `from golden-magic` in `nu/golden-magic.nu`
- descriptor registry loading with `--descriptor-dir`
- default descriptor discovery from XDG config with `--no-default-descriptors` opt-out
- config-file descriptor directory overrides via `--config` or `$XDG_CONFIG_HOME/golden-magic/config.toml`
- optional Nix-backed fixture test pattern for real CLI isolation
- Criterion benchmark harness and cargo-test parser performance gates

Not implemented yet:

- Native Nushell plugin binary for `from golden-magic`
- broader extension loading beyond TOML descriptors
- full descriptor-driven Nix fixture manifest harness
- implemented hidden debug channel; current design explicitly rejects hidden channels by default

See:

- [`docs/VISION.md`](docs/VISION.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/DESCRIPTORS.md`](docs/DESCRIPTORS.md)
- [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md)
- [`docs/NIX-FIXTURES.md`](docs/NIX-FIXTURES.md)
- [`docs/DEBUG-INSTRUMENTATION.md`](docs/DEBUG-INSTRUMENTATION.md)
- [`AGENTS.md`](AGENTS.md)

## Development

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench --bench parser -- --sample-size 10
```

The parser core is intentionally independent from Nushell plugin APIs so a future Nu adapter can reuse the same domain logic.
