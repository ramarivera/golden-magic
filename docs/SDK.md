# Golden Magic Extension Author SDK

The current extension SDK is descriptor-first. It gives authors a stable TOML descriptor format, a validation command, fixture conventions, a schema for editor tooling, and compatibility guidance.

It does not load arbitrary native Rust code at runtime. That remains deliberately outside the current SDK because runtime native loading needs a separate security and portability review.

## Descriptor Pack Layout

Use one directory per descriptor pack:

```text
my-pack/
  descriptor.toml
  input.txt
  negative.txt
  expected.rows.json
```

Only `descriptor.toml` is loaded by Golden Magic. The fixture files are for author tests and review.

## Descriptor Format

```toml
id = "example.simple-pipes"
name = "Example Simple Pipes"
priority = 10

[matches]
required_substrings = ["NAME|STATUS"]

[parser]
backend = "heuristic"
only_rules = ["detect.delimited.pipes"]
disable_rules = []
```

Required fields:

- `id`: stable unique id, using lowercase letters, numbers, dots, underscores, or dashes.
- `name`: human-readable name.

Optional fields:

- `priority`: higher priority wins when multiple descriptors match.
- `matches.required_substrings`: every listed string must appear in the input.
- `parser.backend`: parser backend id. Implemented ids are `heuristic` and `sections`.
- `parser.only_rules`: restrict parser selection to specific stable rule ids.
- `parser.disable_rules`: disable specific stable rule ids.

Inspect available rule ids:

```bash
golden-magic --list-rules
```

Inspect available parser backend ids:

```bash
golden-magic --list-backends
```

Validate a descriptor pack without stdin:

```bash
golden-magic --validate-descriptor-dir ./my-pack
```

The validator checks TOML loading, duplicate descriptor ids, parser backend ids, and parser rule ids. A valid directory prints the number of descriptors loaded. Unknown rules fail with a message that points back to `--list-rules`.

## Schema

Editor and CI tooling can use:

```text
schemas/descriptor.schema.json
```

The schema documents the current descriptor shape. Golden Magic itself remains the source of truth for rule id validation because rule ids come from the compiled parser registry.

## Fixture Expectations

Recommended fixture files:

- `input.txt`: representative positive output.
- `negative.txt`: nearby output that should not select the descriptor.
- `expected.rows.json`: expected rows from `golden-magic --output rows-json`.

The in-repo descriptor harness uses the same shape under `tests/fixtures/descriptors/<case>/`.

## Compatibility

Descriptor authors should treat these as stable:

- descriptor ids
- parser rule ids returned by `--list-rules`
- parser backend id `heuristic`
- parser backend id `sections`
- `matches.required_substrings`
- `parser.only_rules`
- `parser.disable_rules`
- fixture file names above

Adding new matcher fields or parser engines is allowed in future releases, but existing descriptor fields should remain backwards compatible unless a spec explicitly breaks them. `tree-sitter` is reserved as a candidate backend in [`docs/PARSER-BACKENDS.md`](PARSER-BACKENDS.md), but descriptors cannot use it until the backend is implemented.

When a descriptor selects `backend = "heuristic"`, Golden Magic records `backend.heuristic` in trace output before heuristic rule events.
When a descriptor selects `backend = "sections"`, Golden Magic parses repeated `section: <name>` blocks followed by `key: value` fields into one row per section.

## Publishing

Publish descriptor packs as plain source files. Keep one descriptor per TOML file when possible, include fixtures beside the descriptor, and pin any external command output used to create fixtures in review notes or tests.

Do not publish native dynamic libraries as Golden Magic extensions. The supported extension boundary today is declarative descriptor data.
