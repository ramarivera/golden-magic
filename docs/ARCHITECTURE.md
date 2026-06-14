# Golden Magic Architecture

Golden Magic uses a hexagonal-ish split so parser logic stays independent from Nushell, CLIs, descriptors, and test harnesses.

## Layers

```text
┌───────────────────────────────────────────────────────────┐
│ Adapters                                                   │
│ - CLI binary                                               │
│ - Nushell wrapper module                                   │
│ - future native Nushell plugin                             │
│ - future test/runtime harness                              │
└───────────────────────────┬───────────────────────────────┘
                            │
┌───────────────────────────▼───────────────────────────────┐
│ Application Services                                       │
│ - parse input                                              │
│ - load descriptors                                         │
│ - select parser strategy                                   │
│ - emit report/result/trace                                 │
└───────────────────────────┬───────────────────────────────┘
                            │
┌───────────────────────────▼───────────────────────────────┐
│ Domain Core                                                │
│ - delimiter detection                                      │
│ - fixed-width detection                                    │
│ - fallback line preservation                               │
│ - confidence and trace events                              │
│ - future type inference                                    │
└───────────────────────────────────────────────────────────┘
```

## Current Parser Flow

1. Normalize input into non-empty significant lines.
2. Try rectangular delimiter candidates in priority order: tab, pipe, comma, semicolon.
3. If no delimiter candidate is rectangular, try repeated-space fixed-width splitting.
4. If table inference is unsafe, return a one-column `line` table.
5. Emit `ParseReport` with `kind`, `confidence`, `columns`, `rows`, and `trace`.

## Descriptor Registry

The current descriptor substrate lives in `src/descriptors.rs`. It can recursively load TOML descriptors, reject duplicate ids, sort by priority, and select descriptors by required substrings. The CLI consumes selected descriptors by applying their parser rule hints before calling the parser core.

## Extension Direction

Known-tool support should start as declarative pattern packs:

```text
descriptors/<tool>/<pattern>.toml|yaml|ron
fixtures/<tool>/<case>.txt
fixtures/<tool>/<case>.expected.nuon|json
```

A descriptor should be testable alone and inside the full registry. Native dynamic loading is intentionally not part of the early design.

## Debug/Instrumentation Direction

Debug channels must not interfere with stdout/stderr pipeline semantics. Prefer explicit `--output trace-json` / `--explain` outputs first. `docs/DEBUG-INSTRUMENTATION.md` documents the current decision: no hidden side channel in normal builds, and strict compile-time/runtime gates before any future harness-only channel.

## Nushell Integration Direction

The current Nushell adapter is `nu/golden-magic.nu`, which exports `from golden-magic` and shells out to the Rust CLI. This gives the intended spell shape now while preserving the parser core as the stable boundary.

The eventual target is a native Nushell plugin command shaped like `from golden-magic`, backed by the same domain core and compatible with the wrapper's command surface where practical.
