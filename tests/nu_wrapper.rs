use assert_cmd::cargo::cargo_bin;
use std::process::Command;

#[test]
fn nu_wrapper_exports_from_golden_magic_command() {
    let binary = cargo_bin("golden-magic");
    let output = Command::new("nu")
        .arg("-c")
        .arg(format!(
            "use ./nu/golden-magic.nu *; 'name\tstatus\nalpha\tok\n' | from golden-magic --binary '{}' --headers first-row | to nuon",
            binary.display()
        ))
        .output()
        .expect("nu executable is available");

    assert!(
        output.status.success(),
        "nu wrapper failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "[[name, status]; [alpha, ok]]"
    );
}
