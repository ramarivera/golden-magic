# Prior Art Research

Golden Magic sits between Nushell's native structured-data model and the messy reality of CLIs that only print human tables. The useful prior art points toward three design rules: preserve Nu semantics at the edge, reuse format parsers when a real format exists, and keep extension boundaries data-first or sandboxed.

## Nushell Integration

Nushell plugins are external executables that speak a serialized protocol over stdio or a local socket. The official plugin docs describe plugins as applications that exchange data streams with Nu, which matches Golden Magic's current `nu_plugin_golden_magic` shape rather than an in-process dynamic library model. See the Nushell [plugin contributor docs](https://www.nushell.sh/contributor-book/plugins.html) and [plugin protocol reference](https://www.nushell.sh/contributor-book/plugin_protocol_reference.html).

`plugin use` matters operationally because it is a parser keyword and requires the plugin definition to exist in the registry at parse time. That supports the current integration-test pattern: prepare an isolated plugin registry with `plugin add --plugin-config`, then run scripts with `--plugin-config` and `plugin use`. See Nushell's [`plugin use` command docs](https://www.nushell.sh/commands/docs/plugin_use.html).

Nushell custom commands provide typed signatures and parser-time type checking. That argues for keeping Golden Magic flags narrow and typed in both the wrapper and native plugin rather than accepting arbitrary loosely typed extension payloads. See Nushell [custom command docs](https://www.nushell.sh/book/custom_commands.html).

## Built-In Format Parsers

Nu already has native format converters such as `from csv`. Its CSV parser supports separators, flexible rows, comments, header handling, and trimming. Golden Magic should treat those as first-choice parsers when descriptor or detector evidence says the input is actually CSV/TSV-like, rather than permanently maintaining weaker ad hoc logic for real delimited formats. See Nushell [`from csv`](https://www.nushell.sh/commands/docs/from_csv.html) and the broader [command reference](https://www.nushell.sh/commands/).

The Rust [`csv` crate](https://docs.rs/csv) is also mature, fast, Serde-friendly, and configurable. For CSV/TSV/semicolon variants, a future parser layer should delegate to `csv` with explicit settings and only fall back to Golden Magic heuristics when rectangular parsing fails or the input is not a real delimited format.

## Grammar And Heuristic Parsing

[`nom`](https://docs.rs/nom) is a strong fit for byte/string-level parsers where Golden Magic needs explicit grammar rules, streaming behavior, and precise failures. It is parser-combinator oriented, so it fits a future "grammar engine" only if the engine exposes a curated set of composable primitives rather than arbitrary user-authored Rust.

[Tree-sitter](https://tree-sitter.github.io/) is a parser generator and incremental parsing library. Its strength is concrete syntax trees for languages and editors, not one-shot hostile CLI table extraction. It is useful prior art for stable grammars and error nodes, but it is probably too heavy as the first Golden Magic grammar backend.

Miller (`mlr`) is an important comparator for command-line tabular data. Its docs describe it as a tool for querying, shaping, and reformatting CSV, TSV, JSON, JSON Lines, YAML, and related formats. Golden Magic should not try to become Miller; the narrower mission is "infer Nu-shaped rows from hostile text when upstream JSON/native parsers are absent." See [Miller docs](https://miller.readthedocs.io/).

## Extension Boundaries

Rust native dynamic loading exists through crates like [`libloading`](https://docs.rs/libloading/), but it is not a safe default extension story for Golden Magic. Loading native code creates trust, ABI, initialization, unloading, and platform-distribution problems. Even if `libloading` improves API safety around library and symbol lifetimes, it does not sandbox the loaded code or make arbitrary third-party machine code safe.

WebAssembly is stronger prior art for untrusted or semi-trusted extension boundaries. The Bytecode Alliance describes the [WebAssembly Component Model](https://component-model.bytecodealliance.org/) as an architecture for interoperable Wasm libraries, applications, and environments. [WASI](https://wasi.dev/) frames execution around capability-oriented sandboxing. That maps better to Golden Magic's likely future than in-process native dylibs: descriptors first, subprocesses second, WASM/WASI third, native runtime loading only after explicit design review.

## Design Implications

- Keep the native Nu plugin as a protocol executable, not a host for arbitrary in-process native plugins.
- Route known formats to real parsers (`from csv` behavior conceptually, Rust `csv` implementation in core) before using weaker heuristics.
- Treat descriptors as the first extension SDK: stable rule ids, schemas, fixtures, negative inputs, and registry conflict tests.
- Make the known-tool corpus data-first. Each tool descriptor should carry captured input, expected rows, negative inputs, and optional Nix manifest metadata.
- If a grammar engine is added, start with a small declarative grammar schema and a controlled parser implementation. Do not jump straight to user-authored Rust parser plugins.
- If executable extensions are needed, prefer subprocess or WASM/WASI boundaries before native dynamic loading.

## Open Follow-Up

- `golden-magic-by0`: safe runtime extension architecture.
- `golden-magic-9pu`: known-tool descriptor corpus.
- `golden-magic-2mf`: grammar engine.
- `golden-magic-euf`: extension-author SDK.
