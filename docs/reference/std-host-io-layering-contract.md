# `std` Host I/O Layering Contract

This file captures the current layering contract for the checked-in `std`
 host-I/O lanes.

It sits one level below
[std-mainline-layering-contract.md](std-mainline-layering-contract.md):
that file explains the global `std` rule of thumb, while this file explains
what the host-I/O lane currently means in repository practice.

## Current Lane Shape

The current host-I/O lane prefers this order:

```text
compiler/read bridge context
-> narrow host/runtime recipes
-> wider observation or shaping recipes
-> source/project companions
```

For checked-in `std`, that currently means:

```text
io / stdin / tty / argv / env / process / command / subprocess
-> input / terminal_io / cli_session / cli_shell_session / cli_report_session / command_shell / cli_runtime / report_runtime
-> examples/ns/ffi mirrors and examples/projects companions
```

## Bridge Boundary First

The compiler and the source layer are related here, but they are not one flat
surface yet.

Current bridge reference:

* [host-read-bridge.md](host-read-bridge.md)

The practical rule today is:

* compiler-known host reads may already be classified semantically
* raw `std` host facades still mostly lower through explicit host FFI
* pure host-I/O recipes are the checked-in source-level contract that sits
  between those raw facades and the wider umbrella recipes

That means the pure `std` host-I/O layer is not “the compiler truth”.
It is the current repository front door for readable, narrow host/runtime
surfaces.

## Pure Host I/O Layers

These are the current narrow checked-in host-I/O routes.

### Execution And Process Surface

* [argv_runtime_recipe.ns](../../stdlib/std/argv_runtime_recipe.ns)
* [env_runtime_recipe.ns](../../stdlib/std/env_runtime_recipe.ns)
* [process_runtime_recipe.ns](../../stdlib/std/process_runtime_recipe.ns)
* [command_runtime_recipe.ns](../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../stdlib/std/subprocess_runtime_recipe.ns)

These are the narrowest readable contracts for:

* process arguments and environment
* process identity and status
* command lifecycle
* subprocess lifecycle and signaling

### Input And Terminal Observation Surface

* [io_runtime_recipe.ns](../../stdlib/std/io_runtime_recipe.ns)
* [stdin_runtime_recipe.ns](../../stdlib/std/stdin_runtime_recipe.ns)
* [tty_runtime_recipe.ns](../../stdlib/std/tty_runtime_recipe.ns)
* [line_input_recipe.ns](../../stdlib/std/line_input_recipe.ns)

These are the narrowest readable contracts for:

* stdout/stderr writes and flushes
* stdin reads
* tty shape observation
* line-oriented stdin use

The important current wrinkle is that
[line_input_runtime.ns](../../stdlib/std/line_input_runtime.ns)
does not have a separate `*_runtime_recipe.ns`, but the checked-in repository
already treats
[line_input_recipe.ns](../../stdlib/std/line_input_recipe.ns)
as the effective narrow pure layer for that lane.

## Wider Composition Layers

These recipes intentionally combine several pure host-I/O surfaces into one
practical route.

* [input_runtime_recipe.ns](../../stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](../../stdlib/std/terminal_io_recipe.ns)
* [cli_session_recipe.ns](../../stdlib/std/cli_session_recipe.ns)
* [cli_shell_session_recipe.ns](../../stdlib/std/cli_shell_session_recipe.ns)
* [cli_report_session_recipe.ns](../../stdlib/std/cli_report_session_recipe.ns)
* [command_shell_recipe.ns](../../stdlib/std/command_shell_recipe.ns)
* [cli_runtime_recipe.ns](../../stdlib/std/cli_runtime_recipe.ns)
* [report_runtime_recipe.ns](../../stdlib/std/report_runtime_recipe.ns)

Current role split:

* `input_runtime` combines stdin-style observation and tty context
* `terminal_io` combines output writes, flushes, stdin, and tty shape
* `cli_session` combines argv count, line-oriented prompt input, terminal
  output, timeout-sensitive task branching, and monotonic tick capture into
  one narrow interactive CLI route
* `cli_shell_session` combines prompt input, argv/program selection, shell
  spawn/subprocess join, terminal writes, flushes, and monotonic tick capture
  into one narrow interactive shell-session route
* `cli_report_session` combines prompt input, timeout-sensitive task result
  selection, diagnostic emission, terminal writes, flushes, and monotonic tick
  capture into one narrow interactive report-session route
* `command_shell` combines command and subprocess surfaces into one shell/tool
  route
* `cli_runtime` pulls several host-I/O and tooling surfaces into one CLI lane
* `report_runtime` sits above the raw writes and diagnostics-facing reporting
  path

The current rule is the same as the global `std` rule:

* these wider recipes should not be the first place a host/runtime surface
  becomes understandable if the narrower pure layer can reasonably exist

## Current Host I/O Clusters

### Observation And Shaping

```text
io
-> stdin / tty / line_input
-> input
-> terminal_io
```

Concrete sources:

* [io_runtime_recipe.ns](../../stdlib/std/io_runtime_recipe.ns)
* [stdin_runtime_recipe.ns](../../stdlib/std/stdin_runtime_recipe.ns)
* [tty_runtime_recipe.ns](../../stdlib/std/tty_runtime_recipe.ns)
* [line_input_recipe.ns](../../stdlib/std/line_input_recipe.ns)
* [input_runtime_recipe.ns](../../stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](../../stdlib/std/terminal_io_recipe.ns)

### Execution And Shelling

```text
argv / env / process
-> command
-> subprocess
-> command_shell
-> cli_runtime
```

Concrete sources:

* [argv_runtime_recipe.ns](../../stdlib/std/argv_runtime_recipe.ns)
* [env_runtime_recipe.ns](../../stdlib/std/env_runtime_recipe.ns)
* [process_runtime_recipe.ns](../../stdlib/std/process_runtime_recipe.ns)
* [command_runtime_recipe.ns](../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../stdlib/std/subprocess_runtime_recipe.ns)
* [command_shell_recipe.ns](../../stdlib/std/command_shell_recipe.ns)
* [cli_runtime_recipe.ns](../../stdlib/std/cli_runtime_recipe.ns)

### Reporting Edge

```text
io
-> diagnostic/result-adjacent reporting
-> report_runtime
```

Concrete sources:

* [io_runtime_recipe.ns](../../stdlib/std/io_runtime_recipe.ns)
* [diagnostic_runtime_recipe.ns](../../stdlib/std/diagnostic_runtime_recipe.ns)
* [result_runtime_recipe.ns](../../stdlib/std/result_runtime_recipe.ns)
* [error_runtime_recipe.ns](../../stdlib/std/error_runtime_recipe.ns)
* [report_runtime_recipe.ns](../../stdlib/std/report_runtime_recipe.ns)
* [result_diagnostic_recipe.ns](../../stdlib/std/result_diagnostic_recipe.ns)

## Companion Expectation

The current checked-in host-I/O lane is expected to have direct mirrors in:

* `examples/ns/ffi` for the source-level facade view
* `examples/projects/tooling` for the project-form route

Examples:

* [hello_io_runtime_facades.ns](../../examples/ns/ffi/hello_io_runtime_facades.ns)
* [hello_command_runtime_facades.ns](../../examples/ns/ffi/hello_command_runtime_facades.ns)
* [hello_subprocess_runtime_facades.ns](../../examples/ns/ffi/hello_subprocess_runtime_facades.ns)
* [hello_tty_runtime_facades.ns](../../examples/ns/ffi/hello_tty_runtime_facades.ns)
* [io_runtime_demo](../../examples/projects/tooling/io_runtime_demo)
* [command_runtime_demo](../../examples/projects/tooling/command_runtime_demo)
* [subprocess_runtime_demo](../../examples/projects/tooling/subprocess_runtime_demo)
* [tty_runtime_demo](../../examples/projects/tooling/tty_runtime_demo)

## What This Contract Does Not Promise

This file does not promise that:

* every host-I/O lane will become compiler-known as `HostReadOnly`
* every CLI-facing recipe is side-effect-light
* `input_runtime` or `terminal_io` are the final long-term package boundaries
* the current execution and reporting split is frozen forever

It only captures the current repository truth about how the checked-in host-I/O
lanes are meant to stack today.

## Current Guidance

If you are extending host I/O today:

* add the narrow host/runtime recipe first when a single surface can stand on
  its own
* only add the umbrella CLI, shell, or terminal route after the narrow layer is
  readable
* keep the source and project companions close to the narrow route before
  widening the lane
* check whether the new surface belongs under the existing host-read bridge
  discussion before claiming compiler-level read semantics

If you are reading host I/O today:

* start with [host-read-bridge.md](host-read-bridge.md)
  if you need the compiler/source boundary
* start with the pure `*_runtime_recipe.ns` files if you need the narrow
  checked-in source contract
* move to `input_runtime`, `terminal_io`, `command_shell`, or `cli_runtime`
  only after the pure layer is clear

## Related References

* [host-read-bridge.md](host-read-bridge.md)
* [std-mainline-layering-contract.md](std-mainline-layering-contract.md)
* [cpu-task-scheduler-clock.md](cpu-task-scheduler-clock.md)
* [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
