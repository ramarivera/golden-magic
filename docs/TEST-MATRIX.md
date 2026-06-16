# Test Matrix

Golden Magic has a generated deterministic integration matrix for the explicit 2,000+ test-case target.

Run it with:

```bash
cargo test --test generated_matrix -- --nocapture
```

The harness fails if fewer than 2,000 generated cases execute.

Current generated case budget:

| Surface | Cases |
| --- | ---: |
| delimiter parsing | 900 |
| first-row headers | 350 |
| fallback lines | 300 |
| sections backend | 300 |
| tree-sitter Rust backend | 300 |
| wasm-json backend | 200 |
| tool-pack loader | 250 |
| tool-pack duplicate-id rejection | 100 |
| total | 2,700 |

Descriptor fixtures also expose a countable matrix through the shared test
support utilities. Each fixture contributes expected-row, negative-input, and
isolation assertions, so the current 887+ descriptor fixtures represent at
least 2,661 fixture-derived assertion cases in addition to the generated parser
matrix.

This does not complete the 10,000-tool corpus target. It makes the test-count requirement measurable and keeps it from being silently reduced.
