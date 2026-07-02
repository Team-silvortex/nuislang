# Tooling Project Companions

This folder contains narrow project-form host/runtime companions.

Most entries here are small surface probes, not showcase programs.

Older low-level shell, line-input, automation, and report probes have been
retired from the checked-in examples tree.

Current role rule:

* this subtree is mostly companion-only by design
* only the shortest CLI/command/workflow trio should be treated as frontdoor
* most other routes are narrow surface probes for one runtime/tooling contract

## Start Here

If you only want the shortest tooling route, start with:

* [cli_runtime_demo](cli_runtime_demo)
* [cli_cat_demo](cli_cat_demo)
* [cli_wc_demo](cli_wc_demo)
* [command_runtime_demo](command_runtime_demo)
* [workflow_runtime_demo](workflow_runtime_demo)
* [native_artifact_closure_demo](native_artifact_closure_demo)

Current runnable CLI frontdoor:

* `cli_runtime_demo` is the shortest checked-in host/CLI project that now fits
  the normal `build -> artifact-doctor -> run-artifact` success path
* `cli_cat_demo` now turns that same host bridge into a minimal practical
  file-to-stdout CLI, so the tooling lane now has a tiny real text utility
* `cli_wc_demo` now adds a first performance-oriented text/file stats seed over
  the same host bridge, currently anchored on byte count plus text-bridge
  length verification while general synchronous scan loops are still being
  tightened in lowering
* `cli_session_demo` and `cli_report_session_demo` now also fit the same launch
  path, while still representing the interactive/session-oriented lane
* `workflow_runtime_demo` now fits the same launch path as the workflow-shaped
  command/session frontdoor
* `command_runtime_demo` and `subprocess_runtime_demo` now also fit that same
  launch path as the narrow command/process companions
* `cli_compile_workflow_demo` now fits that same launch path as the current
  higher-level compile-workflow companion
* `cli_workflow_automation_demo` now fits that same launch path as the current
  smaller workflow-automation companion
* `cli_build_pipeline_demo` and `cli_project_build_report_demo` now extend that
  same launch path into the higher-level build-pipeline and project-build-report
  companions
* `cli_pgm_info_demo` now fits that same launch path as the first checked-in
  image-shaped CLI companion, proving that the tooling lane can already host a
  narrow file-backed image probe without adding a new runtime bridge
* `cli_pgm_invert_demo` now extends that image-shaped lane into a real
  file-to-file transform companion, which is the current CPU-side stepping
  stone before shader-backed image examples
* `cli_pgm_threshold_demo` now extends that same lane into a mask-style image
  prepass companion, which is closer to the kind of CPU-side preprocessing we
  can feed into later shader-oriented examples

Current split:

* launch-shaped frontdoors:
  `cli_runtime_demo`, `cli_cat_demo`, `cli_wc_demo`, `cli_session_demo`, `cli_report_session_demo`,
  `workflow_runtime_demo`, `command_runtime_demo`, `subprocess_runtime_demo`,
  `native_artifact_closure_demo`, `cli_compile_workflow_demo`,
  `cli_workflow_automation_demo`,
  `cli_build_pipeline_demo`, `cli_project_build_report_demo`,
  `cli_pgm_info_demo`, `cli_pgm_invert_demo`, `cli_pgm_threshold_demo`
* probe-style companions:
  most of the remaining entries in this folder still exist to expose one host
  surface, one report shape, or one runtime observation slice at a time

## Current High-Level Authoring Shape

For the current higher-level tooling companions, prefer one stable
project-form pattern:

```text
Seed
-> capture context
-> build step bundle
-> run/skip ordered steps
-> build success/failure report
-> build summary
-> return one exit code
```

Current exemplars:

* [cli_compile_workflow_demo](cli_compile_workflow_demo)
* [cli_workflow_automation_demo](cli_workflow_automation_demo)
* [cli_build_pipeline_demo](cli_build_pipeline_demo)
* [cli_project_build_report_demo](cli_project_build_report_demo)
* [cli_cat_demo](cli_cat_demo)
* [cli_wc_demo](cli_wc_demo)
* [cli_pgm_info_demo](cli_pgm_info_demo)
* [cli_pgm_invert_demo](cli_pgm_invert_demo)
* [cli_pgm_threshold_demo](cli_pgm_threshold_demo)

Short rule:

* step assembly should be explicit through one `*Steps` bundle
* step progression should read as ordered `run_step` / `skipped_step`
* success/failure collapse should happen through one small report helper set
* `main()` should stay terminal and lowering-friendly

## Pick By Goal

* argv and environment:
  [argv_runtime_demo](argv_runtime_demo),
  [env_runtime_demo](env_runtime_demo)
* process and command execution:
  [process_runtime_demo](process_runtime_demo),
  [command_runtime_demo](command_runtime_demo),
  [subprocess_runtime_demo](subprocess_runtime_demo),
  [native_artifact_closure_demo](native_artifact_closure_demo)
* text, json, diagnostics:
  [host_text_runtime_demo](host_text_runtime_demo),
  [text_pipeline_demo](text_pipeline_demo),
  [text_report_builder_demo](text_report_builder_demo),
  [text_report_json_demo](text_report_json_demo),
  [time_report_demo](time_report_demo),
  [benchmark_report_demo](benchmark_report_demo),
  [benchmark_report_count_demo](benchmark_report_count_demo),
  [benchmark_report_file_demo](benchmark_report_file_demo),
  [hetero_proxy_benchmark_demo](hetero_proxy_benchmark_demo)
  as the benchmark/text/filesystem/heterogeneous proxy std contract consumers,
  [io_report_demo](io_report_demo)
  as the console/text std contract consumer,
  [filesystem_io_report_demo](filesystem_io_report_demo)
  as the filesystem/console std contract consumer,
  [json_runtime_demo](json_runtime_demo),
  [text_json_demo](text_json_demo),
  [text_format_runtime_demo](text_format_runtime_demo),
  [diagnostic_runtime_demo](diagnostic_runtime_demo)
* result and error surfaces:
  [error_runtime_demo](error_runtime_demo),
  [result_runtime_demo](result_runtime_demo),
  [result_diagnostic_demo](result_diagnostic_demo)
* input and terminal I/O:
  [input_runtime_demo](input_runtime_demo),
  [cli_cat_demo](cli_cat_demo),
  [cli_wc_demo](cli_wc_demo),
  [io_runtime_demo](io_runtime_demo)
  as the base console std contract smoke,
  [stdin_runtime_demo](stdin_runtime_demo),
  [tty_runtime_demo](tty_runtime_demo),
  [terminal_io_demo](terminal_io_demo)
  as the terminal/stdin/TTY std contract smoke
* file-backed image probe:
  [cli_pgm_info_demo](cli_pgm_info_demo)
* file-backed image transform:
  [cli_pgm_invert_demo](cli_pgm_invert_demo)
* file-backed image mask prepass:
  [cli_pgm_threshold_demo](cli_pgm_threshold_demo)
* time and clock:
  [time_runtime_demo](time_runtime_demo),
  [sleep_runtime_demo](sleep_runtime_demo),
  [clock_runtime_demo](clock_runtime_demo),
  [clock_domain_runtime_demo](clock_domain_runtime_demo)
* CLI session flows:
  [cli_session_demo](cli_session_demo),
  [cli_shell_session_demo](cli_shell_session_demo),
  [cli_report_session_demo](cli_report_session_demo)

## Reading Rule

* use one representative route per surface
* do not read this folder top-to-bottom unless you are auditing runtime
  coverage
* treat most entries outside the CLI/command/workflow trio as companion-only
  probes, not as equal-entry onboarding material
* for repo-level routing, prefer
  [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
* if the question is specifically “can `nuis` compile its own native/artifact
  bundle and survive a real launch-shaped host bridge route?”, start with
  [native_artifact_closure_demo](native_artifact_closure_demo)
