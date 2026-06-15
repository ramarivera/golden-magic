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
