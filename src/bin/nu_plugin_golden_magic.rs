extern crate golden_magic;

use golden_magic::cli::{load_tool_packs, parser_options_from_descriptors, reject_unknown_rules};
use golden_magic::tool_packs::LoadedToolPack;
use golden_magic::{HeaderMode, ParseOptions, parse_with_options};
use nu_plugin::{EvaluatedCall, JsonSerializer, Plugin, SimplePluginCommand, serve_plugin};
use nu_protocol::{
    Category, Example, LabeledError, ShellError, Signature, Span, SyntaxShape, Type, Value,
};
use std::path::PathBuf;

struct GoldenMagicPlugin;
struct FromGoldenMagic {
    name: &'static str,
}

impl Plugin for GoldenMagicPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        [
            "from golden-magic",
            "from gold",
            "from golden",
            "from magic",
            "from magia",
        ]
        .into_iter()
        .map(|name| {
            Box::new(FromGoldenMagic { name }) as Box<dyn nu_plugin::PluginCommand<Plugin = Self>>
        })
        .collect()
    }
}

impl SimplePluginCommand for FromGoldenMagic {
    type Plugin = GoldenMagicPlugin;

    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Parse hostile table-ish text into Nushell records using Golden Magic heuristics"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .category(Category::Formats)
            .input_output_type(Type::String, Type::List(Type::Record([].into()).into()))
            .named(
                "headers",
                SyntaxShape::String,
                "Header mode: generated or first-row",
                None,
            )
            .named(
                "disable-rule",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Disable heuristic rule ids",
                None,
            )
            .named(
                "only-rule",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Run only these heuristic rule ids",
                None,
            )
            .named(
                "descriptor-dir",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Descriptor directories to load after default/config descriptors",
                None,
            )
            .named(
                "config",
                SyntaxShape::Filepath,
                "Config file with descriptor_dirs overrides",
                None,
            )
            .named(
                "tool-pack-dir",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Declarative tool-pack directories to load and validate",
                None,
            )
            .switch(
                "list-tool-packs",
                "Return loaded declarative tool packs instead of parsing input rows",
                None,
            )
            .switch(
                "no-default-descriptors",
                "Disable XDG/default descriptor and config discovery",
                None,
            )
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: "'name\tstatus\nalpha\tok\n' | from golden-magic --headers first-row",
            description: "Parse tab-delimited text with first-row headers",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &GoldenMagicPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let span = input.span();
        let text = input
            .as_str()
            .map_err(|error| labeled_error(error, call.head))?;
        let loaded_tool_packs = tool_packs_from_call(call)?;
        if has_switch(call, "list-tool-packs") {
            return Ok(tool_packs_to_value(&loaded_tool_packs, span));
        }

        let options = options_from_call(call, text)?;
        let report = parse_with_options(text, &options);
        let rows = report
            .rows
            .into_iter()
            .map(|row| row_to_value(row, span))
            .collect::<Vec<_>>();

        Ok(Value::list(rows, span))
    }
}

fn tool_packs_from_call(call: &EvaluatedCall) -> Result<Vec<LoadedToolPack>, LabeledError> {
    let tool_pack_dirs = string_list_flag(call, "tool-pack-dir")?
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let descriptor_dirs = string_list_flag(call, "descriptor-dir")?
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let config_path = string_flag(call, "config")?.map(PathBuf::from);
    let no_default_descriptors = has_switch(call, "no-default-descriptors");

    load_tool_packs(
        &tool_pack_dirs,
        &descriptor_dirs,
        config_path.as_deref(),
        no_default_descriptors,
    )
    .map_err(|error| {
        LabeledError::new("Golden Magic tool-pack error").with_label(error.to_string(), call.head)
    })
}

fn options_from_call(call: &EvaluatedCall, input: &str) -> Result<ParseOptions, LabeledError> {
    let descriptor_dirs = string_list_flag(call, "descriptor-dir")?
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let config_path = string_flag(call, "config")?.map(PathBuf::from);
    let no_default_descriptors = has_switch(call, "no-default-descriptors");

    let mut options = parser_options_from_descriptors(
        input,
        &descriptor_dirs,
        config_path.as_deref(),
        no_default_descriptors,
    )
    .map_err(|error| {
        LabeledError::new("Golden Magic descriptor error").with_label(error.to_string(), call.head)
    })?;

    if let Some(headers) = call.get_flag_value("headers") {
        let mode = headers
            .as_str()
            .map_err(|error| labeled_error(error, call.head))?;
        options = match mode {
            "generated" => options.header_mode(HeaderMode::Generated),
            "first-row" => options.header_mode(HeaderMode::FirstRow),
            other => {
                return Err(LabeledError::new("Invalid headers mode").with_label(
                    format!("expected generated or first-row, got {other}"),
                    call.head,
                ));
            }
        };
    }

    let disabled_rules = string_list_flag(call, "disable-rule")?;
    reject_unknown_rules(&disabled_rules).map_err(|error| {
        LabeledError::new("Golden Magic rule error").with_label(error.to_string(), call.head)
    })?;
    for rule in disabled_rules {
        options = options.disable_rule(rule);
    }

    let only_rules = string_list_flag(call, "only-rule")?;
    reject_unknown_rules(&only_rules).map_err(|error| {
        LabeledError::new("Golden Magic rule error").with_label(error.to_string(), call.head)
    })?;
    for rule in only_rules {
        options = options.only_rule(rule);
    }

    Ok(options)
}

fn has_switch(call: &EvaluatedCall, name: &str) -> bool {
    call.named
        .iter()
        .any(|(flag, value)| flag.item == name && value.is_none())
}

fn string_flag(call: &EvaluatedCall, name: &str) -> Result<Option<String>, LabeledError> {
    call.get_flag_value(name)
        .map(|value| {
            value
                .coerce_string()
                .map_err(|error| labeled_error(error, call.head))
        })
        .transpose()
}

fn string_list_flag(call: &EvaluatedCall, name: &str) -> Result<Vec<String>, LabeledError> {
    let Some(value) = call.get_flag_value(name) else {
        return Ok(Vec::new());
    };

    value
        .as_list()
        .map_err(|error| labeled_error(error, call.head))?
        .iter()
        .map(|value| {
            value
                .coerce_string()
                .map_err(|error| labeled_error(error, call.head))
        })
        .collect()
}

fn row_to_value(row: std::collections::BTreeMap<String, String>, span: Span) -> Value {
    let record = row
        .into_iter()
        .map(|(column, value)| (column, Value::string(value, span)))
        .collect();
    Value::record(record, span)
}

fn tool_packs_to_value(packs: &[LoadedToolPack], span: Span) -> Value {
    let rows = packs
        .iter()
        .map(|loaded| {
            Value::record(
                [
                    ("id".to_string(), Value::string(&loaded.pack.id, span)),
                    ("name".to_string(), Value::string(&loaded.pack.name, span)),
                    (
                        "version".to_string(),
                        Value::string(&loaded.pack.version, span),
                    ),
                    (
                        "path".to_string(),
                        Value::string(loaded.path.display().to_string(), span),
                    ),
                    (
                        "descriptors".to_string(),
                        Value::list(
                            loaded
                                .pack
                                .descriptor_refs()
                                .into_iter()
                                .map(|descriptor| Value::string(descriptor, span))
                                .collect(),
                            span,
                        ),
                    ),
                ]
                .into_iter()
                .collect(),
                span,
            )
        })
        .collect();

    Value::list(rows, span)
}

fn labeled_error(error: ShellError, span: Span) -> LabeledError {
    LabeledError::new("Golden Magic plugin error").with_label(error.to_string(), span)
}

fn main() {
    serve_plugin(&GoldenMagicPlugin, JsonSerializer)
}
