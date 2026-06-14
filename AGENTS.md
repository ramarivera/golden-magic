# AGENTS.md

## Project Mission

Golden Magic turns hostile, table-ish CLI text into structured data for Nushell without requiring upstream JSON. The first-class user story is:

```nu
^some-cli-with-annoying-output | from golden-magic
```

When native Nushell parsers already work, Golden Magic should reuse them. When they fail, it should explain why, pick a safer heuristic, and emit structured rows plus traceable confidence.

## Non-Negotiables

- Prefer source-grounded research before inventing parser machinery.
- Preserve Nushell semantics: records/lists/tables should feel native in Nu.
- Never require JSON from the upstream command for the core use case.
- Every heuristic must have a stable rule id, fixtures, unit tests, integration tests, and documented failure modes.
- Every vendored tool descriptor must be tested both in isolation and as part of the full registry.
- Property-based testing is mandatory for parser invariants.
- Performance regressions require an explicit tradeoff note.
- Do not add dynamic native Rust plugin loading without a security and portability design review.
- Prefer declarative descriptors, fixtures, subprocess/WASM boundaries, or compile-time registries before arbitrary runtime native code loading.

## Parallelization Rule

Optimize work for parallel execution whenever dependencies allow it. Split tasks by ownership boundary: detector, parser, registry, Nu plugin, fixtures, docs, test harness, performance harness, and known-tool descriptors. After any parallel wave, run reconciliation, full tests, and fix integration drift.

## Planning Discipline

- Use OpenSpec for feature/spec-level planning.
- Use beads/bd for task-level tracking.
- One OpenSpec task checkbox should map to exactly one bead item.
- One OpenSpec spec should map to one bead epic when practical.

## Architecture Bias

Hexagonal-ish is allowed here because the domain boundary matters:

- Core parser/detector logic must not depend on Nushell plugin APIs.
- Nu plugin/CLI adapters are outer layers.
- Descriptor loading and test harnesses are outer layers.
- Fixtures and known-tool registries must be data-first and easy to review.

## Testing Requirements

- Unit tests for individual heuristics and descriptors.
- Registry tests for descriptor picker conflicts.
- Property tests for parser invariants and hostile input.
- E2E tests that invoke a real binary/CLI where feasible.
- Nu integration tests that prove the pipeline can produce native Nu-shaped data.
- Performance tests with documented budgets before claiming speed.

## Comment Style

Code should be readable without verbose narration. Comments are for parser invariants, ambiguous heuristics, protocol decisions, and accepted tradeoffs.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:7510c1e2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
