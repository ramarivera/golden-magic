use assert_cmd::cargo::cargo_bin;
use std::process::Command;

#[test]
fn nu_wrapper_exports_all_from_aliases() {
    let binary = cargo_bin("golden-magic");

    for command in [
        "from golden-magic",
        "from gold",
        "from golden",
        "from magic",
        "from magia",
    ] {
        let output = Command::new("nu")
            .arg("--no-config-file")
            .arg("-c")
            .arg(format!(
                "use ./nu/golden-magic.nu *; 'name\tstatus\nalpha\tok\n' | {command} --binary '{}' --headers first-row | to nuon",
                binary.display()
            ))
            .output()
            .expect("nu executable is available");

        assert!(
            output.status.success(),
            "nu wrapper failed for {command}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "[[name, status]; [alpha, ok]]",
            "nu wrapper alias {command}"
        );
    }
}
