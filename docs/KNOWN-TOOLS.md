# Known Tool Descriptor Corpus

Golden Magic keeps known-tool support data-first. A known-tool descriptor is a reviewed fixture pack under `tests/fixtures/descriptors/<tool-or-shape>/` with:

- `descriptor.toml`
- `input.txt`
- `negative.txt`
- `expected.rows.json`
- optional `nix.toml`

The descriptor fixture test harness verifies every pack in isolation and checks that duplicate ids fail at full-registry load time.

## Current Corpus

| Fixture | Descriptor id | Shape | Notes |
| --- | --- | --- | --- |
| `aws-s3-ls` | `known.aws.s3-ls` | repeated-space listing | Covers date/time plus size/path style output. |
| `brew-list-versions` | `known.brew.list-versions` | repeated-space package versions | Covers package/version rows from Homebrew-style output. |
| `cargo-tree-duplicates` | `known.cargo.tree-duplicates` | pipe-delimited package rows | Covers duplicate dependency summaries. |
| `docker-ps` | `known.docker.ps` | repeated-space container table | Covers image/status/ports/name columns. |
| `gh-pr-list` | `known.gh.pr-list` | repeated-space PR table | Covers issue number, state, title, owner, branch. |
| `git-branch-verbose` | `known.git.branch-verbose` | repeated-space branch rows | Covers selected branch marker, sha, upstream, subject. |
| `kubectl-get-pods` | `known.kubectl.get-pods` | repeated-space pod table | Covers name/ready/status/restarts/age columns. |
| `pnpm-outdated` | `known.pnpm.outdated` | repeated-space dependency table | Covers package/current/latest/dependent rows. |
| `ps-basic` | `known.ps.basic` | repeated-space process table | Covers PID/TTY/time/command rows. |
| `rust-declarations` | `known.rust.declarations` | tree-sitter Rust declarations | Covers `mod`, `struct`, and `fn` extraction through the `tree-sitter` backend with `grammar = "rust"`. |
| `sectioned-services` | `known.sectioned.services` | sectioned key-value blocks | Covers repeated `section: <name>` blocks through the `sections` backend. |
| `systemctl-list-units` | `known.systemctl.list-units` | repeated-space service table | Covers service unit/load/active/sub/description rows. |

`generic-pipes` remains the generic pipe-delimited descriptor fixture and is not counted as a known-tool descriptor.

## Limits

The current descriptor schema can select implemented parser backends and heuristic rule hints. It cannot yet express column-specific grammars, free-text trailing fields, optional columns, or tool-specific cleanup. Fixtures in this corpus are intentionally shaped to what the current parser can prove.

More hostile real outputs need either a new descriptor-selected backend or a tree-sitter grammar target with fixtures and traceable row extraction.
