# Golden Magic

Golden Magic is an experimental parser for turning hostile, table-ish CLI text into structured data for Nushell-oriented workflows without requiring upstream JSON.

It currently provides a Rust core, CLI binaries, a Nushell wrapper module, and an optional native Nushell plugin binary:

```bash
printf 'alpha\tbeta\ngamma\tdelta\n' | golden-magic
# aliases also work: gold, golden, magic, magia
```

```nu
use ./nu/golden-magic.nu *
"name\tstatus\nalpha\tok\n" | from golden-magic --headers first-row
```

Build the optional native plugin:

```bash
cargo build --features nu-plugin --bin nu_plugin_golden_magic
```

Native plugin smoke path:

```nu
plugin add ./target/debug/nu_plugin_golden_magic
plugin use golden_magic
"name\tstatus\nalpha\tok\n" | from gold --headers first-row
```

Nushell command aliases work in both the wrapper and native plugin paths:

```nu
from golden-magic
from gold
from golden
from magic
from magia
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
- CLI binaries `golden-magic`, `gold`, `golden`, `magic`, and `magia`
- Nushell wrapper commands `from golden-magic`, `from gold`, `from golden`, `from magic`, and `from magia` in `nu/golden-magic.nu`
- optional native Nushell plugin binary `nu_plugin_golden_magic` exporting the same `from ...` aliases behind the `nu-plugin` Cargo feature
- descriptor registry loading with `--descriptor-dir`
- default descriptor discovery from XDG config with `--no-default-descriptors` opt-out
- config-file descriptor directory overrides via `--config` or `$XDG_CONFIG_HOME/golden-magic/config.toml`
- optional Nix-backed fixture test pattern for real CLI isolation
- Criterion benchmark harness and cargo-test parser performance gates

Not implemented yet:

- descriptor/config loading inside the native Nu plugin; use the CLI wrapper when descriptor packs are needed
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
cargo clippy --features nu-plugin -- -D warnings
cargo test --features nu-plugin
cargo bench --bench parser -- --sample-size 10
```

The parser core is intentionally independent from Nushell plugin APIs. Both the CLI wrapper and native plugin reuse the same domain logic.
