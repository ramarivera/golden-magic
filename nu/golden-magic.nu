# Nushell wrapper for the Golden Magic CLI.
#
# This gives users the intended spell shape today:
#
#   ^some-cli | from golden-magic
#
# The implementation shells out to the Rust CLI for now. A future native Nu
# plugin can keep this surface area and replace the adapter internals.

export def "from golden-magic" [
    --binary: string = "golden-magic" # Golden Magic executable path.
    --headers: string = "generated" # Header mode: generated or first-row.
    --report # Return the full parse report instead of rows only.
    --trace # Return trace events instead of rows only.
    --disable-rule: list<string> = [] # Heuristic rule ids to disable.
    --only-rule: list<string> = [] # Restrict parser selection to these rule ids.
    --descriptor-dir: list<string> = [] # Descriptor directories to load.
    --no-default-descriptors # Do not load XDG default descriptors.
] {
    let output = if $report {
        "report-json"
    } else if $trace {
        "trace-json"
    } else {
        "rows-json"
    }

    mut args = ["--output" $output "--headers" $headers]

    if $no_default_descriptors {
        $args = ($args | append "--no-default-descriptors")
    }

    for dir in $descriptor_dir {
        $args = ($args | append ["--descriptor-dir" $dir])
    }

    for rule in $disable_rule {
        $args = ($args | append ["--disable-rule" $rule])
    }

    for rule in $only_rule {
        $args = ($args | append ["--only-rule" $rule])
    }

    $in | ^$binary ...$args | from json
}
