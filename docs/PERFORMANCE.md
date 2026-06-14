# Golden Magic Performance

Golden Magic includes a Criterion benchmark suite for parser hot paths.

Run:

```bash
cargo bench --bench parser
```

Fast local smoke run:

```bash
cargo bench --bench parser -- --sample-size 10
```

## Current Benchmarks

Initial local baseline from 2026-06-14 on Ramiro's machine, using `--sample-size 10`:

| Benchmark | Time |
| --- | ---: |
| `parse medium rectangular tsv` (1,000 × 8) | ~0.98 ms |
| `parse large rectangular tsv` (10,000 × 8) | ~9.66 ms |
| `parse first-row headers` (1,000 × 2) | ~0.38 ms |

These are baseline observations, not hard gates yet.

## Future Gates

Before claiming production-grade performance, define explicit budgets for:

- interactive CLI latency
- large stdin parsing
- descriptor registry selection
- Nushell adapter overhead
- full fixture harness execution

Any regression beyond the budget should require either an optimization or a documented tradeoff.
