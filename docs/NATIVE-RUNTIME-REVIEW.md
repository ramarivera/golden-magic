# Native Runtime Extension Review

Date: 2026-06-15

Golden Magic does not implement arbitrary native Rust runtime extension loading. This review is the design gate required before that could change.

## Decision

Native runtime loading remains rejected for the current project.

Descriptor packs, subprocess extensions, and WASM/WASI are the supported or preferred extension boundaries. Loading `.so`, `.dylib`, or `.dll` files into the Golden Magic process is not allowed until every acceptance gate in this document has an implemented, tested answer.

## Scope

"Native runtime loading" means discovering and loading compiled native libraries at runtime with `dlopen`, `LoadLibrary`, `libloading`, or equivalent APIs, then calling exported parser functions inside the Golden Magic process.

This is different from:

- the native Nushell plugin binary, which is a separate executable speaking the Nu plugin protocol
- descriptor packs, which are data-only
- subprocess extensions, which run out of process
- WASM/WASI modules, which can be sandboxed by a host runtime

## Threat Model

Native plugins execute with the same privileges as Golden Magic.

They can:

- read process memory, environment variables, stdin, stdout, stderr, and open file descriptors
- read or mutate files available to the process
- open network connections through normal system APIs
- run initialization code before Golden Magic can validate behavior
- panic, abort, deadlock, corrupt memory, leak memory, or poison global state
- produce rows, traces, and diagnostics that look first-party unless provenance is enforced

Golden Magic's first-class use case is shell pipeline parsing. That means a compromised or careless plugin may run in contexts that include secrets, generated credentials, project source, or command output that users did not intend to expose.

## Portability Problems

Rust does not provide a stable ABI for arbitrary dynamic plugin interfaces. A native plugin ABI must therefore avoid Rust types across the boundary or pin a compiler/toolchain and build matrix tightly enough to be practical.

Any native ABI design must answer:

- C ABI shape for input, output, allocation, and error ownership
- plugin API version negotiation
- struct layout and alignment compatibility
- allocator ownership for strings, buffers, rows, and diagnostics
- panic and unwind behavior across FFI boundaries
- symbol naming and namespacing
- macOS code signing and quarantine behavior
- Linux distro/glibc/musl compatibility
- Windows DLL search path and loader behavior
- Nix packaging and reproducibility
- unload behavior and lifetime guarantees for borrowed symbols

Without those answers, native loading is not portable enough to be a Golden Magic feature.

## Security Requirements

A future native-loading proposal must include:

1. An explicit trust model for plugin authors and distributors.
2. A plugin manifest with name, version, API version, supported targets, hash, signature status, and requested capabilities.
3. A default-deny discovery path. Golden Magic must never auto-load libraries from writable project directories without an explicit config entry.
4. Pinning by digest, not only by path or semver.
5. Provenance in every trace event emitted by plugin code.
6. Output validation before plugin rows enter the normal parser result.
7. Timeouts or cancellation boundaries where the platform can enforce them. If in-process native code cannot be interrupted safely, that limitation must be called out as a reason to reject the design.
8. Crash behavior documentation. A plugin crash must not be described as recoverable if it can take down the process.
9. Negative fixtures for malicious or malformed plugin outputs.
10. A reviewable install/update story that does not train users to run untrusted native code casually.

## Required Test Matrix

Native loading cannot be accepted without tests for:

- unsupported API versions
- missing required symbols
- duplicate plugin ids
- bad manifests
- hash mismatch
- signature or trust failure
- invalid UTF-8 and oversized output
- plugin panic or abort behavior
- plugin returning malformed rows
- plugin diagnostics and trace provenance
- platform-specific loading errors on macOS, Linux, and Windows

These tests must run in CI or be explicitly documented as manual release gates. A single happy-path fixture is not enough.

## Preferred Alternatives

### Descriptor Packs

Use descriptor packs when the parser can be described with matching rules, parser hints, backend selection, fixtures, and expected rows. This remains the default extension SDK.

### Subprocess Extensions

Use subprocess extensions when code is necessary. A subprocess can be written in Rust or any other language, but it communicates through a versioned JSON protocol over stdin/stdout and can be killed by the host.

This is the preferred next executable boundary because crashes and global state are isolated from Golden Magic.

### WASM/WASI

Use WASM/WASI for sandboxable executable parser extensions after a host/runtime design exists. WASM still needs capability policy, memory limits, fuel/time limits, and fixture coverage, but its sandbox story is clearer than native dynamic libraries.

## Acceptance Gates Before Implementation

Native runtime loading may only move from rejected to implementable if all of these are true:

1. A real parser use case cannot be handled by descriptors, current backends, subprocesses, or WASM/WASI.
2. The user explicitly approves taking on native runtime loading despite the security and portability cost.
3. A versioned ABI document exists.
4. A manifest schema exists and is validated.
5. Plugin discovery is explicit and digest-pinned.
6. The implementation has a cross-platform CI or documented release-test matrix.
7. FFI ownership, allocator, panic, and unload behavior are tested.
8. Trace provenance distinguishes plugin-originated rows from core parser rows.
9. Documentation warns users that native plugins execute with full process privileges.
10. A safer boundary was considered and rejected with evidence.

Until then, "arbitrary Rust runtime extension/plugin loading" remains a deliberately unimplemented feature, not a missing parser bug.
