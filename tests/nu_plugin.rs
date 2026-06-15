#![cfg(feature = "nu-plugin")]

use assert_cmd::cargo::cargo_bin;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[test]
fn native_nu_plugin_exports_all_from_aliases() {
    let Some(nu) = find_on_path("nu") else {
        eprintln!("skipping native Nu plugin test; nu is not available on PATH");
        return;
    };

    let plugin = cargo_bin("nu_plugin_golden_magic");
    let temp = tempdir().expect("temp plugin config dir");
    let plugin_config = temp.path().join("plugins.msgpackz");

    let add = Command::new(&nu)
        .arg("-c")
        .arg(format!(
            "plugin add --plugin-config {} {}",
            nu_string(plugin_config.to_string_lossy().as_ref()),
            nu_string(plugin.to_string_lossy().as_ref())
        ))
        .stdin(Stdio::null())
        .output()
        .expect("nu plugin add starts");

    assert!(
        add.status.success(),
        "plugin add failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&add.stdout),
        String::from_utf8_lossy(&add.stderr)
    );

    for command in [
        "from golden-magic",
        "from gold",
        "from golden",
        "from magic",
        "from magia",
    ] {
        let run = Command::new(&nu)
            .arg("--plugin-config")
            .arg(&plugin_config)
            .arg("-c")
            .arg(format!(
                "plugin use golden_magic; 'name\tstatus\nalpha\tok\n' | {command} --headers first-row | to json -r"
            ))
            .stdin(Stdio::null())
            .output()
            .expect("nu plugin command starts");

        assert!(
            run.status.success(),
            "plugin command failed for {command}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let rows: serde_json::Value =
            serde_json::from_slice(&run.stdout).expect("plugin emits JSON-serializable Nu rows");
        assert_eq!(rows[0]["name"], "alpha", "plugin alias {command}");
        assert_eq!(rows[0]["status"], "ok", "plugin alias {command}");
    }
}

fn find_on_path(binary: &str) -> Option<String> {
    std::env::var_os("PATH")?
        .to_string_lossy()
        .split(':')
        .find_map(|dir| {
            let path = std::path::Path::new(dir).join(binary);
            path.exists().then(|| path.to_string_lossy().into_owned())
        })
}

fn nu_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
