# `std/host`

This directory is the reading router for the current `std host I/O and text`
lane.

Keep the actual recipe sources in
[stdlib/std](../../../stdlib/std) for now; this file
exists to give the lane a cluster-shaped front door before any higher-risk
filesystem reshuffle.

Canonical companions:

* cluster contract:
  [std-host-io-layering-contract.md](../../../docs/reference/std-host-io-layering-contract.md)
* auto-injected text/json helper surface:
  [lib/text_contracts.ns](../../../stdlib/std/lib/text_contracts.ns)
* global `std` rule:
  [std-mainline-layering-contract.md](../../../docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [current-mainline-map.md](../../../docs/current-mainline-map.md)
* project companions:
  [examples/projects/tooling/README.md](../../../examples/projects/tooling/README.md)
* source companions:
  [examples/ns/ffi/README.md](../../../examples/ns/ffi/README.md)

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
  [tooling/README.md](../../../stdlib/std/tooling/README.md)
* use this router for host reads, writes, text shaping, and report-facing
  helpers
* shared text/json probe summaries should live in `StdTextContracts`, not be
  re-encoded independently in every text or CLI recipe
* treat `line_input_recipe` as the effective narrow line-input pure layer even
  though there is no separate `line_input_runtime_recipe`

## Source Router

### Raw Output And Input Edge

* [io_runtime_recipe.ns](../../../stdlib/std/io_runtime_recipe.ns)
* [stdin_runtime_recipe.ns](../../../stdlib/std/stdin_runtime_recipe.ns)
* [tty_runtime_recipe.ns](../../../stdlib/std/tty_runtime_recipe.ns)
* [line_input_recipe.ns](../../../stdlib/std/line_input_recipe.ns)

### Input And Terminal Shaping

* [input_runtime_recipe.ns](../../../stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](../../../stdlib/std/terminal_io_recipe.ns)
* [cli_session_recipe.ns](../../../stdlib/std/cli_session_recipe.ns)
* [cli_shell_session_recipe.ns](../../../stdlib/std/cli_shell_session_recipe.ns)
* [cli_report_session_recipe.ns](../../../stdlib/std/cli_report_session_recipe.ns)
* [cli_runtime_recipe.ns](../../../stdlib/std/cli_runtime_recipe.ns)

### Text Formatting And Json Shaping

* [host_text_runtime_recipe.ns](../../../stdlib/std/host_text_runtime_recipe.ns)
* [text_format_runtime_recipe.ns](../../../stdlib/std/text_format_runtime_recipe.ns)
* [json_runtime_recipe.ns](../../../stdlib/std/json_runtime_recipe.ns)
* [text_pipeline_recipe.ns](../../../stdlib/std/text_pipeline_recipe.ns)
* [text_report_builder_recipe.ns](../../../stdlib/std/text_report_builder_recipe.ns)
* [text_report_json_recipe.ns](../../../stdlib/std/text_report_json_recipe.ns)
* [time_report_recipe.ns](../../../stdlib/std/time_report_recipe.ns)
* [benchmark_report_recipe.ns](../../../stdlib/std/benchmark_report_recipe.ns)
* [benchmark_report_count_recipe.ns](../../../stdlib/std/benchmark_report_count_recipe.ns)
* [benchmark_report_file_recipe.ns](../../../stdlib/std/benchmark_report_file_recipe.ns)
* [io_report_recipe.ns](../../../stdlib/std/io_report_recipe.ns)
* cross-lane closure:
  [filesystem_report_recipe.ns](../../../stdlib/std/filesystem_report_recipe.ns)
  ->
  [filesystem_io_report_recipe.ns](../../../stdlib/std/filesystem_io_report_recipe.ns)
  ->
  [filesystem_report_file_recipe.ns](../../../stdlib/std/filesystem_report_file_recipe.ns)
* [text_json_recipe.ns](../../../stdlib/std/text_json_recipe.ns)

### Diagnostic And Reporting Edge

* [error_model_runtime_recipe.ns](../../../stdlib/std/error_model_runtime_recipe.ns)
* [error_bridge_runtime_recipe.ns](../../../stdlib/std/error_bridge_runtime_recipe.ns)
* [error_codes_runtime_recipe.ns](../../../stdlib/std/error_codes_runtime_recipe.ns)
* [error_runtime_recipe.ns](../../../stdlib/std/error_runtime_recipe.ns)
* [result_runtime_recipe.ns](../../../stdlib/std/result_runtime_recipe.ns)
* [result_enum_runtime_recipe.ns](../../../stdlib/std/result_enum_runtime_recipe.ns)
* [diagnostic_runtime_recipe.ns](../../../stdlib/std/diagnostic_runtime_recipe.ns)
* [result_diagnostic_recipe.ns](../../../stdlib/std/result_diagnostic_recipe.ns)
* [task_result_enum_recipe.ns](../../../stdlib/std/task_result_enum_recipe.ns)
* [report_runtime_recipe.ns](../../../stdlib/std/report_runtime_recipe.ns)
* [net_result_enum_recipe.ns](../../../stdlib/std/net_result_enum_recipe.ns)
* [shader_result_enum_recipe.ns](../../../stdlib/std/shader_result_enum_recipe.ns)
* shared contract:
  [std-result-bridge-contract.md](../../../docs/reference/std-result-bridge-contract.md)

## Companion Validation Router

Use the FFI and project companions as grouped mirrors instead of browsing every
small host-facing probe first.

Shortest grouped route:

* source-level anchors:
  [hello_io_runtime_facades.ns](../../../examples/ns/ffi/hello_io_runtime_facades.ns),
  [hello_input_runtime_facades.ns](../../../examples/ns/ffi/hello_input_runtime_facades.ns),
  [hello_terminal_io_facades.ns](../../../examples/ns/ffi/hello_terminal_io_facades.ns),
  [hello_host_text_runtime_facades.ns](../../../examples/ns/ffi/hello_host_text_runtime_facades.ns),
  [hello_json_runtime_facades.ns](../../../examples/ns/ffi/hello_json_runtime_facades.ns)
* project-form anchors:
  [io_runtime_demo](../../../examples/projects/tooling/io_runtime_demo),
  [input_runtime_demo](../../../examples/projects/tooling/input_runtime_demo),
  [terminal_io_demo](../../../examples/projects/tooling/terminal_io_demo),
  [host_text_runtime_demo](../../../examples/projects/tooling/host_text_runtime_demo),
  [text_json_demo](../../../examples/projects/tooling/text_json_demo),
  [text_report_json_demo](../../../examples/projects/tooling/text_report_json_demo),
  [time_report_demo](../../../examples/projects/tooling/time_report_demo),
  [benchmark_report_demo](../../../examples/projects/tooling/benchmark_report_demo),
  [benchmark_report_count_demo](../../../examples/projects/tooling/benchmark_report_count_demo),
  [benchmark_report_file_demo](../../../examples/projects/tooling/benchmark_report_file_demo)

Wider grouped route:

* stdin and tty probes:
  [stdin_runtime_demo](../../../examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](../../../examples/projects/tooling/tty_runtime_demo)
* formatting and result/report probes:
  [text_format_runtime_demo](../../../examples/projects/tooling/text_format_runtime_demo),
  [text_report_builder_demo](../../../examples/projects/tooling/text_report_builder_demo),
  [text_report_json_demo](../../../examples/projects/tooling/text_report_json_demo),
  [time_report_demo](../../../examples/projects/tooling/time_report_demo),
  [benchmark_report_demo](../../../examples/projects/tooling/benchmark_report_demo),
  [benchmark_report_count_demo](../../../examples/projects/tooling/benchmark_report_count_demo),
  [benchmark_report_file_demo](../../../examples/projects/tooling/benchmark_report_file_demo),
  [error_runtime_demo](../../../examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](../../../examples/projects/tooling/result_runtime_demo),
  [result_enum_runtime_demo](../../../examples/projects/tooling/result_enum_runtime_demo),
  [diagnostic_runtime_demo](../../../examples/projects/tooling/diagnostic_runtime_demo),
  [result_diagnostic_demo](../../../examples/projects/tooling/result_diagnostic_demo),
  [task_result_enum_demo](../../../examples/projects/task/task_result_enum_demo),
  [net_result_enum_recipe_demo](../../../examples/projects/domains/net_result_enum_recipe_demo),
  [shader_result_enum_demo](../../../examples/projects/domains/shader_result_enum_demo)
* cli/session probes:
  [cli_session_demo](../../../examples/projects/tooling/cli_session_demo),
  [cli_shell_session_demo](../../../examples/projects/tooling/cli_shell_session_demo),
  [cli_report_session_demo](../../../examples/projects/tooling/cli_report_session_demo),
  [cli_runtime_demo](../../../examples/projects/tooling/cli_runtime_demo)

## Current Reading Rule

If you only want one pass:

1. start with [io_runtime_recipe.ns](../../../stdlib/std/io_runtime_recipe.ns)
2. widen to [terminal_io_recipe.ns](../../../stdlib/std/terminal_io_recipe.ns)
3. then read [host_text_runtime_recipe.ns](../../../stdlib/std/host_text_runtime_recipe.ns)
4. then [text_report_builder_recipe.ns](../../../stdlib/std/text_report_builder_recipe.ns)
5. then [io_report_recipe.ns](../../../stdlib/std/io_report_recipe.ns)
6. end with [report_runtime_recipe.ns](../../../stdlib/std/report_runtime_recipe.ns)

Short rule:

* terminal edge first
* text/data shaping second
* report-facing aggregation last
* command/workflow branching belongs to the tooling lane
