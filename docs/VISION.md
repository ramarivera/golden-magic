# Golden Magic Vision

Golden Magic exists because many excellent CLIs still emit human-formatted text by default, and humans cannot reliably see the invisible rules behind that text. Tabs, padded spaces, fixed-width columns, box-drawing tables, and ad-hoc status lines can all look similar in a terminal.

The goal is not to replace structured CLI output when it exists. If a command offers JSON, CSV, TSV, or another stable machine format, use it. Golden Magic is for the messy middle: output that is structured enough to parse, but not honest enough to identify itself.

## North Star

```nu
^some-cli | from golden-magic
```

The result should feel like native Nushell data: records, lists, strings, numbers, dates where safe, and a trace explaining which heuristics fired.

## Design Principles

1. **Reuse before inventing**: delegate to known format parsers when confidently detected.
2. **Trace every guess**: each heuristic has a stable id, confidence, and reason.
3. **Fail soft**: if table inference is unsafe, return lines rather than hallucinated columns.
4. **Data-first extension**: known patterns should start as descriptors and fixtures, not arbitrary code.
5. **Test hostile inputs**: parser correctness must be defended with unit, property, integration, and performance tests.
6. **Keep the core portable**: the parser engine must not depend on Nushell plugin APIs.

## Non-Goals For The First Cut

- Perfect parsing of all terminal output.
- Runtime loading of arbitrary native Rust plugins.
- Tool-specific descriptors before the generic detector/parser pipeline is stable.
- Replacing upstream JSON/CSV/TSV modes when those modes are available and acceptable.
