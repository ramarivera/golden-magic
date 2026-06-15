use assert_cmd::cargo::cargo_bin;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn nix_manifest_fixtures_run_cli_without_system_installs_when_enabled() {
    if std::env::var_os("GOLDEN_MAGIC_RUN_NIX_FIXTURES").is_none() {
        eprintln!("skipping nix fixture; set GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 to enable");
        return;
    }

    if Command::new("nix").arg("--version").output().is_err() {
        eprintln!("skipping nix fixture; nix is not available on PATH");
        return;
    }

    let golden_magic = cargo_bin("golden-magic");
    let fixtures = nix_fixtures();
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

fn run_nix_fixture(golden_magic: &Path, fixture: &Path) {
    let manifest_path = fixture.join("nix.toml");
    let manifest_text = fs::read_to_string(&manifest_path).expect("nix manifest reads");
    let manifest: NixFixture = toml::from_str(&manifest_text).expect("nix manifest parses");
    assert!(
        !manifest.packages.is_empty(),
        "fixture {fixture:?} must declare at least one Nix package"
    );

    let expected_path = fixture.join(&manifest.expected_rows);
    let expected: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&expected_path).expect("expected rows fixture exists"),
    )
    .expect("expected rows JSON parses");

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
        "nix fixture {fixture:?} failed\nstatus: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("golden-magic emits JSON rows");
    assert_eq!(actual, expected, "fixture {fixture:?} rows differ");
}

fn fixture_script(golden_magic: &Path, fixture: &Path, manifest: &NixFixture) -> String {
    let mut parser_args = vec![
        "--no-default-descriptors".to_string(),
        "--descriptor-dir".to_string(),
        fixture.display().to_string(),
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

fn nix_fixtures() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/descriptors");
    fs::read_dir(root)
        .expect("descriptor fixture root exists")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.join("nix.toml").exists())
        .collect()
}

fn default_expected_rows() -> String {
    "expected.rows.json".to_string()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
