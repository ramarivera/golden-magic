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

If descriptors cannot express a tool, the next extension boundary should be a subprocess protocol, not native dynamic loading.

Subprocess extensions should:

- be executable files launched explicitly by path or descriptor reference
- receive input on stdin
- emit rows/report/trace JSON on stdout
- emit diagnostics on stderr
- use a versioned JSON protocol
- have timeouts, output-size limits, and clear failure reporting
- be opt-in per descriptor or config file

Subprocess extensions isolate crashes better than in-process native libraries and work across languages without forcing Rust ABI stability.

## Future Boundary: WASM/WASI

WASM/WASI is the preferred future path for sandboxable executable extensions.

A WASM extension design must specify:

- host imports and denied capabilities
- filesystem access policy
- network access policy
- memory and fuel limits
- deterministic input/output protocol
- component versioning
- how descriptors bind to WASM modules
- test fixtures and compatibility gates

WASM is not implemented yet. It is the preferred executable-plugin research direction because it offers a clearer sandbox story than loading native dylibs.

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
- Subprocess extensions: designed here, not implemented.
- WASM/WASI extensions: design direction only, not implemented.
- Native runtime loading: explicitly rejected by [`docs/NATIVE-RUNTIME-REVIEW.md`](NATIVE-RUNTIME-REVIEW.md) until separate approval and acceptance gates are satisfied.
