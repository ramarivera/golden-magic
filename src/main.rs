use clap::{Parser, ValueEnum};
use golden_magic::config::{Config, load_config};
use golden_magic::descriptors::{DescriptorRegistry, descriptor_rule_ids};
use golden_magic::{HeaderMode, ParseOptions, known_rule_ids, parse_with_options};
use std::io::{self, Read};
use std::path::PathBuf;

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

    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    no_default_descriptors: bool,

    #[arg(long)]
    explain: bool,

    #[arg(long)]
    list_rules: bool,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list_rules {
        for rule_id in known_rule_ids() {
            println!("{rule_id}");
        }
        return Ok(());
    }

    reject_unknown_rules(&args.disable_rules)?;
    reject_unknown_rules(&args.only_rules)?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let descriptor_dirs = descriptor_dirs(&args)?;
    let mut options = descriptor_options(&descriptor_dirs, &input)?;
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

fn descriptor_dirs(args: &Args) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut dirs = Vec::new();

    if !args.no_default_descriptors {
        let config = load_cli_config(args)?;
        if config.descriptor_dirs.is_empty() {
            dirs.extend(default_descriptor_dirs());
        } else {
            dirs.extend(config.descriptor_dirs);
        }
    }

    dirs.extend(args.descriptor_dirs.iter().cloned());
    Ok(dirs)
}

fn load_cli_config(args: &Args) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(path) = &args.config {
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

fn config_home() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

fn descriptor_options(
    descriptor_dirs: &[PathBuf],
    input: &str,
) -> Result<ParseOptions, Box<dyn std::error::Error>> {
    let mut options = ParseOptions::new();

    for dir in descriptor_dirs {
        let registry = DescriptorRegistry::load_dir(dir)?;
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

fn reject_unknown_rules(rules: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
