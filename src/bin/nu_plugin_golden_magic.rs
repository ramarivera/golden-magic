extern crate golden_magic;

use golden_magic::{HeaderMode, ParseOptions, parse_with_options};
use nu_plugin::{EvaluatedCall, JsonSerializer, Plugin, SimplePluginCommand, serve_plugin};
use nu_protocol::{
    Category, Example, LabeledError, ShellError, Signature, Span, SyntaxShape, Type, Value,
};

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
                SyntaxShape::String,
                "Disable a heuristic rule id; repeat by using the CLI wrapper for now",
                None,
            )
            .named(
                "only-rule",
                SyntaxShape::String,
                "Run only one heuristic rule id; repeat by using the CLI wrapper for now",
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
        let options = options_from_call(call)?;
        let report = parse_with_options(text, &options);
        let rows = report
            .rows
            .into_iter()
            .map(|row| row_to_value(row, span))
            .collect::<Vec<_>>();

        Ok(Value::list(rows, span))
    }
}

fn options_from_call(call: &EvaluatedCall) -> Result<ParseOptions, LabeledError> {
    let mut options = ParseOptions::new();

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

    if let Some(rule) = call.get_flag_value("disable-rule") {
        options = options.disable_rule(
            rule.as_str()
                .map_err(|error| labeled_error(error, call.head))?,
        );
    }

    if let Some(rule) = call.get_flag_value("only-rule") {
        options = options.only_rule(
            rule.as_str()
                .map_err(|error| labeled_error(error, call.head))?,
        );
    }

    Ok(options)
}

fn row_to_value(row: std::collections::BTreeMap<String, String>, span: Span) -> Value {
    let record = row
        .into_iter()
        .map(|(column, value)| (column, Value::string(value, span)))
        .collect();
    Value::record(record, span)
}

fn labeled_error(error: ShellError, span: Span) -> LabeledError {
    LabeledError::new("Golden Magic plugin error").with_label(error.to_string(), span)
}

fn main() {
    serve_plugin(&GoldenMagicPlugin, JsonSerializer)
}
