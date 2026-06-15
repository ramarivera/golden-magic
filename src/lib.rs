pub mod cli;
pub mod config;
pub mod descriptors;
pub mod tool_packs;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use wasmi::{Config as WasmConfig, Engine as WasmEngine, Linker as WasmLinker};

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
const BACKEND_TREE_SITTER: &str = "tree-sitter";
const BACKEND_TREE_SITTER_RUST: &str = "tree-sitter-rust";
const BACKEND_EXECUTABLE_JSON: &str = "executable-json";
const BACKEND_WASM_JSON: &str = "wasm-json";
const BACKEND_SECTIONS_SELECTED: &str = "backend.sections";
const BACKEND_SECTIONS_PARSED: &str = "backend.sections.parsed";
const BACKEND_SECTIONS_MALFORMED: &str = "backend.sections.malformed";
const BACKEND_TREE_SITTER_RUST_PARSED: &str = "backend.tree-sitter-rust.parsed";
const BACKEND_TREE_SITTER_RUST_ERROR: &str = "backend.tree-sitter-rust.error";
const BACKEND_EXECUTABLE_JSON_PARSED: &str = "backend.executable-json.parsed";
const BACKEND_EXECUTABLE_JSON_ERROR: &str = "backend.executable-json.error";
const BACKEND_WASM_JSON_PARSED: &str = "backend.wasm-json.parsed";
const BACKEND_WASM_JSON_ERROR: &str = "backend.wasm-json.error";
const EXECUTABLE_JSON_PROTOCOL: &str = "golden-magic.executable-json.v1";
const EXECUTABLE_JSON_TIMEOUT: Duration = Duration::from_secs(2);
const EXECUTABLE_JSON_MAX_STDOUT: usize = 1024 * 1024;
const EXECUTABLE_JSON_MAX_STDERR: usize = 64 * 1024;
const WASM_JSON_PROTOCOL: &str = "golden-magic.wasm-json.v1";
const WASM_JSON_INPUT_OFFSET: usize = 1024;
const WASM_JSON_MAX_INPUT: usize = 1024 * 1024;
const WASM_JSON_MAX_OUTPUT: usize = 1024 * 1024;
const WASM_JSON_FUEL: u64 = 100_000;

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
    TreeSitter,
    Plugin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub rule_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ExecutableJsonOutput {
    Envelope { protocol: String, rows: Vec<Row> },
    Rows(Vec<Row>),
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
    tree_sitter_grammar: Option<String>,
    tree_sitter_query: Option<PathBuf>,
    executable_plugin: Option<PathBuf>,
    wasm_module: Option<PathBuf>,
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

    pub fn tree_sitter_grammar(mut self, grammar: impl Into<String>) -> Self {
        self.tree_sitter_grammar = Some(grammar.into());
        self
    }

    pub fn tree_sitter_query(mut self, path: impl Into<PathBuf>) -> Self {
        self.tree_sitter_query = Some(path.into());
        self
    }

    pub fn executable_plugin(mut self, path: impl Into<PathBuf>) -> Self {
        self.executable_plugin = Some(path.into());
        self
    }

    pub fn wasm_module(mut self, path: impl Into<PathBuf>) -> Self {
        self.wasm_module = Some(path.into());
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

    pub fn selected_tree_sitter_grammar(&self) -> Option<&str> {
        self.tree_sitter_grammar.as_deref()
    }

    pub fn selected_tree_sitter_query(&self) -> Option<&PathBuf> {
        self.tree_sitter_query.as_ref()
    }

    pub fn selected_executable_plugin(&self) -> Option<&PathBuf> {
        self.executable_plugin.as_ref()
    }

    pub fn selected_wasm_module(&self) -> Option<&PathBuf> {
        self.wasm_module.as_ref()
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
        BACKEND_TREE_SITTER => parse_with_tree_sitter_backend(input, options),
        BACKEND_TREE_SITTER_RUST => parse_with_tree_sitter_rust_backend(input, options),
        BACKEND_EXECUTABLE_JSON => parse_with_executable_json_backend(input, options),
        BACKEND_WASM_JSON => parse_with_wasm_json_backend(input, options),
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
    &[
        BACKEND_HEURISTIC,
        BACKEND_SECTIONS,
        BACKEND_TREE_SITTER,
        BACKEND_TREE_SITTER_RUST,
        BACKEND_EXECUTABLE_JSON,
        BACKEND_WASM_JSON,
    ]
}

fn options_trace(options: &ParseOptions) -> Vec<TraceEvent> {
    let mut trace = options.trace_events.clone();

    if options.backend.explicit {
        trace.push(event(
            backend_rule_id(options.selected_backend()),
            format!("selected {} parser backend", options.selected_backend()),
        ));
    }

    if let Some(grammar) = options.selected_tree_sitter_grammar() {
        trace.push(event(
            "backend.tree-sitter.grammar",
            format!("selected tree-sitter grammar {grammar}"),
        ));
    }

    if let Some(query) = options.selected_tree_sitter_query() {
        trace.push(event(
            "backend.tree-sitter.query",
            format!("selected tree-sitter query {}", query.display()),
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
        BACKEND_TREE_SITTER => "backend.tree-sitter",
        BACKEND_TREE_SITTER_RUST => "backend.tree-sitter-rust",
        BACKEND_EXECUTABLE_JSON => "backend.executable-json",
        BACKEND_WASM_JSON => "backend.wasm-json",
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

fn parse_with_tree_sitter_backend(input: &str, options: &ParseOptions) -> ParseReport {
    match options.selected_tree_sitter_grammar() {
        Some("rust") => parse_with_tree_sitter_rust_backend(input, options),
        Some(grammar) => {
            let mut trace = options_trace(options);
            trace.push(event(
                BACKEND_TREE_SITTER_RUST_ERROR,
                format!(
                    "tree-sitter grammar {grammar} is not implemented; implemented grammar(s): rust"
                ),
            ));
            ParseReport {
                kind: ParseKind::TreeSitter,
                confidence: 0.0,
                columns: tree_sitter_declaration_columns(),
                rows: Vec::new(),
                trace,
            }
        }
        None => {
            let mut trace = options_trace(options);
            trace.push(event(
                BACKEND_TREE_SITTER_RUST_ERROR,
                "tree-sitter backend requires parser.grammar",
            ));
            ParseReport {
                kind: ParseKind::TreeSitter,
                confidence: 0.0,
                columns: tree_sitter_declaration_columns(),
                rows: Vec::new(),
                trace,
            }
        }
    }
}

fn parse_with_tree_sitter_rust_backend(input: &str, options: &ParseOptions) -> ParseReport {
    let mut trace = options_trace(options);
    let mut parser = tree_sitter::Parser::new();
    if let Err(error) = parser.set_language(&tree_sitter_rust::LANGUAGE.into()) {
        trace.push(event(
            BACKEND_TREE_SITTER_RUST_ERROR,
            format!("failed to load tree-sitter Rust grammar: {error}"),
        ));
        return ParseReport {
            kind: ParseKind::TreeSitter,
            confidence: 0.0,
            columns: tree_sitter_declaration_columns(),
            rows: Vec::new(),
            trace,
        };
    }

    let Some(tree) = parser.parse(input, None) else {
        trace.push(event(
            BACKEND_TREE_SITTER_RUST_ERROR,
            "tree-sitter Rust parser did not return a syntax tree",
        ));
        return ParseReport {
            kind: ParseKind::TreeSitter,
            confidence: 0.0,
            columns: tree_sitter_declaration_columns(),
            rows: Vec::new(),
            trace,
        };
    };

    let root = tree.root_node();
    if root.has_error() {
        trace.push(event(
            BACKEND_TREE_SITTER_RUST_ERROR,
            "tree-sitter Rust grammar reported syntax errors",
        ));
    }

    let mut rows = Vec::new();
    collect_rust_declarations(root, input.as_bytes(), &mut rows);

    trace.push(event(
        BACKEND_TREE_SITTER_RUST_PARSED,
        format!(
            "tree-sitter Rust backend extracted {} declaration row(s)",
            rows.len()
        ),
    ));

    ParseReport {
        kind: ParseKind::TreeSitter,
        confidence: if root.has_error() { 0.45 } else { 0.88 },
        columns: tree_sitter_declaration_columns(),
        rows,
        trace,
    }
}

fn parse_with_executable_json_backend(input: &str, options: &ParseOptions) -> ParseReport {
    let mut trace = options_trace(options);
    let Some(path) = options.selected_executable_plugin() else {
        trace.push(event(
            BACKEND_EXECUTABLE_JSON_ERROR,
            "executable-json backend requires a descriptor parser.executable path",
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    };

    let mut child = match Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            trace.push(event(
                BACKEND_EXECUTABLE_JSON_ERROR,
                format!(
                    "failed to start executable parser {}: {error}",
                    path.display()
                ),
            ));
            return plugin_report(Vec::new(), 0.0, trace);
        }
    };

    if let Some(mut stdin) = child.stdin.take()
        && let Err(error) = stdin.write_all(input.as_bytes())
    {
        trace.push(event(
            BACKEND_EXECUTABLE_JSON_ERROR,
            format!(
                "failed to send input to executable parser {}: {error}",
                path.display()
            ),
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    }

    let output = match wait_for_executable_json_output(&mut child) {
        Ok(output) => output,
        Err(error) => {
            trace.push(event(
                BACKEND_EXECUTABLE_JSON_ERROR,
                format!(
                    "failed to wait for executable parser {}: {error}",
                    path.display()
                ),
            ));
            return plugin_report(Vec::new(), 0.0, trace);
        }
    };

    if output.timed_out {
        trace.push(event(
            BACKEND_EXECUTABLE_JSON_ERROR,
            format!(
                "executable parser {} exceeded {:?} timeout",
                path.display(),
                EXECUTABLE_JSON_TIMEOUT
            ),
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    }

    if output.stdout_truncated {
        trace.push(event(
            BACKEND_EXECUTABLE_JSON_ERROR,
            format!(
                "executable parser {} exceeded stdout limit of {} byte(s)",
                path.display(),
                EXECUTABLE_JSON_MAX_STDOUT
            ),
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    }

    if !output.status_success {
        trace.push(event(
            BACKEND_EXECUTABLE_JSON_ERROR,
            format!(
                "executable parser {} exited unsuccessfully; stderr: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    }

    let rows = match parse_executable_json_output(&output.stdout) {
        Ok(rows) => rows,
        Err(error) => {
            trace.push(event(
                BACKEND_EXECUTABLE_JSON_ERROR,
                format!(
                    "executable parser {} did not emit a JSON array of row objects: {error}",
                    path.display()
                ),
            ));
            return plugin_report(Vec::new(), 0.0, trace);
        }
    };

    trace.push(event(
        BACKEND_EXECUTABLE_JSON_PARSED,
        format!(
            "executable parser {} emitted {} row(s)",
            path.display(),
            rows.len()
        ),
    ));

    plugin_report(rows, 0.78, trace)
}

fn parse_with_wasm_json_backend(input: &str, options: &ParseOptions) -> ParseReport {
    let mut trace = options_trace(options);
    let Some(path) = options.selected_wasm_module() else {
        trace.push(event(
            BACKEND_WASM_JSON_ERROR,
            "wasm-json backend requires a descriptor parser.module path",
        ));
        return plugin_report(Vec::new(), 0.0, trace);
    };

    let rows = match run_wasm_json_plugin(path, input) {
        Ok(rows) => rows,
        Err(error) => {
            trace.push(event(BACKEND_WASM_JSON_ERROR, error));
            return plugin_report(Vec::new(), 0.0, trace);
        }
    };

    trace.push(event(
        BACKEND_WASM_JSON_PARSED,
        format!(
            "wasm module {} emitted {} row(s)",
            path.display(),
            rows.len()
        ),
    ));

    plugin_report(rows, 0.8, trace)
}

fn run_wasm_json_plugin(path: &PathBuf, input: &str) -> Result<Vec<Row>, String> {
    if input.len() > WASM_JSON_MAX_INPUT {
        return Err(format!(
            "wasm parser input exceeded limit of {} byte(s)",
            WASM_JSON_MAX_INPUT
        ));
    }

    let bytes = std::fs::read(path)
        .map_err(|error| format!("failed to read wasm parser {}: {error}", path.display()))?;
    let mut config = WasmConfig::default();
    config.consume_fuel(true);
    let engine = WasmEngine::new(&config);
    let module = wasmi::Module::new(&engine, &bytes[..])
        .map_err(|error| format!("failed to compile wasm parser {}: {error}", path.display()))?;
    let linker = WasmLinker::new(&engine);
    let mut store = wasmi::Store::new(&engine, ());
    store
        .set_fuel(WASM_JSON_FUEL)
        .map_err(|error| format!("failed to configure wasm fuel: {error}"))?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| {
            format!(
                "failed to instantiate wasm parser {}: {error}",
                path.display()
            )
        })?;
    let memory = instance
        .get_memory(&store, "memory")
        .ok_or_else(|| "wasm parser must export memory named memory".to_string())?;
    let parse = instance
        .get_func(&store, "golden_magic_parse")
        .ok_or_else(|| "wasm parser must export function golden_magic_parse".to_string())?
        .typed::<(i32, i32), i64>(&store)
        .map_err(|error| {
            format!("wasm parser function golden_magic_parse has wrong signature: {error}")
        })?;

    let input_offset = WASM_JSON_INPUT_OFFSET;
    let input_end = input_offset + input.len();
    let memory_len = memory.data(&store).len();
    if input_end > memory_len {
        return Err(format!(
            "wasm parser memory is too small for input: need {input_end} byte(s), memory has {memory_len}"
        ));
    }
    memory.data_mut(&mut store)[input_offset..input_end].copy_from_slice(input.as_bytes());

    let packed = parse
        .call(&mut store, (input_offset as i32, input.len() as i32))
        .map_err(|error| format!("wasm parser trapped or exhausted fuel: {error}"))?;
    let output_offset = ((packed >> 32) & 0xffff_ffff) as usize;
    let output_len = (packed & 0xffff_ffff) as usize;
    if output_len > WASM_JSON_MAX_OUTPUT {
        return Err(format!(
            "wasm parser output exceeded limit of {} byte(s)",
            WASM_JSON_MAX_OUTPUT
        ));
    }
    let output_end = output_offset
        .checked_add(output_len)
        .ok_or_else(|| "wasm parser output pointer overflowed".to_string())?;
    let memory = memory.data(&store);
    if output_end > memory.len() {
        return Err(format!(
            "wasm parser output range {output_offset}..{output_end} exceeds memory size {}",
            memory.len()
        ));
    }

    parse_wasm_json_output(&memory[output_offset..output_end])
}

fn parse_wasm_json_output(bytes: &[u8]) -> Result<Vec<Row>, String> {
    match serde_json::from_slice::<ExecutableJsonOutput>(bytes) {
        Ok(ExecutableJsonOutput::Rows(rows)) => Ok(rows),
        Ok(ExecutableJsonOutput::Envelope { protocol, rows }) => {
            if protocol == WASM_JSON_PROTOCOL {
                Ok(rows)
            } else {
                Err(format!(
                    "unsupported wasm-json protocol {protocol}; expected {WASM_JSON_PROTOCOL}"
                ))
            }
        }
        Err(error) => Err(format!("wasm parser did not emit valid row JSON: {error}")),
    }
}

#[derive(Debug)]
struct ExecutableProcessOutput {
    status_success: bool,
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
}

fn wait_for_executable_json_output(
    child: &mut std::process::Child,
) -> Result<ExecutableProcessOutput, std::io::Error> {
    let mut stdout = child.stdout.take().expect("child stdout configured");
    let mut stderr = child.stderr.take().expect("child stderr configured");
    let stdout_reader =
        thread::spawn(move || read_limited(&mut stdout, EXECUTABLE_JSON_MAX_STDOUT));
    let stderr_reader =
        thread::spawn(move || read_limited(&mut stderr, EXECUTABLE_JSON_MAX_STDERR));
    let start = Instant::now();
    let mut timed_out = false;

    let status_success = loop {
        if let Some(status) = child.try_wait()? {
            break status.success();
        }
        if start.elapsed() >= EXECUTABLE_JSON_TIMEOUT {
            timed_out = true;
            let _ = child.kill();
            let status = child.wait()?;
            break status.success();
        }
        thread::sleep(Duration::from_millis(10));
    };

    let (stdout, stdout_truncated) = stdout_reader.join().expect("stdout reader thread joins")?;
    let (stderr, _) = stderr_reader.join().expect("stderr reader thread joins")?;

    Ok(ExecutableProcessOutput {
        status_success,
        timed_out,
        stdout,
        stderr,
        stdout_truncated,
    })
}

fn read_limited(reader: &mut impl Read, limit: usize) -> Result<(Vec<u8>, bool), std::io::Error> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut truncated = false;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(output.len());
        if read > remaining {
            output.extend_from_slice(&buffer[..remaining]);
            truncated = true;
            break;
        }
        output.extend_from_slice(&buffer[..read]);
    }

    Ok((output, truncated))
}

fn parse_executable_json_output(bytes: &[u8]) -> Result<Vec<Row>, String> {
    match serde_json::from_slice::<ExecutableJsonOutput>(bytes) {
        Ok(ExecutableJsonOutput::Rows(rows)) => Ok(rows),
        Ok(ExecutableJsonOutput::Envelope { protocol, rows }) => {
            if protocol == EXECUTABLE_JSON_PROTOCOL {
                Ok(rows)
            } else {
                Err(format!(
                    "unsupported executable-json protocol {protocol}; expected {EXECUTABLE_JSON_PROTOCOL}"
                ))
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

fn plugin_report(rows: Vec<Row>, confidence: f32, trace: Vec<TraceEvent>) -> ParseReport {
    let columns = rows
        .iter()
        .flat_map(|row| row.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    ParseReport {
        kind: ParseKind::Plugin,
        confidence,
        columns,
        rows,
        trace,
    }
}

fn tree_sitter_declaration_columns() -> Vec<String> {
    ["kind", "name", "start_line", "end_line"]
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
}

fn collect_rust_declarations(node: tree_sitter::Node, source: &[u8], rows: &mut Vec<Row>) {
    if let Some(kind) = rust_declaration_kind(node.kind())
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(source)
    {
        rows.push(BTreeMap::from([
            ("kind".to_string(), kind.to_string()),
            ("name".to_string(), name.to_string()),
            (
                "start_line".to_string(),
                (node.start_position().row + 1).to_string(),
            ),
            (
                "end_line".to_string(),
                (node.end_position().row + 1).to_string(),
            ),
        ]));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_rust_declarations(child, source, rows);
    }
}

fn rust_declaration_kind(node_kind: &str) -> Option<&'static str> {
    match node_kind {
        "function_item" => Some("function"),
        "mod_item" => Some("module"),
        "struct_item" => Some("struct"),
        _ => None,
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
        let options = ParseOptions::new().backend("tree-sitter-unknown");
        let report = parse_with_options("alpha\tbeta\ngamma\tdelta\n", &options);

        assert_eq!(report.kind, ParseKind::Lines);
        assert_eq!(report.confidence, 0.0);
        assert!(report.rows.is_empty());
        assert_eq!(report.trace[0].rule_id, "backend.unsupported");
    }

    #[test]
    fn tree_sitter_rust_backend_extracts_declarations() {
        let options = ParseOptions::new().backend(BACKEND_TREE_SITTER_RUST);
        let report = parse_with_options(
            "mod commands {\n  pub struct Tool;\n  pub fn run() {}\n}\nfn main() {}\n",
            &options,
        );

        assert_eq!(report.kind, ParseKind::TreeSitter);
        assert_eq!(report.columns, tree_sitter_declaration_columns());
        assert_eq!(report.rows[0]["kind"], "module");
        assert_eq!(report.rows[0]["name"], "commands");
        assert_eq!(report.rows[1]["kind"], "struct");
        assert_eq!(report.rows[1]["name"], "Tool");
        assert_eq!(report.rows[2]["kind"], "function");
        assert_eq!(report.rows[2]["name"], "run");
        assert_eq!(report.rows[3]["name"], "main");
        assert!(
            report
                .trace
                .iter()
                .any(|event| event.rule_id == BACKEND_TREE_SITTER_RUST_PARSED)
        );
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
