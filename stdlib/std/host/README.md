# `std/host`

This directory is the reading router for the current `std host I/O and text`
lane.

Keep the actual recipe sources in
[stdlib/std](/Users/Shared/chroot/dev/nuislang/stdlib/std) for now; this file
exists to give the lane a cluster-shaped front door before any higher-risk
filesystem reshuffle.

Canonical companions:

* cluster contract:
  [std-host-io-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-host-io-layering-contract.md)
* global `std` rule:
  [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* project companions:
  [examples/projects/tooling/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/README.md)
* source companions:
  [examples/ns/ffi/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)

## Current Lane Shape

Read the current lane in this order:

```text
raw output and input edge
-> input and terminal shaping
-> text formatting and json shaping
-> diagnostic and reporting edge
-> cli/session companions
```

Current rule:

* keep command/subprocess/workflow-specific routing in
  [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
* use this router for host reads, writes, text shaping, and report-facing
  helpers
* treat `line_input_recipe` as the effective narrow line-input pure layer even
  though there is no separate `line_input_runtime_recipe`

## Source Router

### Raw Output And Input Edge

* [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
* [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
* [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
* [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns)

### Input And Terminal Shaping

* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
* [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
* [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
* [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)

### Text Formatting And Json Shaping

* [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
* [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns)
* [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns)
* [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)

### Diagnostic And Reporting Edge

* [error_model_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_model_runtime_recipe.ns)
* [error_codes_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_codes_runtime_recipe.ns)
* [error_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime_recipe.ns)
* [result_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime_recipe.ns)
* [result_enum_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_enum_runtime_recipe.ns)
* [diagnostic_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime_recipe.ns)
* [result_diagnostic_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_diagnostic_recipe.ns)
* [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)

## Companion Validation Router

Use the FFI and project companions as grouped mirrors instead of browsing every
small host-facing probe first.

Shortest grouped route:

* source-level anchors:
  [hello_io_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns),
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns),
  [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns),
  [hello_host_text_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns),
  [hello_json_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_json_runtime_facades.ns)
* project-form anchors:
  [io_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/io_runtime_demo),
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/input_runtime_demo),
  [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/terminal_io_demo),
  [host_text_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/host_text_runtime_demo),
  [text_json_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_json_demo)

Wider grouped route:

* stdin and tty probes:
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/tty_runtime_demo)
* formatting and result/report probes:
  [text_format_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_format_runtime_demo),
  [error_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_runtime_demo),
  [diagnostic_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/diagnostic_runtime_demo),
  [result_diagnostic_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_diagnostic_demo)
* cli/session probes:
  [cli_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_session_demo),
  [cli_shell_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_shell_session_demo),
  [cli_report_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_report_session_demo),
  [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)

## Current Reading Rule

If you only want one pass:

1. start with [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
2. widen to [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
3. then read [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
4. end with [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)

Short rule:

* terminal edge first
* text/data shaping second
* report-facing aggregation last
* command/workflow branching belongs to the tooling lane
