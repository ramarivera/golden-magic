# Golden Magic Beads

This project uses beads/bd for task-level tracking. The repo currently has manual seed tasks until the bd database is initialized.

OpenSpec mapping rule:

- One OpenSpec task checkbox should map to one bead item.
- One OpenSpec feature spec should map to one bead epic where practical.

Seed epic:

- `golden-magic-core`: Generic parser core and CLI rule controls.

Deferred epics:

- `nu-plugin-adapter`: `from golden-magic` plugin integration.
- `descriptor-registry`: declarative known-output registry and XDG extension loading.
- `fixture-harness`: isolated CLI fixture install/run/teardown, likely Nix-backed.
- `perf-harness`: benchmark suite and hard performance gates.
