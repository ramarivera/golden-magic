# Extension Architecture

Golden Magic should not load arbitrary native Rust plugins at runtime. Native dynamic loading creates too many unresolved problems for this project right now: trust, ABI compatibility, platform packaging, crash isolation, filesystem/network access, initialization side effects, and unload/lifetime behavior.

The extension model is intentionally staged.

## Supported Now: Descriptor Packs

Descriptor packs are the current extension surface.

A descriptor pack is data, not code:

```text
descriptors/<tool-or-domain>/
  descriptor.toml
  input.txt
  negative.txt
  expected.rows.json
  nix.toml
```

Descriptors can:

- match input using declared rules
- apply parser hints such as `only_rules` and `disable_rules`
- participate in full-registry duplicate-id checks
- run fixture tests with expected rows and negative inputs
- optionally run Nix-backed command fixtures when Nix is available

Descriptors cannot:

- execute arbitrary code
- read ambient secrets
- mutate the filesystem
- open network connections
- load native libraries

## Next Boundary: Subprocess Extensions

If descriptors cannot express a tool, the next extension boundary is a
subprocess protocol, not native dynamic loading. Golden Magic implements this
first as the descriptor-selected `executable-json` backend.

```toml
[parser]
backend = "executable-json"
executable = "./parser-plugin"
```

Golden Magic writes the raw input to stdin and expects stdout to be a v1 JSON
envelope:

```json
{
  "protocol": "golden-magic.executable-json.v1",
  "rows": [
    {"name": "alpha", "status": "ok"}
  ]
}
```

Relative executable paths resolve beside `descriptor.toml`. The current host
also accepts a bare JSON array of row objects for the first compatibility pass,
but new plugins should emit the envelope. The host enforces a 2 second timeout,
a 1 MiB stdout cap, and a 64 KiB stderr cap.

Subprocess extensions should:

- be executable files launched explicitly by path or descriptor reference
- receive input on stdin
- emit rows/report/trace JSON on stdout
- emit diagnostics on stderr
- use a versioned JSON protocol as this surface evolves
- have timeouts, output-size limits, and clear failure reporting
- be opt-in per descriptor or config file

Subprocess extensions isolate crashes better than in-process native libraries and work across languages without forcing Rust ABI stability.

## Supported Now: WASM Parser Plugins

WASM is the preferred executable extension path when a descriptor needs code but
should not load native libraries into the Golden Magic process. Golden Magic
implements this as the descriptor-selected `wasm-json` backend.

```toml
[parser]
backend = "wasm-json"
module = "./parser-plugin.wasm"
```

The v1 ABI is intentionally small:

- export `memory`
- export `golden_magic_parse(ptr: i32, len: i32) -> i64`
- receive raw input bytes at `(ptr, len)`
- return an `i64` whose high 32 bits are the JSON output offset and whose low
  32 bits are the JSON output length
- emit a `golden-magic.wasm-json.v1` JSON envelope from that output range

The host provides no imports, enables fuel metering, caps input and output at
1 MiB each, and loads only descriptor-declared module paths.

## Forbidden Until Review: Native Runtime Loading

Native runtime loading means loading `.so`, `.dylib`, or `.dll` code into the Golden Magic process with APIs such as `dlopen`, `LoadLibrary`, or Rust wrappers around them.

This is forbidden until the security and portability gates in [`docs/NATIVE-RUNTIME-REVIEW.md`](NATIVE-RUNTIME-REVIEW.md) are satisfied. In short, any native-loading design must answer:

- What is the trust model for plugin authors and plugin distribution?
- How are ABI and Rust compiler/version compatibility handled?
- What happens when a plugin panics, aborts, leaks, deadlocks, or corrupts memory?
- What prevents plugin initialization code from doing unwanted work at load time?
- How are library unload and symbol lifetimes proven safe?
- How are macOS, Linux, Windows, and Nix packaging differences handled?
- How are plugins pinned, signed, hashed, or otherwise verified?
- How does a user audit which native code will run?

Until those answers exist in implemented, tested form, native loading is not a Golden Magic feature.

## Implementation Implications

- Keep parser core independent from Nushell and extension runtimes.
- Keep descriptor loading data-first and reviewable.
- Add schema validation before expanding descriptor features.
- Treat subprocess/WASM outputs as untrusted parser candidates that still need validation.
- Require fixtures for every extension surface, including negative inputs and expected rows.
- Preserve traceability: every parser decision from an extension must produce stable rule ids or extension ids.

## Status

- Descriptor packs: implemented substrate and fixture harness.
- Descriptor-driven Nix manifests: implemented and live-verified through `nixos/nix:latest`; host runs still require Nix on `PATH`.
- Subprocess extensions: implemented through `executable-json`.
- WASM parser plugins: implemented through `wasm-json`.
- Native runtime loading: explicitly rejected by [`docs/NATIVE-RUNTIME-REVIEW.md`](NATIVE-RUNTIME-REVIEW.md) until separate approval and acceptance gates are satisfied.
