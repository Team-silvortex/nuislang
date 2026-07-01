# `std` Mainline Layering Contract

This file captures the current layering contract for the checked-in `std`
mainline.

It is not a style guide for every future module. It is the current repository
truth about how the already-checked-in `std` lanes are meant to stack.

## Current Rule Of Thumb

The current `std` mainline prefers:

* a narrow `*_runtime_recipe.ns` pure layer first
* a wider composition recipe only after the pure layer already exists
* a source-level facade mirror in `examples/ns/ffi`
* a project-form mirror in `examples/projects`

That means the default growth shape today is:

```text
raw runtime facade
-> pure runtime recipe
-> wider composition recipe
-> source/project companions
```

Not every lane has every step, but the repository now consistently prefers that
direction.

## What The Pure Layer Is

A pure layer is the narrowest currently-checked-in contract for one runtime
surface.

Today that usually means:

* one family of host/runtime/data operations
* no extra umbrella orchestration unless it is inseparable from the surface
* one small summary struct or equivalent capture path
* a shape that can be mirrored directly in both source and project examples

Examples:

* [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
* [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
* [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns)
* [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns)
* [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns)
* [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns)

## What The Composition Layer Is

A composition layer is a wider recipe that intentionally combines several pure
surfaces into one practical route.

Today that includes lanes such as:

* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)
* [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
* [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)

The important current rule is:

* a composition recipe should not be the first place a runtime surface becomes
  readable if the narrower pure layer can reasonably exist

## Current Mainline Clusters

The repository currently exposes these mainline layering clusters.

### Task

```text
task_runtime
-> task_status
-> task_value
-> task_compare
-> task_lifecycle
-> task_clock / task_scheduler
-> task_cli
```

Concrete sources:

* [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns)
* [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns)
* [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns)
* [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)

Cluster contract:

* [std-task-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-task-layering-contract.md)

### Host I/O

```text
io
-> stdin / tty
-> input
-> terminal_io
```

Concrete sources:

* [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns)
* [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns)
* [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns)
* [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns)

Cluster contract:

* [std-host-io-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-host-io-layering-contract.md)

### Text / Data

```text
host_text
-> text_format
-> json
-> text_pipeline
-> text_report_builder
-> io_report
-> text_json
```

Concrete sources:

* [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns)
* [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns)
* [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns)
* [text_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_pipeline_recipe.ns)
* [text_report_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_report_builder_recipe.ns)
* [io_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_report_recipe.ns)
* [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)

### Command / Tooling

```text
command
-> subprocess
-> command_shell
```

Concrete sources:

* [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
* [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)

Current forward contract:

* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)

### Filesystem Metadata

```text
fs_metadata
-> directory
-> stat
-> directory_stat
```

Concrete sources:

* [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns)
* [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns)
* [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns)
* [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

### Filesystem I/O And Reports

```text
file_read / file_write / file_copy / file_roundtrip
-> file_output
-> filesystem_report
-> filesystem_io_report
-> filesystem_report_file
```

Concrete sources:

* [file_read_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_read_recipe.ns)
* [file_write_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_write_recipe.ns)
* [file_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_copy_recipe.ns)
* [file_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_roundtrip_recipe.ns)
* [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)
* [filesystem_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_recipe.ns)
* [filesystem_io_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_io_report_recipe.ns)
* [filesystem_report_file_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_file_recipe.ns)

Current project proof route:

* [file_read_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_read_demo)
* [file_write_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_write_demo)
* [file_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_copy_demo)
* [file_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_roundtrip_demo)
* [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_output_demo)
* [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_create_demo)
* [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_remove_demo)
* [filesystem_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/filesystem_report_demo)
* [filesystem_report_file_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/filesystem_report_file_demo)
* [filesystem_io_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/filesystem_io_report_demo)

Current contract rule:

* project-form filesystem examples that claim to run should consume
  `StdFsContracts` through `galaxy = ["std=workspace"]`
* successful smoke examples should return `fs_ok()` and failure should return
  `fs_error()` instead of leaking compact probe totals as process exit codes
* temp-backed host paths are preferred for true run-artifact smoke; fixed
  integer path handles should be kept only for narrow lowering probes

### Data / Window / Fabric

```text
window
-> pipe
-> fabric
-> handle_table
-> window_fabric
```

Concrete sources:

* [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns)
* [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns)
* [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns)
* [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns)
* [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)

Cluster contract:

* [std-data-window-fabric-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-data-window-fabric-layering-contract.md)

## What This Contract Does Not Promise

This document does not promise that:

* every future runtime surface must use the exact same naming pattern
* every composition recipe will always be strictly linear
* the current cluster order is a frozen language-level ABI
* these layers are already the final framework or package architecture

It only captures what the repository currently treats as the safest growth
direction for checked-in `std` lanes.

## Current Guidance

If you are adding a new `std` runtime lane today:

* first ask whether a narrow pure `*_runtime_recipe.ns` can exist
* only add the wider composition recipe after the pure layer is readable
* keep the first companion source and project examples as narrow mirrors
* prefer extending an existing cluster over inventing a new umbrella layer too
  early

If you are reading the repository today:

* use this contract to understand how the lanes stack
* use the cluster-specific layering contracts when one lane needs more detail
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  for local module inventory

## Related References

Global index and route:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)

Cluster contracts:

* [std-host-io-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-host-io-layering-contract.md)
* [std-data-window-fabric-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-data-window-fabric-layering-contract.md)
* [std-task-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-task-layering-contract.md)

Semantic boundaries:

* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
* [nir-optimization-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-optimization-contract.md)
