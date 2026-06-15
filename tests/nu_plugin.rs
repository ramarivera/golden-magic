#![cfg(feature = "nu-plugin")]

use assert_cmd::cargo::cargo_bin;
use std::fs;
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

#[test]
fn native_nu_plugin_applies_descriptor_dir_parser_hints() {
    let Some(nu) = find_on_path("nu") else {
        eprintln!("skipping native Nu plugin descriptor test; nu is not available on PATH");
        return;
    };

    let plugin = cargo_bin("nu_plugin_golden_magic");
    let temp = tempdir().expect("temp plugin config dir");
    let plugin_config = temp.path().join("plugins.msgpackz");
    let descriptor_dir = temp.path().join("descriptors");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::write(
        descriptor_dir.join("pipes.toml"),
        r#"
id = "nu-plugin.pipes"
name = "Nu Plugin Pipes"
priority = 10
[matches]
required_substrings = ["|"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");

    add_plugin(&nu, &plugin_config, &plugin);

    let run = Command::new(&nu)
        .arg("--plugin-config")
        .arg(&plugin_config)
        .arg("-c")
        .arg(format!(
            "plugin use golden_magic; 'alpha|beta\ngamma|delta\n' | from golden-magic --descriptor-dir [{}] | to json -r",
            nu_string(descriptor_dir.to_string_lossy().as_ref())
        ))
        .stdin(Stdio::null())
        .output()
        .expect("nu plugin command starts");

    assert!(
        run.status.success(),
        "plugin descriptor command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let rows: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("plugin emits JSON-serializable Nu rows");
    assert_eq!(rows[0]["column0"], "alpha");
    assert_eq!(rows[0]["column1"], "beta");
}

#[test]
fn native_nu_plugin_applies_config_descriptor_dirs() {
    let Some(nu) = find_on_path("nu") else {
        eprintln!("skipping native Nu plugin config test; nu is not available on PATH");
        return;
    };

    let plugin = cargo_bin("nu_plugin_golden_magic");
    let temp = tempdir().expect("temp plugin config dir");
    let plugin_config = temp.path().join("plugins.msgpackz");
    let descriptor_dir = temp.path().join("configured-descriptors");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::write(
        descriptor_dir.join("pipes.toml"),
        r#"
id = "nu-plugin.configured-pipes"
name = "Nu Plugin Configured Pipes"
priority = 10
[matches]
required_substrings = ["configured"]
[parser]
only_rules = ["detect.delimited.pipes"]
"#,
    )
    .expect("write descriptor");
    let config = temp.path().join("golden-magic.toml");
    fs::write(
        &config,
        format!(
            "descriptor_dirs = [{}]\n",
            toml::Value::String(descriptor_dir.display().to_string())
        ),
    )
    .expect("write config");

    add_plugin(&nu, &plugin_config, &plugin);

    let run = Command::new(&nu)
        .arg("--plugin-config")
        .arg(&plugin_config)
        .arg("-c")
        .arg(format!(
            "plugin use golden_magic; 'configured|beta\ngamma|delta\n' | from golden-magic --config {} | to json -r",
            nu_string(config.to_string_lossy().as_ref())
        ))
        .stdin(Stdio::null())
        .output()
        .expect("nu plugin command starts");

    assert!(
        run.status.success(),
        "plugin config command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let rows: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("plugin emits JSON-serializable Nu rows");
    assert_eq!(rows[0]["column0"], "configured");
    assert_eq!(rows[0]["column1"], "beta");
}

#[test]
fn native_nu_plugin_lists_tool_packs_with_descriptor_validation() {
    let Some(nu) = find_on_path("nu") else {
        eprintln!("skipping native Nu plugin tool-pack test; nu is not available on PATH");
        return;
    };

    let plugin = cargo_bin("nu_plugin_golden_magic");
    let temp = tempdir().expect("temp plugin config dir");
    let plugin_config = temp.path().join("plugins.msgpackz");
    let descriptor_dir = temp.path().join("descriptors");
    let tool_pack_dir = temp.path().join("tool-packs");
    fs::create_dir_all(&descriptor_dir).expect("create descriptor dir");
    fs::create_dir_all(&tool_pack_dir).expect("create tool-pack dir");
    fs::write(
        descriptor_dir.join("git-branch.toml"),
        r#"
id = "known.git.branch-verbose"
name = "Git Branch Verbose"
[matches]
required_substrings = ["branch"]
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

    add_plugin(&nu, &plugin_config, &plugin);

    let run = Command::new(&nu)
        .arg("--plugin-config")
        .arg(&plugin_config)
        .arg("-c")
        .arg(format!(
            "plugin use golden_magic; '' | from golden-magic --no-default-descriptors --descriptor-dir [{}] --tool-pack-dir [{}] --list-tool-packs | to json -r",
            nu_string(descriptor_dir.to_string_lossy().as_ref()),
            nu_string(tool_pack_dir.to_string_lossy().as_ref())
        ))
        .stdin(Stdio::null())
        .output()
        .expect("nu plugin command starts");

    assert!(
        run.status.success(),
        "plugin tool-pack command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let rows: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("plugin emits JSON-serializable Nu rows");
    assert_eq!(rows[0]["id"], "tool.git");
    assert_eq!(rows[0]["name"], "git");
    assert_eq!(rows[0]["descriptors"][0], "known.git.branch-verbose");
}

#[test]
fn native_nu_plugin_rejects_tool_packs_with_unknown_descriptors() {
    let Some(nu) = find_on_path("nu") else {
        eprintln!(
            "skipping native Nu plugin tool-pack validation test; nu is not available on PATH"
        );
        return;
    };

    let plugin = cargo_bin("nu_plugin_golden_magic");
    let temp = tempdir().expect("temp plugin config dir");
    let plugin_config = temp.path().join("plugins.msgpackz");
    let tool_pack_dir = temp.path().join("tool-packs");
    fs::create_dir_all(&tool_pack_dir).expect("create tool-pack dir");
    fs::write(
        tool_pack_dir.join("tool.toml"),
        r#"
id = "tool.bad"
name = "bad"
version = "1"

[[commands]]
name = "bad"
descriptor = "known.missing"
"#,
    )
    .expect("write tool pack");

    add_plugin(&nu, &plugin_config, &plugin);

    let run = Command::new(&nu)
        .arg("--plugin-config")
        .arg(&plugin_config)
        .arg("-c")
        .arg(format!(
            "plugin use golden_magic; '' | from golden-magic --no-default-descriptors --tool-pack-dir [{}] --list-tool-packs",
            nu_string(tool_pack_dir.to_string_lossy().as_ref())
        ))
        .stdin(Stdio::null())
        .output()
        .expect("nu plugin command starts");

    assert!(
        !run.status.success(),
        "plugin should reject unknown descriptor refs\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("known.missing"),
        "stderr should mention missing descriptor\nstderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
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

fn add_plugin(nu: &str, plugin_config: &std::path::Path, plugin: &std::path::Path) {
    let add = Command::new(nu)
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
}
