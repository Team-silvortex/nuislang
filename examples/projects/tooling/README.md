# Tooling Project Companions

This folder contains narrow project-form host/runtime companions.

Most entries here are small surface probes, not showcase programs.

Older low-level shell, line-input, and report probes now live under:

* [examples/legacy/tooling](/Users/Shared/chroot/dev/nuislang/examples/legacy/tooling)

## Start Here

If you only want the shortest tooling route, start with:

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)

## Pick By Goal

* argv and environment:
  [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/argv_runtime_demo),
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/env_runtime_demo)
* process and command execution:
  [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/process_runtime_demo),
  [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo),
  [subprocess_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo)
* text, json, diagnostics:
  [host_text_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/host_text_runtime_demo),
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
* for repo-level routing, prefer
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
