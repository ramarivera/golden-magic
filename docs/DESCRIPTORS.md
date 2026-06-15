# Golden Magic Descriptors

Descriptors are declarative pattern packs that let Golden Magic recognize specific output shapes without embedding tool-specific code in the parser core.

The descriptor system is intentionally data-first. Native runtime plugin loading is not part of the current design.

## Format

Descriptors are TOML files loaded recursively from a directory. `nix.toml` is reserved for optional fixture metadata and is ignored by descriptor loading.

```toml
id = "example.table"
name = "Example Table"
priority = 10

[matches]
required_substrings = ["NAME", "STATUS"]

[parser]
backend = "heuristic"
only_rules = ["detect.delimited.tabs"]
disable_rules = []
```

## Fields

- `id`: stable unique descriptor id. Duplicate ids are rejected across the full registry.
- `name`: human-readable descriptor name.
- `priority`: higher values are selected first when multiple descriptors match.
- `matches.required_substrings`: all listed strings must appear in the input.
- `parser.backend`: parser backend id. Only `heuristic` is implemented today.
- `parser.only_rules`: heuristic rule ids to restrict parser selection.
- `parser.disable_rules`: heuristic rule ids to disable.

## Current Behavior

Implemented:

- load `.toml` descriptors recursively from a directory
- reject duplicate descriptor ids
- sort by descending priority, then id
- select descriptors whose required substrings all match
- expose descriptor parser rule ids for validation/wiring
- validate descriptor parser backend ids

CLI integration:

```bash
golden-magic --descriptor-dir ./descriptors
```

Author validation:

```bash
golden-magic --validate-descriptor-dir ./descriptors
```

The validation command loads descriptor TOML files, rejects duplicate descriptor ids, and checks parser rule ids without reading stdin.

Default discovery:

```text
$XDG_CONFIG_HOME/golden-magic/descriptors
~/.config/golden-magic/descriptors
```

Use `--no-default-descriptors` for hermetic runs.

Config override:

```toml
# $XDG_CONFIG_HOME/golden-magic/config.toml
descriptor_dirs = ["/path/to/descriptors"]
```

When `descriptor_dirs` is present in config, it replaces the built-in default descriptor directory. Additional `--descriptor-dir` flags are appended. Use `--config <path>` to load a specific config file.

When descriptors match, the highest-priority selected descriptor can apply parser `only_rules` and `disable_rules`. The trace includes `descriptor.selected`.

Fixture harness:

Descriptor fixtures live under `tests/fixtures/descriptors/<case>/`.

Each case contains:

- `descriptor.toml`
- `input.txt`
- `negative.txt`
- `expected.rows.json`

The harness verifies isolated descriptor selection, negative inputs, expected parsed rows, and full-registry duplicate-id failures.

Not wired yet:

- descriptor conflict diagnostics beyond duplicate ids

See also:

- [`docs/SDK.md`](SDK.md)
- [`schemas/descriptor.schema.json`](../schemas/descriptor.schema.json)
- [`examples/descriptors/simple-pipes/`](../examples/descriptors/simple-pipes/)
