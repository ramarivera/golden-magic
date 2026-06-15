use crate::config::{Config, load_config};
use crate::descriptors::{DescriptorRegistry, descriptor_backend_id, descriptor_rule_ids};
use crate::tool_packs::{LoadedToolPack, load_tool_packs_dir};
use crate::{HeaderMode, ParseOptions, known_backend_ids, known_rule_ids, parse_with_options};
use clap::{Parser, ValueEnum};
use std::collections::BTreeSet;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(version, about = "Infer structured data from hostile CLI text")]
struct Args {
    #[arg(long, value_enum, default_value_t = OutputFormat::Report)]
    output: OutputFormat,

    #[arg(long = "disable-rule")]
    disable_rules: Vec<String>,

    #[arg(long = "only-rule", conflicts_with = "disable_rules")]
    only_rules: Vec<String>,

    #[arg(long, value_enum, default_value_t = CliHeaderMode::Generated)]
    headers: CliHeaderMode,

    #[arg(long = "descriptor-dir")]
    descriptor_dirs: Vec<PathBuf>,

    #[arg(long = "validate-descriptor-dir")]
    validate_descriptor_dirs: Vec<PathBuf>,

    #[arg(long = "tool-pack-dir")]
    tool_pack_dirs: Vec<PathBuf>,

    #[arg(long = "validate-tool-pack-dir")]
    validate_tool_pack_dirs: Vec<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    no_default_descriptors: bool,

    #[arg(long)]
    explain: bool,

    #[arg(long)]
    list_rules: bool,

    #[arg(long)]
    list_backends: bool,

    #[arg(long)]
    list_tool_packs: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    #[value(name = "report-json")]
    Report,
    #[value(name = "rows-json")]
    Rows,
    #[value(name = "trace-json")]
    Trace,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliHeaderMode {
    Generated,
    FirstRow,
}

impl From<CliHeaderMode> for HeaderMode {
    fn from(value: CliHeaderMode) -> Self {
        match value {
            CliHeaderMode::Generated => HeaderMode::Generated,
            CliHeaderMode::FirstRow => HeaderMode::FirstRow,
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list_rules {
        for rule_id in known_rule_ids() {
            println!("{rule_id}");
        }
        return Ok(());
    }

    if args.list_backends {
        for backend_id in known_backend_ids() {
            println!("{backend_id}");
        }
        return Ok(());
    }

    if !args.validate_descriptor_dirs.is_empty() {
        validate_descriptor_dirs(&args.validate_descriptor_dirs)?;
        return Ok(());
    }

    if !args.validate_tool_pack_dirs.is_empty() {
        validate_tool_pack_dirs(
            &args.validate_tool_pack_dirs,
            &args.descriptor_dirs,
            args.config.as_deref(),
            args.no_default_descriptors,
        )?;
        return Ok(());
    }

    if args.list_tool_packs {
        list_tool_packs(
            &args.tool_pack_dirs,
            &args.descriptor_dirs,
            args.config.as_deref(),
            args.no_default_descriptors,
        )?;
        return Ok(());
    }

    reject_unknown_rules(&args.disable_rules)?;
    reject_unknown_rules(&args.only_rules)?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut options = parser_options_from_descriptors(
        &input,
        &args.descriptor_dirs,
        args.config.as_deref(),
        args.no_default_descriptors,
    )?;
    for rule in &args.disable_rules {
        options = options.disable_rule(rule);
    }
    for rule in &args.only_rules {
        options = options.only_rule(rule);
    }
    options = options.header_mode(args.headers.into());

    let report = parse_with_options(&input, &options);
    let output = if args.explain {
        OutputFormat::Trace
    } else {
        args.output
    };

    match output {
        OutputFormat::Report => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Rows => println!("{}", serde_json::to_string_pretty(&report.rows)?),
        OutputFormat::Trace => println!("{}", serde_json::to_string_pretty(&report.trace)?),
    }

    Ok(())
}

pub fn parser_options_from_descriptors(
    input: &str,
    extra_descriptor_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<ParseOptions, Box<dyn std::error::Error>> {
    let descriptor_dirs =
        descriptor_dirs(extra_descriptor_dirs, config_path, no_default_descriptors)?;
    descriptor_options(&descriptor_dirs, input)
}

fn descriptor_dirs(
    extra_descriptor_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut dirs = Vec::new();

    if !no_default_descriptors {
        let config = load_cli_config(config_path)?;
        if config.descriptor_dirs.is_empty() {
            dirs.extend(default_descriptor_dirs());
        } else {
            dirs.extend(config.descriptor_dirs);
        }
    }

    dirs.extend(extra_descriptor_dirs.iter().cloned());
    Ok(dirs)
}

fn tool_pack_dirs(
    extra_tool_pack_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut dirs = Vec::new();

    if !no_default_descriptors {
        let config = load_cli_config(config_path)?;
        if config.tool_pack_dirs.is_empty() {
            dirs.extend(default_tool_pack_dirs());
        } else {
            dirs.extend(config.tool_pack_dirs);
        }
    }

    dirs.extend(extra_tool_pack_dirs.iter().cloned());
    Ok(dirs)
}

fn load_cli_config(config_path: Option<&Path>) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(path) = config_path {
        return Ok(load_config(path)?);
    }

    let Some(path) = default_config_path() else {
        return Ok(Config::default());
    };

    if path.exists() {
        Ok(load_config(path)?)
    } else {
        Ok(Config::default())
    }
}

fn default_config_path() -> Option<PathBuf> {
    config_home().map(|config_home| config_home.join("golden-magic").join("config.toml"))
}

fn default_descriptor_dirs() -> Vec<PathBuf> {
    let Some(config_home) = config_home() else {
        return Vec::new();
    };

    vec![config_home.join("golden-magic").join("descriptors")]
}

fn default_tool_pack_dirs() -> Vec<PathBuf> {
    let Some(config_home) = config_home() else {
        return Vec::new();
    };

    vec![config_home.join("golden-magic").join("tool-packs")]
}

fn config_home() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

pub fn load_tool_packs(
    extra_tool_pack_dirs: &[PathBuf],
    extra_descriptor_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<Vec<LoadedToolPack>, Box<dyn std::error::Error>> {
    let descriptor_dirs =
        descriptor_dirs(extra_descriptor_dirs, config_path, no_default_descriptors)?;
    let descriptor_ids = descriptor_ids(&descriptor_dirs)?;
    let mut packs = Vec::new();

    for dir in tool_pack_dirs(extra_tool_pack_dirs, config_path, no_default_descriptors)? {
        packs.extend(load_tool_packs_dir(dir)?);
    }

    reject_unknown_tool_pack_descriptors(&packs, &descriptor_ids)?;
    Ok(packs)
}

fn list_tool_packs(
    extra_tool_pack_dirs: &[PathBuf],
    extra_descriptor_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    for loaded in load_tool_packs(
        extra_tool_pack_dirs,
        extra_descriptor_dirs,
        config_path,
        no_default_descriptors,
    )? {
        println!(
            "{}\t{}\t{}",
            loaded.pack.id,
            loaded.pack.name,
            loaded.path.display()
        );
    }

    Ok(())
}

fn validate_tool_pack_dirs(
    dirs: &[PathBuf],
    extra_descriptor_dirs: &[PathBuf],
    config_path: Option<&Path>,
    no_default_descriptors: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_dirs =
        descriptor_dirs(extra_descriptor_dirs, config_path, no_default_descriptors)?;
    let descriptor_ids = descriptor_ids(&descriptor_dirs)?;
    let mut total = 0usize;

    for dir in dirs {
        let packs = load_tool_packs_dir(dir)?;
        reject_unknown_tool_pack_descriptors(&packs, &descriptor_ids)?;
        let count = packs.len();
        total += count;
        println!("validated {count} tool pack(s) from {}", dir.display());
    }

    println!("validated {total} tool pack(s) total");
    Ok(())
}

fn descriptor_ids(dirs: &[PathBuf]) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut ids = BTreeSet::new();

    for dir in dirs {
        let registry = DescriptorRegistry::load_dir(dir)?;
        reject_unknown_descriptor_backends(&registry)?;
        reject_unknown_descriptor_rules(&registry)?;
        ids.extend(
            registry
                .descriptors()
                .iter()
                .map(|loaded| loaded.descriptor.id.clone()),
        );
    }

    Ok(ids)
}

fn reject_unknown_tool_pack_descriptors(
    packs: &[LoadedToolPack],
    descriptor_ids: &BTreeSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut unknown = Vec::new();

    for loaded in packs {
        for descriptor in loaded.pack.descriptor_refs() {
            if !descriptor_ids.contains(descriptor) {
                unknown.push(format!(
                    "{} references unknown descriptor {}",
                    loaded.pack.id, descriptor
                ));
            }
        }
    }

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "tool pack descriptor reference error(s): {}",
            unknown.join(", ")
        )
        .into())
    }
}

fn descriptor_options(
    descriptor_dirs: &[PathBuf],
    input: &str,
) -> Result<ParseOptions, Box<dyn std::error::Error>> {
    let mut options = ParseOptions::new();

    for dir in descriptor_dirs {
        let registry = DescriptorRegistry::load_dir(dir)?;
        reject_unknown_descriptor_backends(&registry)?;
        reject_unknown_descriptor_rules(&registry)?;
        let selected = registry.select(input);
        if let Some(loaded) = selected.first() {
            options = options.trace_event(
                "descriptor.selected",
                format!(
                    "selected descriptor {} from {}",
                    loaded.descriptor.id,
                    loaded.path.display()
                ),
            );
            options = apply_descriptor_backend(options, loaded)?;
            for rule in &loaded.descriptor.parser.disable_rules {
                options = options.disable_rule(rule);
            }
            for rule in &loaded.descriptor.parser.only_rules {
                options = options.only_rule(rule);
            }
            break;
        }
    }

    Ok(options)
}

fn validate_descriptor_dirs(dirs: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let mut total = 0usize;

    for dir in dirs {
        let registry = DescriptorRegistry::load_dir(dir)?;
        reject_unknown_descriptor_backends(&registry)?;
        reject_unknown_descriptor_rules(&registry)?;
        let count = registry.descriptors().len();
        total += count;
        println!("validated {count} descriptor(s) from {}", dir.display());
    }

    println!("validated {total} descriptor(s) total");
    Ok(())
}

fn apply_descriptor_backend(
    options: ParseOptions,
    loaded: &crate::descriptors::LoadedDescriptor,
) -> Result<ParseOptions, Box<dyn std::error::Error>> {
    let Some(backend) = descriptor_backend_id(&loaded.descriptor) else {
        return Ok(options);
    };

    if known_backend_ids().iter().any(|known| known == &backend) {
        Ok(options.backend(backend))
    } else {
        Err(format!(
            "descriptor {} requests parser backend {backend}, but only {} is implemented. See docs/PARSER-BACKENDS.md.",
            loaded.descriptor.id,
            known_backend_ids().join(", ")
        )
        .into())
    }
}

fn reject_unknown_descriptor_backends(
    registry: &DescriptorRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    let known = known_backend_ids();
    let unknown: Vec<String> = registry
        .descriptors()
        .iter()
        .filter_map(|loaded| {
            descriptor_backend_id(&loaded.descriptor)
                .filter(|backend| !known.iter().any(|known_backend| known_backend == backend))
                .map(ToOwned::to_owned)
        })
        .collect();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(format!(
        "descriptor contains unknown or unsupported parser backend(s): {}. Implemented backend(s): {}.",
        unknown.join(", "),
        known.join(", ")
    )
    .into())
}

fn reject_unknown_descriptor_rules(
    registry: &DescriptorRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    let known = known_rule_ids();
    let unknown: Vec<String> = registry
        .descriptors()
        .iter()
        .flat_map(|loaded| descriptor_rule_ids(&loaded.descriptor))
        .filter(|rule| !known.iter().any(|known_rule| known_rule == rule))
        .map(ToOwned::to_owned)
        .collect();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(format!(
        "descriptor contains unknown rule id(s): {}. Run `golden-magic --list-rules` to inspect available rules.",
        unknown.join(", ")
    )
    .into())
}

pub fn reject_unknown_rules(rules: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let known = known_rule_ids();
    let unknown: Vec<&String> = rules
        .iter()
        .filter(|rule| !known.iter().any(|known_rule| known_rule == &rule.as_str()))
        .collect();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(format!(
        "unknown rule id(s): {}. Run `golden-magic --list-rules` to inspect available rules.",
        unknown
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    )
    .into())
}
