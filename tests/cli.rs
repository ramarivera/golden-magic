use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn cli_alias_binaries_parse_rows() {
    for binary in ["golden-magic", "gold", "golden", "magic", "magia"] {
        let output = Command::cargo_bin(binary)
            .expect("binary exists")
            .arg("--no-default-descriptors")
            .arg("--output")
            .arg("rows-json")
            .arg("--headers")
            .arg("first-row")
            .write_stdin("name\tstatus\nalpha\tok\n")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");
        assert_eq!(rows[0]["name"], "alpha", "binary alias {binary}");
        assert_eq!(rows[0]["status"], "ok", "binary alias {binary}");
    }
}

#[test]
fn cli_parses_tabular_stdin_as_json_report() {
    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .write_stdin("alpha\tbeta\ngamma\tdelta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).expect("valid JSON report");

    assert_eq!(report["kind"], "delimited");
    assert_eq!(report["columns"], serde_json::json!(["column0", "column1"]));
    assert_eq!(report["rows"][1]["column1"], "delta");
    assert_eq!(report["trace"][0]["rule_id"], "detect.delimited.tabs");
}

#[test]
fn cli_can_emit_rows_only_for_nushell_friendly_pipelines() {
    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--output")
        .arg("rows-json")
        .write_stdin("alpha\tbeta\ngamma\tdelta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");

    assert_eq!(rows[0]["column0"], "alpha");
    assert_eq!(rows[1]["column1"], "delta");
    assert!(
        rows.as_array()
            .expect("array rows")
            .iter()
            .all(|row| row.get("kind").is_none())
    );
}

#[test]
fn cli_can_emit_trace_only() {
    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--output")
        .arg("trace-json")
        .write_stdin("alpha\tbeta\ngamma\tdelta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let trace: Value = serde_json::from_slice(&output).expect("valid JSON trace");

    assert_eq!(trace[0]["rule_id"], "detect.delimited.tabs");
}

#[test]
fn cli_can_use_first_row_as_headers() {
    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--output")
        .arg("rows-json")
        .arg("--headers")
        .arg("first-row")
        .write_stdin("name\tstatus\nalpha\tok\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");

    assert_eq!(rows[0]["name"], "alpha");
    assert_eq!(rows[0]["status"], "ok");
    assert!(rows[0].get("column0").is_none());
}

#[test]
fn cli_applies_selected_descriptor_parser_hints() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("pipes.toml"),
        r#"
id = "generic.pipes"
name = "Generic Pipes"
priority = 10
[matches]
required_substrings = ["|"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .write_stdin("alpha|beta\ngamma|delta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).expect("valid JSON report");

    assert_eq!(report["kind"], "delimited");
    assert!(
        report["trace"]
            .as_array()
            .expect("trace array")
            .iter()
            .any(|event| event["rule_id"] == "descriptor.selected")
    );
    assert!(
        report["trace"]
            .as_array()
            .expect("trace array")
            .iter()
            .any(|event| event["rule_id"] == "options.only-rule")
    );
    assert!(
        report["trace"]
            .as_array()
            .expect("trace array")
            .iter()
            .any(|event| event["rule_id"] == "detect.delimited.pipes")
    );
}

#[test]
fn cli_applies_selected_descriptor_backend() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("pipes.toml"),
        r#"
id = "backend.pipes"
name = "Backend Pipes"
priority = 10
[matches]
required_substrings = ["|"]
[parser]
backend = "heuristic"
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .arg("--output")
        .arg("trace-json")
        .write_stdin("alpha|beta\ngamma|delta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let trace: Value = serde_json::from_slice(&output).expect("valid JSON trace");

    assert!(
        trace
            .as_array()
            .expect("trace array")
            .iter()
            .any(|event| event["rule_id"] == "backend.heuristic")
    );
}

#[test]
fn cli_rejects_unsupported_descriptor_backend() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("unsupported.toml"),
        r#"
id = "backend.unsupported"
name = "Backend Unsupported"
[parser]
backend = "definitely-not-real"
"#,
    )
    .expect("write descriptor");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--validate-descriptor-dir")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "descriptor contains unknown or unsupported parser backend(s): definitely-not-real",
        ));
}

#[test]
fn cli_applies_sections_descriptor_backend() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("sections.toml"),
        r#"
id = "backend.sections"
name = "Backend Sections"
priority = 10
[matches]
required_substrings = ["section:", "status:"]
[parser]
backend = "sections"
"#,
    )
    .expect("write descriptor");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .arg("--output")
        .arg("rows-json")
        .write_stdin("section: api\n  status: ok\nsection: worker\n  status: degraded\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");

    assert_eq!(rows[0]["section"], "api");
    assert_eq!(rows[0]["status"], "ok");
    assert_eq!(rows[1]["section"], "worker");
    assert_eq!(rows[1]["status"], "degraded");
}

#[test]
fn cli_applies_tree_sitter_rust_descriptor_backend() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("rust.toml"),
        r#"
id = "backend.tree-sitter-rust"
name = "Backend Tree Sitter Rust"
priority = 10
[matches]
required_substrings = ["fn ", "struct "]
[parser]
backend = "tree-sitter-rust"
"#,
    )
    .expect("write descriptor");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .arg("--output")
        .arg("rows-json")
        .write_stdin("struct Tool;\nfn run() {}\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");
    assert_eq!(rows[0]["kind"], "struct");
    assert_eq!(rows[0]["name"], "Tool");
    assert_eq!(rows[1]["kind"], "function");
    assert_eq!(rows[1]["name"], "run");
}

fn toml_string(value: String) -> String {
    serde_json::to_string(&value).expect("path string serializes")
}

#[test]
fn cli_rejects_unknown_descriptor_rule_ids() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("bad.toml"),
        r#"
id = "bad"
name = "Bad"
[parser]
only_rules = ["detect.nope"]
"#,
    )
    .expect("write descriptor");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .write_stdin("alpha\tbeta\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "descriptor contains unknown rule id(s): detect.nope",
        ));
}

#[test]
fn cli_validates_descriptor_dir_without_stdin() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("pipes.toml"),
        r#"
id = "sdk.pipes"
name = "SDK Pipes"
priority = 10
[matches]
required_substrings = ["|"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--validate-descriptor-dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("validated 1 descriptor(s)"))
        .stdout(predicates::str::contains("validated 1 descriptor(s) total"));
}

#[test]
fn cli_descriptor_validation_rejects_unknown_rule_ids() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("bad.toml"),
        r#"
id = "sdk.bad"
name = "SDK Bad"
[parser]
only_rules = ["detect.nope"]
"#,
    )
    .expect("write descriptor");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--validate-descriptor-dir")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "descriptor contains unknown rule id(s): detect.nope",
        ));
}

#[test]
fn cli_validates_tool_pack_descriptor_references() {
    let dir = tempdir().expect("temp dir");
    let descriptor_dir = dir.path().join("descriptors");
    let tool_pack_dir = dir.path().join("tool-packs/git");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::create_dir_all(&tool_pack_dir).expect("create tool-pack dir");
    fs::write(
        descriptor_dir.join("branches.toml"),
        r#"
id = "known.git.branch-verbose"
name = "Git branch verbose"
"#,
    )
    .expect("write descriptor");
    fs::write(
        tool_pack_dir.join("tool.toml"),
        r#"
id = "tool.git"
name = "git"
version = "1"

[[commands]]
name = "branch"

[[commands.subcommands]]
name = "--verbose"
descriptor = "known.git.branch-verbose"
patterns = ["git branch -v"]
"#,
    )
    .expect("write tool pack");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(&descriptor_dir)
        .arg("--validate-tool-pack-dir")
        .arg(dir.path().join("tool-packs"))
        .assert()
        .success()
        .stdout(predicates::str::contains("validated 1 tool pack(s)"))
        .stdout(predicates::str::contains("validated 1 tool pack(s) total"));
}

#[test]
fn cli_rejects_tool_pack_missing_descriptor_reference() {
    let dir = tempdir().expect("temp dir");
    let tool_pack_dir = dir.path().join("tool-packs/git");
    fs::create_dir_all(&tool_pack_dir).expect("create tool-pack dir");
    fs::write(
        tool_pack_dir.join("tool.toml"),
        r#"
id = "tool.git"
name = "git"
version = "1"

[[commands]]
name = "branch"
descriptor = "known.git.branch-verbose"
"#,
    )
    .expect("write tool pack");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--validate-tool-pack-dir")
        .arg(dir.path().join("tool-packs"))
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "tool.git references unknown descriptor known.git.branch-verbose",
        ));
}

#[test]
fn cli_lists_tool_packs_from_configured_dirs() {
    let dir = tempdir().expect("temp dir");
    let descriptor_dir = dir.path().join("descriptors");
    let tool_pack_dir = dir.path().join("tool-packs/git");
    let config_path = dir.path().join("config.toml");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::create_dir_all(&tool_pack_dir).expect("create tool-pack dir");
    fs::write(
        descriptor_dir.join("branches.toml"),
        r#"
id = "known.git.branch-verbose"
name = "Git branch verbose"
"#,
    )
    .expect("write descriptor");
    fs::write(
        tool_pack_dir.join("tool.toml"),
        r#"
id = "tool.git"
name = "git"
version = "1"

[[commands]]
name = "branch"
descriptor = "known.git.branch-verbose"
"#,
    )
    .expect("write tool pack");
    fs::write(
        &config_path,
        format!(
            "descriptor_dirs = [{}]\ntool_pack_dirs = [{}]\n",
            toml_string(descriptor_dir.display().to_string()),
            toml_string(dir.path().join("tool-packs").display().to_string())
        ),
    )
    .expect("write config");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--config")
        .arg(config_path)
        .arg("--list-tool-packs")
        .assert()
        .success()
        .stdout(predicates::str::contains("tool.git\tgit\t"));
}

#[test]
fn cli_loads_default_xdg_descriptor_dir() {
    let config_home = tempdir().expect("temp config home");
    let descriptor_dir = config_home.path().join("golden-magic").join("descriptors");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::write(
        descriptor_dir.join("pipes.toml"),
        r#"
id = "xdg.pipes"
name = "XDG Pipes"
priority = 10
[matches]
required_substrings = ["|"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .env("XDG_CONFIG_HOME", config_home.path())
        .write_stdin("alpha|beta\ngamma|delta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).expect("valid JSON report");

    assert!(
        report["trace"]
            .as_array()
            .expect("trace array")
            .iter()
            .any(|event| event["rule_id"] == "descriptor.selected")
    );
}

#[test]
fn cli_config_descriptor_dirs_override_default_descriptor_dir() {
    let config_home = tempdir().expect("temp config home");
    let default_descriptor_dir = config_home.path().join("golden-magic").join("descriptors");
    let configured_descriptor_dir = config_home.path().join("custom-descriptors");
    fs::create_dir_all(&default_descriptor_dir).expect("create default descriptor dir");
    fs::create_dir_all(&configured_descriptor_dir).expect("create configured descriptor dir");
    fs::write(
        default_descriptor_dir.join("default.toml"),
        r#"
id = "default"
name = "Default"
priority = 100
[matches]
required_substrings = ["|"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write default descriptor");
    fs::write(
        configured_descriptor_dir.join("configured.toml"),
        r#"
id = "configured"
name = "Configured"
priority = 1
[matches]
required_substrings = ["custom"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write configured descriptor");
    fs::write(
        config_home.path().join("golden-magic").join("config.toml"),
        format!(
            "descriptor_dirs = [{}]\n",
            toml::Value::String(configured_descriptor_dir.display().to_string())
        ),
    )
    .expect("write config");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .env("XDG_CONFIG_HOME", config_home.path())
        .write_stdin("custom|beta\ngamma|delta\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).expect("valid JSON report");
    let selected = report["trace"]
        .as_array()
        .expect("trace array")
        .iter()
        .find(|event| event["rule_id"] == "descriptor.selected")
        .expect("descriptor selected");

    assert!(
        selected["message"]
            .as_str()
            .expect("message string")
            .contains("configured")
    );
}

#[test]
fn cli_rejects_unknown_rule_ids() {
    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--disable-rule")
        .arg("detect.nope")
        .write_stdin("alpha\tbeta\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown rule id(s): detect.nope"));
}

#[test]
fn cli_lists_known_rule_ids() {
    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--list-rules")
        .assert()
        .success()
        .stdout(predicates::str::contains("detect.delimited.tabs"))
        .stdout(predicates::str::contains("detect.fixed-width.gaps"));
}

#[test]
fn cli_lists_known_backend_ids() {
    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--list-backends")
        .assert()
        .success()
        .stdout(predicates::str::contains("heuristic"))
        .stdout(predicates::str::contains("sections"))
        .stdout(predicates::str::contains("executable-json"));
}

#[test]
fn cli_runs_descriptor_selected_executable_json_backend() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/descriptors/executable-json");

    let output = Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(&fixture)
        .arg("--output")
        .arg("rows-json")
        .write_stdin(fs::read_to_string(fixture.join("input.txt")).expect("fixture input"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rows: Value = serde_json::from_slice(&output).expect("valid JSON rows");
    assert_eq!(rows[0]["name"], "alpha");
    assert_eq!(rows[0]["status"], "ok");
    assert_eq!(rows[1]["name"], "beta");
    assert_eq!(rows[1]["status"], "degraded");
}

#[test]
fn cli_rejects_executable_json_descriptor_without_executable() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join("missing-executable.toml"),
        r#"
id = "plugin.missing-executable"
name = "Missing Executable"
priority = 10
[matches]
required_substrings = ["plugin-row:"]
[parser]
backend = "executable-json"
"#,
    )
    .expect("write descriptor");

    Command::cargo_bin("golden-magic")
        .expect("binary exists")
        .arg("--no-default-descriptors")
        .arg("--descriptor-dir")
        .arg(dir.path())
        .write_stdin("plugin-row: alpha ok\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "uses executable-json backend but parser.executable is missing",
        ));
}
