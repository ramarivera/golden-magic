# 10k CLI Corpus Harness

Golden Magic's long-run corpus target is 10,000 popular CLI/tool repositories selected by objective GitHub stars.

This repository does not yet contain the full 10k corpus. The measurable contract is:

1. Source candidates from GitHub repository search or a checked-in export that includes repository URL, star count, primary language, and CLI evidence.
2. Sort by descending star count at the time of capture.
3. Keep exactly one manifest entry per repository.
4. For each modeled tool, add a tool pack, descriptor fixtures, backend choice, deterministic tests, and exploratory/agentic CLI notes when feasible.
5. Track progress with automated manifest checks; do not claim corpus completion until the manifest has at least 10,000 unique entries and every required modeled artifact exists.

Current bootstrap manifest: `corpus/cli-tools.seed.json`.

Run the manifest checks with:

```bash
cargo test --test corpus_manifest -- --nocapture
```
