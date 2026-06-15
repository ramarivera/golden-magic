use golden_magic::tool_packs::{ToolArgKind, load_tool_pack_file};
use golden_magic::{HeaderMode, ParseKind, ParseOptions, parse, parse_with_options};
use std::fs;
use tempfile::tempdir;

const MIN_GENERATED_CASES: usize = 2_000;

#[test]
fn generated_parser_matrix_runs_more_than_two_thousand_cases() {
    let mut cases = 0usize;

    cases += delimiter_matrix_cases();
    cases += first_row_header_cases();
    cases += fallback_line_cases();
    cases += sections_backend_cases();
    cases += tree_sitter_rust_cases();
    cases += wasm_json_cases();
    cases += tool_pack_loader_cases();

    eprintln!("generated parser matrix ran {cases} deterministic cases");
    assert!(
        cases >= MIN_GENERATED_CASES,
        "generated parser matrix ran {cases} cases; expected at least {MIN_GENERATED_CASES}"
    );
}

fn delimiter_matrix_cases() -> usize {
    let delimiters = [
        ('\t', "detect.delimited.tabs"),
        ('|', "detect.delimited.pipes"),
    ];
    let mut cases = 0usize;

    for case_id in 0..900 {
        let (delimiter, rule_id) = delimiters[case_id % delimiters.len()];
        let width = 2 + (case_id % 5);
        let height = 2 + (case_id % 7);
        let input = cell_table(case_id, height, width, delimiter);

        let report = parse(&input);

        assert_eq!(report.kind, ParseKind::Delimited, "case {case_id}");
        assert_eq!(report.columns.len(), width, "case {case_id}");
        assert_eq!(report.rows.len(), height, "case {case_id}");
        assert!(
            report.trace.iter().any(|event| event.rule_id == rule_id),
            "case {case_id} should trace {rule_id}"
        );
        cases += 1;
    }

    cases
}

fn first_row_header_cases() -> usize {
    let mut cases = 0usize;

    for case_id in 0..350 {
        let width = 2 + (case_id % 4);
        let height = 1 + (case_id % 6);
        let headers = (0..width)
            .map(|column| format!("Header {case_id} {column}"))
            .collect::<Vec<_>>()
            .join("\t");
        let input = format!("{headers}\n{}", cell_table(case_id, height, width, '\t'));
        let options = ParseOptions::new().header_mode(HeaderMode::FirstRow);

        let report = parse_with_options(&input, &options);

        assert_eq!(report.kind, ParseKind::Delimited, "case {case_id}");
        assert_eq!(report.rows.len(), height, "case {case_id}");
        assert_eq!(report.columns[0], format!("header-{case_id}-0"));
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "headers.first-row"),
            "case {case_id} should trace first-row headers"
        );
        cases += 1;
    }

    cases
}

fn fallback_line_cases() -> usize {
    let mut cases = 0usize;

    for case_id in 0..300 {
        let input = format!("single opaque line {case_id}\nsecond opaque line {case_id}");

        let report = parse(&input);

        assert_eq!(report.kind, ParseKind::Lines, "case {case_id}");
        assert_eq!(report.rows.len(), 2, "case {case_id}");
        assert_eq!(
            report.rows[0]["line"],
            format!("single opaque line {case_id}")
        );
        cases += 1;
    }

    cases
}

fn sections_backend_cases() -> usize {
    let mut cases = 0usize;

    for case_id in 0..300 {
        let input = format!(
            "section: svc-{case_id}\n  status: ok\n  owner: team-{case_id}\nsection: worker-{case_id}\n  status: degraded"
        );
        let options = ParseOptions::new().backend("sections");

        let report = parse_with_options(&input, &options);

        assert_eq!(report.kind, ParseKind::Sections, "case {case_id}");
        assert_eq!(report.rows.len(), 2, "case {case_id}");
        assert_eq!(report.rows[0]["section"], format!("svc-{case_id}"));
        assert_eq!(report.rows[1]["owner"], "");
        cases += 1;
    }

    cases
}

fn tree_sitter_rust_cases() -> usize {
    let mut cases = 0usize;

    for case_id in 0..300 {
        let input = format!(
            "mod module_{case_id} {{\n  pub struct Tool{case_id};\n  pub fn run_{case_id}() {{}}\n}}\n"
        );
        let options = ParseOptions::new().backend("tree-sitter-rust");

        let report = parse_with_options(&input, &options);

        assert_eq!(report.kind, ParseKind::TreeSitter, "case {case_id}");
        assert_eq!(report.rows.len(), 3, "case {case_id}");
        assert_eq!(report.rows[0]["name"], format!("module_{case_id}"));
        assert_eq!(report.rows[1]["kind"], "struct");
        assert_eq!(report.rows[2]["kind"], "function");
        cases += 1;
    }

    cases
}

fn wasm_json_cases() -> usize {
    let dir = tempdir().expect("temp dir");
    let module = dir.path().join("parser-plugin.wat");
    fs::write(
        &module,
        r#"(module
  (memory (export "memory") 1)
  (data (i32.const 2048) "{\"protocol\":\"golden-magic.wasm-json.v1\",\"rows\":[{\"name\":\"matrix\",\"status\":\"ok\"}]}")
  (func (export "golden_magic_parse") (param $ptr i32) (param $len i32) (result i64)
    (i64.or
      (i64.shl (i64.const 2048) (i64.const 32))
      (i64.const 81)
    )
  )
)"#,
    )
    .expect("write wasm matrix module");

    let mut cases = 0usize;
    for case_id in 0..200 {
        let options = ParseOptions::new()
            .backend("wasm-json")
            .wasm_module(&module);
        let report = parse_with_options(&format!("wasm-row: matrix {case_id}\n"), &options);

        assert_eq!(report.kind, ParseKind::Plugin, "case {case_id}");
        assert_eq!(report.rows.len(), 1, "case {case_id}");
        assert_eq!(report.rows[0]["name"], "matrix");
        assert_eq!(report.rows[0]["status"], "ok");
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "backend.wasm-json.parsed"),
            "case {case_id} should trace wasm-json parse"
        );
        cases += 1;
    }

    cases
}

fn tool_pack_loader_cases() -> usize {
    let mut cases = 0usize;

    for case_id in 0..250 {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("tool.toml");
        fs::write(
            &path,
            format!(
                r#"
id = "tool.case-{case_id}"
name = "tool-{case_id}"
version = "1"

[[commands]]
name = "cmd-{case_id}"
descriptor = "known.case.{case_id}"

[[commands.args]]
name = "--flag-{case_id}"
kind = "flag"
affects_output = true
"#
            ),
        )
        .expect("write tool pack");

        let loaded = load_tool_pack_file(&path).expect("tool pack loads");

        assert_eq!(loaded.pack.id, format!("tool.case-{case_id}"));
        assert_eq!(
            loaded.pack.descriptor_refs(),
            vec![format!("known.case.{case_id}")]
        );
        assert_eq!(loaded.pack.commands[0].args[0].kind, ToolArgKind::Flag);
        cases += 1;
    }

    cases
}

fn cell_table(case_id: usize, rows: usize, columns: usize, delimiter: char) -> String {
    (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| format!("cell-{case_id}-{row}-{column}"))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
