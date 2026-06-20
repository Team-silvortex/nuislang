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

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)
* [native_artifact_closure_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo)

Current runnable CLI frontdoor:

* `cli_runtime_demo` is the shortest checked-in host/CLI project that now fits
  the normal `build -> artifact-doctor -> run-artifact` success path
* `cli_session_demo` and `cli_report_session_demo` now also fit the same launch
  path, while still representing the interactive/session-oriented lane
* `workflow_runtime_demo` now fits the same launch path as the workflow-shaped
  command/session frontdoor
* `command_runtime_demo` and `subprocess_runtime_demo` now also fit that same
  launch path as the narrow command/process companions

Current split:

* launch-shaped frontdoors:
  `cli_runtime_demo`, `cli_session_demo`, `cli_report_session_demo`,
  `workflow_runtime_demo`, `command_runtime_demo`, `subprocess_runtime_demo`,
  `native_artifact_closure_demo`
* probe-style companions:
  most of the remaining entries in this folder still exist to expose one host
  surface, one report shape, or one runtime observation slice at a time

## Pick By Goal

* argv and environment:
  [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/argv_runtime_demo),
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/env_runtime_demo)
* process and command execution:
  [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/process_runtime_demo),
  [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo),
  [subprocess_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo),
  [native_artifact_closure_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo)
* text, json, diagnostics:
  [host_text_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/host_text_runtime_demo),
  [text_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_pipeline_demo),
  [text_report_builder_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_report_builder_demo),
  [io_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/io_report_demo),
  [filesystem_io_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/filesystem_io_report_demo),
  [json_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/json_runtime_demo),
  [text_json_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_json_demo),
  [text_format_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_format_runtime_demo),
  [diagnostic_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/diagnostic_runtime_demo)
* result and error surfaces:
  [error_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_runtime_demo),
  [result_diagnostic_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_diagnostic_demo)
* input and terminal I/O:
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/input_runtime_demo),
  [io_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/io_runtime_demo),
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/tty_runtime_demo),
  [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/terminal_io_demo)
* time and clock:
  [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
  [sleep_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/sleep_runtime_demo),
  [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
  [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo)
* CLI session flows:
  [cli_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_session_demo),
  [cli_shell_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_shell_session_demo),
  [cli_report_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_report_session_demo)

## Reading Rule

* use one representative route per surface
* do not read this folder top-to-bottom unless you are auditing runtime
  coverage
* treat most entries outside the CLI/command/workflow trio as companion-only
  probes, not as equal-entry onboarding material
* for repo-level routing, prefer
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* if the question is specifically “can `nuis` compile its own native/artifact
  bundle and survive a real launch-shaped host bridge route?”, start with
  [native_artifact_closure_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo)
