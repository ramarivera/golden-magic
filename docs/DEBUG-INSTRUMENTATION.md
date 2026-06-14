# Debug Instrumentation Protocol

Golden Magic intentionally avoids hidden debug channels in normal builds. Parser output is pipeline data, so stdout must remain data and stderr must remain human diagnostics. Anything else can silently corrupt Nushell pipelines.

## Decision

The only supported debug surfaces today are explicit and visible:

- `--explain`
- `--output trace-json`
- parse report `trace` events
- test assertions over structured `ParseReport`

No hidden file descriptor, socket, named pipe, environment-secret handshake, or always-listening debug transport is implemented.

## Threat Model

A debug side channel can become dangerous if it:

- leaks parsed input, command output, paths, or user data
- activates outside tests by accident
- changes stdout/stderr behavior
- blocks or slows normal parsing
- introduces a local socket attack surface
- makes test-only behavior diverge from release behavior
- depends on obscurity, secret env var names, or undocumented file descriptors

## Requirements Before Any Future Side Channel

A future harness-only channel must satisfy all requirements below before implementation:

1. **Compile-time feature gate**: unavailable unless built with an explicit Cargo feature such as `harness-debug-channel`.
2. **Runtime opt-in**: requires an explicit CLI flag or test harness API call; environment variables alone are not enough.
3. **No ambient listener**: the binary must not open sockets, pipes, or extra file descriptors unless explicitly requested.
4. **No stdout/stderr interference**: normal data output and diagnostics remain unchanged.
5. **Structured schema**: messages use a versioned JSON/JSONL schema documented in this file or a sibling spec.
6. **Timeouts and bounded buffers**: no unbounded blocking writes to harness channels.
7. **Redaction strategy**: sensitive input and file paths need opt-in inclusion or redaction controls.
8. **Tests proving absence**: release/default tests must prove the channel is inactive unless enabled.
9. **Security review**: implementation PR must include a threat-model update and rationale for why explicit trace output was insufficient.

## Preferred Alternative

Prefer extending explicit trace output first:

```bash
golden-magic --output trace-json
```

or full reports:

```bash
golden-magic --output report-json
```

The trace event model is deliberately stable and rule-id based so tests can assert behavior without regexing terminal text.

## Future Schema Sketch

If a side channel becomes justified, start with JSONL events:

```json
{"version":1,"event":"parser.started","input_bytes":1234}
{"version":1,"event":"rule.selected","rule_id":"detect.delimited.tabs","confidence":0.96}
{"version":1,"event":"parser.finished","rows":42,"columns":3}
```

This sketch is not permission to implement the channel. It is a shape to evaluate if explicit trace output stops being enough.
