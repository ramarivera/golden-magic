use assert_cmd::cargo::cargo_bin;
use std::process::{Command, Stdio};

#[test]
fn nix_fixture_runs_cli_without_system_install_when_enabled() {
    if std::env::var_os("GOLDEN_MAGIC_RUN_NIX_FIXTURES").is_none() {
        eprintln!("skipping nix fixture; set GOLDEN_MAGIC_RUN_NIX_FIXTURES=1 to enable");
        return;
    }

    if Command::new("nix").arg("--version").output().is_err() {
        eprintln!("skipping nix fixture; nix is not available on PATH");
        return;
    }

    let golden_magic = cargo_bin("golden-magic");
    let script = format!(
        "printf 'name\\tstatus\\nalpha\\tok\\n' | {} --headers first-row --output rows-json",
        shell_quote(golden_magic.to_string_lossy().as_ref())
    );

    let output = Command::new("nix")
        .args([
            "shell",
            "nixpkgs#coreutils",
            "--option",
            "extra-experimental-features",
            "nix-command flakes",
            "--command",
            "sh",
            "-c",
            &script,
        ])
        .stdin(Stdio::null())
        .output()
        .expect("nix shell command starts");

    assert!(
        output.status.success(),
        "nix fixture failed\nstatus: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rows: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("golden-magic emits JSON rows");
    assert_eq!(rows[0]["name"], "alpha");
    assert_eq!(rows[0]["status"], "ok");
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
