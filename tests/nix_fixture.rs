mod support;

use assert_cmd::cargo::cargo_bin;
use serde::Deserialize;
use std::path::Path;
use std::process::{Command, Stdio};
use support::{DescriptorFixture, shell_quote};

#[test]
fn nix_manifest_fixtures_run_cli_without_system_installs_when_enabled() {
    if std::env::var_os("GOLDEN_MAGIC_RUN_NIX_FIXTURES").is_none() {
        eprintln!("skipping nix fixture; set GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 to enable");
        return;
    }

    let nix_version = Command::new("nix")
        .arg("--version")
        .output()
        .expect("GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 requires nix on PATH");
    assert!(
        nix_version.status.success(),
        "GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 requires a working nix binary\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&nix_version.stdout),
        String::from_utf8_lossy(&nix_version.stderr)
    );

    let golden_magic = cargo_bin("golden-magic");
    let fixtures = DescriptorFixture::with_nix_manifest();
    assert!(
        !fixtures.is_empty(),
        "at least one descriptor fixture must include nix.toml"
    );

    for fixture in fixtures {
        run_nix_fixture(&golden_magic, &fixture);
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NixFixture {
    packages: Vec<String>,
    command: String,
    #[serde(default = "default_expected_rows")]
    expected_rows: String,
    #[serde(default)]
    parser_args: Vec<String>,
}

fn run_nix_fixture(golden_magic: &Path, fixture: &DescriptorFixture) {
    let manifest: NixFixture = fixture.read_toml("nix.toml");
    assert!(
        !manifest.packages.is_empty(),
        "fixture {:?} must declare at least one Nix package",
        fixture.path()
    );

    let expected: serde_json::Value = fixture.read_json(&manifest.expected_rows);

    let script = fixture_script(golden_magic, fixture, &manifest);
    let mut command = Command::new("nix");
    command.arg("shell");
    for package in &manifest.packages {
        command.arg(package);
    }
    command.args([
        "--option",
        "extra-experimental-features",
        "nix-command flakes",
        "--command",
        "sh",
        "-c",
        &script,
    ]);

    let output = command
        .stdin(Stdio::null())
        .output()
        .expect("nix shell command starts");

    assert!(
        output.status.success(),
        "nix fixture {:?} failed\nstatus: {}\nstdout: {}\nstderr: {}",
        fixture.path(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("golden-magic emits JSON rows");
    assert_eq!(actual, expected, "fixture {:?} rows differ", fixture.path());
}

fn fixture_script(
    golden_magic: &Path,
    fixture: &DescriptorFixture,
    manifest: &NixFixture,
) -> String {
    let mut parser_args = vec![
        "--no-default-descriptors".to_string(),
        "--descriptor-dir".to_string(),
        fixture.path().display().to_string(),
        "--output".to_string(),
        "rows-json".to_string(),
    ];
    parser_args.extend(manifest.parser_args.clone());

    format!(
        "{} | {} {}",
        manifest.command,
        shell_quote(golden_magic.to_string_lossy().as_ref()),
        parser_args
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn default_expected_rows() -> String {
    "expected.rows.json".to_string()
}
