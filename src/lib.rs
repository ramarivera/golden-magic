pub mod cli;
pub mod config;
pub mod descriptors;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type Row = BTreeMap<String, String>;

const DELIMITED_TABS: &str = "detect.delimited.tabs";
const DELIMITED_COMMAS: &str = "detect.delimited.commas";
const DELIMITED_SEMICOLONS: &str = "detect.delimited.semicolons";
const DELIMITED_PIPES: &str = "detect.delimited.pipes";
const FIXED_WIDTH_GAPS: &str = "detect.fixed-width.gaps";
const FALLBACK_EMPTY: &str = "fallback.empty";
const FALLBACK_LINES: &str = "fallback.lines";
const BACKEND_HEURISTIC: &str = "heuristic";
const BACKEND_SECTIONS: &str = "sections";
const BACKEND_SECTIONS_SELECTED: &str = "backend.sections";
const BACKEND_SECTIONS_PARSED: &str = "backend.sections.parsed";
const BACKEND_SECTIONS_MALFORMED: &str = "backend.sections.malformed";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParseReport {
    pub kind: ParseKind,
    pub confidence: f32,
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub trace: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParseKind {
    Delimited,
    FixedWidth,
    Lines,
    Sections,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub rule_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeaderMode {
    #[default]
    Generated,
    FirstRow,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParseOptions {
    disabled_rules: Vec<String>,
    only_rules: Vec<String>,
    header_mode: HeaderMode,
    backend: ParserBackendSelection,
    trace_events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParserBackendSelection {
    id: String,
    explicit: bool,
}

impl Default for ParserBackendSelection {
    fn default() -> Self {
        Self {
            id: BACKEND_HEURISTIC.to_string(),
            explicit: false,
        }
    }
}

impl ParseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disable_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.disabled_rules.push(rule_id.into());
        self
    }

    pub fn only_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.only_rules.push(rule_id.into());
        self
    }

    pub fn header_mode(mut self, header_mode: HeaderMode) -> Self {
        self.header_mode = header_mode;
        self
    }

    pub fn backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = ParserBackendSelection {
            id: backend.into(),
            explicit: true,
        };
        self
    }

    pub fn trace_event(mut self, rule_id: &'static str, message: impl Into<String>) -> Self {
        self.trace_events.push(event(rule_id, message));
        self
    }

    pub fn disabled_rules(&self) -> &[String] {
        &self.disabled_rules
    }

    pub fn only_rules(&self) -> &[String] {
        &self.only_rules
    }

    pub fn selected_header_mode(&self) -> HeaderMode {
        self.header_mode
    }

    pub fn selected_backend(&self) -> &str {
        &self.backend.id
    }

    pub fn configured_trace_events(&self) -> &[TraceEvent] {
        &self.trace_events
    }

    fn allows(&self, rule_id: &str) -> bool {
        let is_enabled = !self
            .disabled_rules
            .iter()
            .any(|disabled| disabled == rule_id);
        let is_in_only_set =
            self.only_rules.is_empty() || self.only_rules.iter().any(|allowed| allowed == rule_id);

        is_enabled && is_in_only_set
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    delimiter: char,
    rule_id: &'static str,
    columns: usize,
    rows: Vec<Vec<String>>,
}

pub fn parse(input: &str) -> ParseReport {
    parse_with_options(input, &ParseOptions::default())
}

pub fn parse_with_options(input: &str, options: &ParseOptions) -> ParseReport {
    match options.selected_backend() {
        BACKEND_HEURISTIC => parse_with_heuristic_backend(input, options),
        BACKEND_SECTIONS => parse_with_sections_backend(input, options),
        other => unsupported_backend_report(other, options),
    }
}

fn parse_with_heuristic_backend(input: &str, options: &ParseOptions) -> ParseReport {
    let lines = significant_lines(input);
    let mut trace = Vec::new();

    trace.extend(options_trace(options));

    if lines.is_empty() {
        trace.push(event(FALLBACK_EMPTY, "input had no non-empty lines"));
        return ParseReport {
            kind: ParseKind::Lines,
            confidence: 1.0,
            columns: vec!["line".to_string()],
            rows: Vec::new(),
            trace,
        };
    }

    if let Some(candidate) = best_delimited_candidate(&lines, options) {
        trace.push(event(
            candidate.rule_id,
            format!(
                "selected delimiter {:?} with {} columns across {} rows",
                candidate.delimiter,
                candidate.columns,
                candidate.rows.len()
            ),
        ));
        return delimited_report(candidate, options.header_mode, trace);
    }

    if options.allows(FIXED_WIDTH_GAPS) {
        if let Some((columns, rows, header_trace)) = parse_fixed_width(&lines, options.header_mode)
        {
            trace.push(event(
                FIXED_WIDTH_GAPS,
                format!("selected fixed-width gaps with {} columns", columns.len()),
            ));
            trace.extend(header_trace);
            return ParseReport {
                kind: ParseKind::FixedWidth,
                confidence: 0.72,
                columns,
                rows,
                trace,
            };
        }
    } else {
        trace.push(event(
            "rule.skipped",
            format!("rule {FIXED_WIDTH_GAPS} skipped by options"),
        ));
    }

    trace.push(event(
        FALLBACK_LINES,
        "no tab, delimiter, or fixed-width table heuristic matched",
    ));
    lines_report(lines, trace)
}

pub fn known_rule_ids() -> &'static [&'static str] {
    &[
        DELIMITED_TABS,
        DELIMITED_COMMAS,
        DELIMITED_SEMICOLONS,
        DELIMITED_PIPES,
        FIXED_WIDTH_GAPS,
        FALLBACK_EMPTY,
        FALLBACK_LINES,
    ]
}

pub fn known_backend_ids() -> &'static [&'static str] {
    &[BACKEND_HEURISTIC, BACKEND_SECTIONS]
}

fn options_trace(options: &ParseOptions) -> Vec<TraceEvent> {
    let mut trace = options.trace_events.clone();

    if options.backend.explicit {
        trace.push(event(
            backend_rule_id(options.selected_backend()),
            format!("selected {} parser backend", options.selected_backend()),
        ));
    }

    for rule_id in &options.disabled_rules {
        trace.push(event(
            "options.disable-rule",
            format!("disabled heuristic rule {rule_id}"),
        ));
    }

    for rule_id in &options.only_rules {
        trace.push(event(
            "options.only-rule",
            format!("restricted parser selection to rule {rule_id}"),
        ));
    }

    trace
}

fn backend_rule_id(backend: &str) -> &'static str {
    match backend {
        BACKEND_HEURISTIC => "backend.heuristic",
        BACKEND_SECTIONS => BACKEND_SECTIONS_SELECTED,
        _ => "backend.unsupported",
    }
}

fn unsupported_backend_report(backend: &str, options: &ParseOptions) -> ParseReport {
    let mut trace = options.trace_events.clone();
    trace.push(event(
        "backend.unsupported",
        format!(
            "parser backend {backend} is not implemented; implemented backend(s): {}",
            known_backend_ids().join(", ")
        ),
    ));

    ParseReport {
        kind: ParseKind::Lines,
        confidence: 0.0,
        columns: vec!["line".to_string()],
        rows: Vec::new(),
        trace,
    }
}

fn parse_with_sections_backend(input: &str, options: &ParseOptions) -> ParseReport {
    let lines = significant_lines(input);
    let mut trace = options_trace(options);

    if lines.is_empty() {
        trace.push(event(FALLBACK_EMPTY, "input had no non-empty lines"));
        return ParseReport {
            kind: ParseKind::Sections,
            confidence: 1.0,
            columns: vec!["section".to_string()],
            rows: Vec::new(),
            trace,
        };
    }

    let Some((columns, rows)) = parse_sections(&lines, &mut trace) else {
        trace.push(event(
            FALLBACK_LINES,
            "section backend could not parse every non-empty line",
        ));
        return lines_report(lines, trace);
    };

    trace.push(event(
        BACKEND_SECTIONS_PARSED,
        format!("selected sections backend with {} row(s)", rows.len()),
    ));

    ParseReport {
        kind: ParseKind::Sections,
        confidence: 0.82,
        columns,
        rows,
        trace,
    }
}

fn parse_sections(
    lines: &[String],
    trace: &mut Vec<TraceEvent>,
) -> Option<(Vec<String>, Vec<Row>)> {
    let mut rows = Vec::new();
    let mut current: Option<Row> = None;
    let mut field_order = Vec::new();
    let mut seen_fields = BTreeSet::new();

    for line in lines {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix("section:") {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            let section = section.trim();
            if section.is_empty() {
                trace.push(event(
                    BACKEND_SECTIONS_MALFORMED,
                    "section header did not include a section name",
                ));
                return None;
            }
            current = Some(BTreeMap::from([(
                "section".to_string(),
                section.to_string(),
            )]));
            continue;
        }

        let Some(row) = current.as_mut() else {
            trace.push(event(
                BACKEND_SECTIONS_MALFORMED,
                format!("field appeared before a section header: {trimmed}"),
            ));
            return None;
        };
        let Some((key, value)) = trimmed.split_once(':') else {
            trace.push(event(
                BACKEND_SECTIONS_MALFORMED,
                format!("line was neither a section header nor key-value field: {trimmed}"),
            ));
            return None;
        };

        let key = normalize_key(key);
        if key.is_empty() {
            trace.push(event(
                BACKEND_SECTIONS_MALFORMED,
                format!("field did not include a key: {trimmed}"),
            ));
            return None;
        }
        if seen_fields.insert(key.clone()) {
            field_order.push(key.clone());
        }
        row.insert(key, value.trim().to_string());
    }

    if let Some(row) = current {
        rows.push(row);
    }

    if rows.is_empty() {
        trace.push(event(
            BACKEND_SECTIONS_MALFORMED,
            "input did not include any section headers",
        ));
        return None;
    }

    let mut columns = vec!["section".to_string()];
    columns.extend(field_order);
    for row in &mut rows {
        for column in &columns {
            row.entry(column.clone()).or_default();
        }
    }

    Some((columns, rows))
}

fn normalize_key(key: &str) -> String {
    key.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn significant_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn best_delimited_candidate(lines: &[String], options: &ParseOptions) -> Option<Candidate> {
    ['\t', ',', ';', '|']
        .into_iter()
        .filter_map(|delimiter| delimited_candidate(lines, delimiter, options))
        .max_by_key(|candidate| delimiter_rank(candidate.delimiter))
}

fn delimiter_rank(delimiter: char) -> u8 {
    match delimiter {
        '\t' => 4,
        '|' => 3,
        ',' => 2,
        ';' => 1,
        _ => 0,
    }
}

fn delimited_candidate(
    lines: &[String],
    delimiter: char,
    options: &ParseOptions,
) -> Option<Candidate> {
    let rule_id = delimiter_rule_id(delimiter);
    if !options.allows(rule_id) {
        return None;
    }

    let rows: Vec<Vec<String>> = lines
        .iter()
        .map(|line| split_delimited(line, delimiter))
        .collect();
    let columns = rows.first()?.len();

    if columns < 2 || !rows.iter().all(|row| row.len() == columns) {
        return None;
    }

    Some(Candidate {
        delimiter,
        rule_id,
        columns,
        rows: rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| cell.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect(),
    })
}

fn delimiter_rule_id(delimiter: char) -> &'static str {
    match delimiter {
        '\t' => DELIMITED_TABS,
        ',' => DELIMITED_COMMAS,
        ';' => DELIMITED_SEMICOLONS,
        '|' => DELIMITED_PIPES,
        _ => "detect.delimited.custom",
    }
}

fn split_delimited(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter).map(ToOwned::to_owned).collect()
}

fn delimited_report(
    candidate: Candidate,
    header_mode: HeaderMode,
    mut trace: Vec<TraceEvent>,
) -> ParseReport {
    let (columns, rows) =
        columns_and_rows(candidate.columns, candidate.rows, header_mode, &mut trace);
    ParseReport {
        kind: ParseKind::Delimited,
        confidence: if candidate.delimiter == '\t' {
            0.96
        } else {
            0.84
        },
        rows,
        columns,
        trace,
    }
}

fn parse_fixed_width(
    lines: &[String],
    header_mode: HeaderMode,
) -> Option<(Vec<String>, Vec<Row>, Vec<TraceEvent>)> {
    if lines.len() < 2 {
        return None;
    }

    let split_rows: Vec<Vec<String>> = lines
        .iter()
        .map(|line| split_on_repeated_spaces(line))
        .collect();
    let columns = split_rows.first()?.len();

    if columns < 2 || !split_rows.iter().all(|row| row.len() == columns) {
        return None;
    }

    let mut trace = Vec::new();
    let (column_names, rows) = columns_and_rows(columns, split_rows, header_mode, &mut trace);
    Some((column_names, rows, trace))
}

fn split_on_repeated_spaces(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut spaces = 0usize;

    for ch in line.chars() {
        if ch == ' ' {
            spaces += 1;
            if spaces < 2 {
                current.push(ch);
            }
            continue;
        }

        if spaces >= 2 && !current.trim().is_empty() {
            cells.push(current.trim().to_string());
            current.clear();
        }

        spaces = 0;
        current.push(ch);
    }

    if !current.trim().is_empty() {
        cells.push(current.trim().to_string());
    }

    cells
}

fn lines_report(lines: Vec<String>, trace: Vec<TraceEvent>) -> ParseReport {
    let columns = vec!["line".to_string()];
    let rows = lines
        .into_iter()
        .map(|line| BTreeMap::from([("line".to_string(), line)]))
        .collect();

    ParseReport {
        kind: ParseKind::Lines,
        confidence: 0.2,
        columns,
        rows,
        trace,
    }
}

fn columns_and_rows(
    count: usize,
    mut cell_rows: Vec<Vec<String>>,
    header_mode: HeaderMode,
    trace: &mut Vec<TraceEvent>,
) -> (Vec<String>, Vec<Row>) {
    let columns = match header_mode {
        HeaderMode::Generated => generated_columns(count),
        HeaderMode::FirstRow if cell_rows.len() > 1 => {
            trace.push(event(
                "headers.first-row",
                "used first parsed row as column names",
            ));
            normalize_headers(cell_rows.remove(0))
        }
        HeaderMode::FirstRow => {
            trace.push(event(
                "headers.first-row.unavailable",
                "first-row headers require at least one data row; generated column names instead",
            ));
            generated_columns(count)
        }
    };
    let rows = rows_from_cells(&columns, &cell_rows);
    (columns, rows)
}

fn normalize_headers(headers: Vec<String>) -> Vec<String> {
    headers
        .into_iter()
        .enumerate()
        .map(|(index, header)| {
            let normalized = header.trim().to_lowercase().replace(' ', "-");
            if normalized.is_empty() {
                format!("column{index}")
            } else {
                normalized
            }
        })
        .collect()
}

fn generated_columns(count: usize) -> Vec<String> {
    (0..count).map(|index| format!("column{index}")).collect()
}

fn rows_from_cells(columns: &[String], cell_rows: &[Vec<String>]) -> Vec<Row> {
    cell_rows
        .iter()
        .map(|cells| {
            columns
                .iter()
                .cloned()
                .zip(cells.iter().cloned())
                .collect::<BTreeMap<_, _>>()
        })
        .collect()
}

fn event(rule_id: &'static str, message: impl Into<String>) -> TraceEvent {
    TraceEvent {
        rule_id: rule_id.to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_tab_delimited_text() {
        let report = parse("alpha\tbeta\ngamma\tdelta\n");

        assert_eq!(report.kind, ParseKind::Delimited);
        assert_eq!(report.columns, vec!["column0", "column1"]);
        assert_eq!(report.rows[0]["column0"], "alpha");
        assert_eq!(report.rows[0]["column1"], "beta");
        assert_eq!(report.trace[0].rule_id, DELIMITED_TABS);
    }

    #[test]
    fn parses_fixed_width_text() {
        let report = parse("name     status\nalpha    ok\nbeta     fail\n");

        assert_eq!(report.kind, ParseKind::FixedWidth);
        assert_eq!(report.rows[1]["column0"], "alpha");
        assert_eq!(report.rows[2]["column1"], "fail");
    }

    #[test]
    fn falls_back_to_lines() {
        let report = parse("one thing\nanother thing\n");

        assert_eq!(report.kind, ParseKind::Lines);
        assert_eq!(report.rows[0]["line"], "one thing");
        assert_eq!(report.trace[0].rule_id, FALLBACK_LINES);
    }

    #[test]
    fn can_use_first_row_as_headers() {
        let options = ParseOptions::new().header_mode(HeaderMode::FirstRow);
        let report = parse_with_options("name\tstatus\nalpha\tok\n", &options);

        assert_eq!(report.columns, vec!["name", "status"]);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0]["name"], "alpha");
        assert_eq!(report.rows[0]["status"], "ok");
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "headers.first-row")
        );
    }

    #[test]
    fn disabled_rule_forces_next_safe_parser() {
        let options = ParseOptions::new().disable_rule(DELIMITED_TABS);
        let report = parse_with_options("alpha\tbeta\ngamma\tdelta\n", &options);

        assert_eq!(report.kind, ParseKind::Lines);
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "options.disable-rule")
        );
    }

    #[test]
    fn only_rule_limits_parser_selection() {
        let options = ParseOptions::new().only_rule(FIXED_WIDTH_GAPS);
        let report = parse_with_options("name     status\nalpha    ok\n", &options);

        assert_eq!(report.kind, ParseKind::FixedWidth);
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "options.only-rule")
        );
    }

    #[test]
    fn explicit_heuristic_backend_traces_backend_selection() {
        let options = ParseOptions::new().backend(BACKEND_HEURISTIC);
        let report = parse_with_options("alpha\tbeta\ngamma\tdelta\n", &options);

        assert_eq!(report.kind, ParseKind::Delimited);
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == "backend.heuristic")
        );
    }

    #[test]
    fn unsupported_backend_reports_diagnostic_without_parsing() {
        let options = ParseOptions::new().backend("tree-sitter");
        let report = parse_with_options("alpha\tbeta\ngamma\tdelta\n", &options);

        assert_eq!(report.kind, ParseKind::Lines);
        assert_eq!(report.confidence, 0.0);
        assert!(report.rows.is_empty());
        assert_eq!(report.trace[0].rule_id, "backend.unsupported");
    }

    #[test]
    fn sections_backend_parses_sectioned_key_values() {
        let options = ParseOptions::new().backend(BACKEND_SECTIONS);
        let report = parse_with_options(
            "section: api\n  status: ok\n  owner: platform\nsection: worker\n  status: degraded\n",
            &options,
        );

        assert_eq!(report.kind, ParseKind::Sections);
        assert_eq!(report.columns, vec!["section", "status", "owner"]);
        assert_eq!(report.rows[0]["section"], "api");
        assert_eq!(report.rows[0]["status"], "ok");
        assert_eq!(report.rows[0]["owner"], "platform");
        assert_eq!(report.rows[1]["section"], "worker");
        assert_eq!(report.rows[1]["owner"], "");
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == BACKEND_SECTIONS_PARSED)
        );
    }

    #[test]
    fn sections_backend_reports_malformed_input() {
        let options = ParseOptions::new().backend(BACKEND_SECTIONS);
        let report = parse_with_options("status: ok\nsection: api\n", &options);

        assert_eq!(report.kind, ParseKind::Lines);
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == BACKEND_SECTIONS_MALFORMED)
        );
    }

    proptest! {
        #[test]
        fn tabular_rows_preserve_rectangular_shape(rows in prop::collection::vec(prop::collection::vec("[a-z]{1,8}", 2..6), 1..20)) {
            let width = rows[0].len();
            let rows: Vec<Vec<String>> = rows.into_iter().filter(|row| row.len() == width).collect();
            prop_assume!(!rows.is_empty());
            let input = rows.iter().map(|row| row.join("\t")).collect::<Vec<_>>().join("\n");

            let report = parse(&input);

            prop_assert_eq!(report.kind, ParseKind::Delimited);
            prop_assert_eq!(report.columns.len(), width);
            prop_assert!(report.rows.iter().all(|row| row.len() == width));
        }

        #[test]
        fn sections_backend_preserves_section_count(names in prop::collection::vec("[a-z]{1,8}", 1..20)) {
            let input = names
                .iter()
                .map(|name| format!("section: {name}\n  status: ok"))
                .collect::<Vec<_>>()
                .join("\n");
            let options = ParseOptions::new().backend(BACKEND_SECTIONS);

            let report = parse_with_options(&input, &options);

            prop_assert_eq!(report.kind, ParseKind::Sections);
            prop_assert_eq!(report.rows.len(), names.len());
            prop_assert!(report.rows.iter().all(|row| row.contains_key("section")));
            prop_assert!(report.rows.iter().all(|row| row.contains_key("status")));
        }
    }
}
