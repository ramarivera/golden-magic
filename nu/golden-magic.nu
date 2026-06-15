# Nushell wrapper for the Golden Magic CLI.
#
# This gives users the intended spell shape today:
#
#   ^some-cli | from golden-magic
#   ^some-cli | from gold
#   ^some-cli | from golden
#   ^some-cli | from magic
#   ^some-cli | from magia
#
# The implementation shells out to the Rust CLI. The native Nu plugin exposes
# the same command aliases when built with the `nu-plugin` Cargo feature.

def build-golden-magic-args [
    headers: string
    report: bool
    trace: bool
    disable_rule: list<string>
    only_rule: list<string>
    descriptor_dir: list<string>
    no_default_descriptors: bool
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

    $args
}

export def "from golden-magic" [
    --binary: string = "golden-magic"
    --headers: string = "generated"
    --report
    --trace
    --disable-rule: list<string> = []
    --only-rule: list<string> = []
    --descriptor-dir: list<string> = []
    --no-default-descriptors
] {
    let args = (build-golden-magic-args $headers $report $trace $disable_rule $only_rule $descriptor_dir $no_default_descriptors)
    $in | ^$binary ...$args | from json
}

export def "from gold" [
    --binary: string = "golden-magic"
    --headers: string = "generated"
    --report
    --trace
    --disable-rule: list<string> = []
    --only-rule: list<string> = []
    --descriptor-dir: list<string> = []
    --no-default-descriptors
] {
    let args = (build-golden-magic-args $headers $report $trace $disable_rule $only_rule $descriptor_dir $no_default_descriptors)
    $in | ^$binary ...$args | from json
}

export def "from golden" [
    --binary: string = "golden-magic"
    --headers: string = "generated"
    --report
    --trace
    --disable-rule: list<string> = []
    --only-rule: list<string> = []
    --descriptor-dir: list<string> = []
    --no-default-descriptors
] {
    let args = (build-golden-magic-args $headers $report $trace $disable_rule $only_rule $descriptor_dir $no_default_descriptors)
    $in | ^$binary ...$args | from json
}

export def "from magic" [
    --binary: string = "golden-magic"
    --headers: string = "generated"
    --report
    --trace
    --disable-rule: list<string> = []
    --only-rule: list<string> = []
    --descriptor-dir: list<string> = []
    --no-default-descriptors
] {
    let args = (build-golden-magic-args $headers $report $trace $disable_rule $only_rule $descriptor_dir $no_default_descriptors)
    $in | ^$binary ...$args | from json
}

export def "from magia" [
    --binary: string = "golden-magic"
    --headers: string = "generated"
    --report
    --trace
    --disable-rule: list<string> = []
    --only-rule: list<string> = []
    --descriptor-dir: list<string> = []
    --no-default-descriptors
] {
    let args = (build-golden-magic-args $headers $report $trace $disable_rule $only_rule $descriptor_dir $no_default_descriptors)
    $in | ^$binary ...$args | from json
}
