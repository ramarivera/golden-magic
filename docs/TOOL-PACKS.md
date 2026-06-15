# Declarative Tool Packs

Golden Magic's safest plugin surface is data.

A tool pack is a directory of TOML files that describes a command family, its subcommands, argument shapes, output patterns, parser backend choices, and fixtures. Loading a tool pack should mean reading validated data from an explicit descriptor directory, not executing native code.

The Rust data model and strict TOML loader live in `src/tool_packs.rs`.

## Directory Shape

```text
tool-packs/<tool>/
  tool.toml
  descriptors/
    <command-or-output-shape>/descriptor.toml
    <command-or-output-shape>/input.txt
    <command-or-output-shape>/negative.txt
    <command-or-output-shape>/expected.rows.json
    <command-or-output-shape>/nix.toml
```

`descriptors/` reuses the existing Golden Magic descriptor harness. `tool.toml` adds command-model metadata around those descriptors.

## Tool Model

```toml
id = "tool.git"
name = "git"
version = "1"

[[commands]]
name = "branch"
description = "Inspect branches"

[[commands.subcommands]]
name = "--verbose"
descriptor = "known.git.branch-verbose"
patterns = ["git branch -v", "git branch --verbose"]

[[commands.args]]
name = "--all"
kind = "flag"
patterns = ["-a", "--all"]
```

Fields:

- `id`: stable tool-pack id.
- `name`: executable or command family name.
- `version`: tool-pack schema version.
- `commands`: top-level commands for the tool.
- `commands.subcommands`: nested command forms or mode selectors.
- `descriptor`: descriptor id used to parse that output shape.
- `patterns`: command-line spellings that commonly produce that output.
- `commands.args`: declared args that affect output shape.

## Loading Rules

- Tool packs are loaded from explicit `--tool-pack-dir` paths, `tool_pack_dirs` in config, or the default XDG path.
- Unknown schema fields fail validation.
- Descriptor ids referenced by `tool.toml` must exist in the same pack or a configured registry.
- Tool packs do not execute code.
- Tool packs do not read secrets.
- Tool packs do not imply shell completion, command execution, or network access.
- Every output shape still needs fixtures and expected rows.

## CLI

Validate a tool-pack directory against configured descriptors:

```bash
golden-magic \
  --no-default-descriptors \
  --descriptor-dir ./descriptors \
  --validate-tool-pack-dir ./tool-packs
```

List discovered tool packs:

```bash
golden-magic \
  --config ./config.toml \
  --list-tool-packs
```

Config supports both descriptor and tool-pack roots:

```toml
descriptor_dirs = ["./descriptors"]
tool_pack_dirs = ["./tool-packs"]
```

Default discovery uses:

```text
$XDG_CONFIG_HOME/golden-magic/descriptors
$XDG_CONFIG_HOME/golden-magic/tool-packs
```

## Relation To Executable Extensions

Use tool packs first. If data cannot express the parser, then use:

1. a new core backend,
2. a tree-sitter grammar backend for syntax-shaped input,
3. a subprocess extension protocol,
4. a WASM/WASI extension protocol.

Native runtime libraries remain the last resort and are rejected until the gates in [`docs/NATIVE-RUNTIME-REVIEW.md`](NATIVE-RUNTIME-REVIEW.md) are satisfied.
