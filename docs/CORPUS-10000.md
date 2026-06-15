# 10k CLI Corpus Harness

Golden Magic's long-run corpus target is 10,000 popular CLI/tool repositories selected by objective GitHub stars.

This repository does not yet contain the full 10k corpus. The measurable contract is:

1. Source candidates from GitHub repository search or a checked-in export that includes repository URL, star count, primary language, and CLI evidence.
2. Sort by descending star count at the time of capture.
3. Keep exactly one manifest entry per repository.
4. For each modeled tool, add a tool pack, descriptor fixtures, backend choice, deterministic tests, and exploratory/agentic CLI notes when feasible.
5. Track progress with automated manifest checks; do not claim corpus completion until the manifest has at least 10,000 unique entries and every required modeled artifact exists.

Current bootstrap manifest: `corpus/cli-tools.seed.json`.

Refresh the seed with:

```bash
scripts/fetch_cli_corpus_seed.sh 100
```

The script reads partitioned GitHub search queries from:

```text
corpus/cli-corpus.queries.txt
```

Each query is fetched independently with GitHub's star ordering, then the script
merges all partitions, deduplicates by repository URL, preserves every
`source_queries` value that found the repository, and reranks the combined seed
by descending stars. This is still a seed harness, not the full corpus.

Override `GOLDEN_MAGIC_CORPUS_QUERIES` to use a different query-partition file.
Any checked-in seed must preserve `source_query`, `source_queries`, `fetched_at`,
`stars`, and descending rank order.

Run the manifest checks with:

```bash
cargo test --test corpus_manifest -- --nocapture
```
