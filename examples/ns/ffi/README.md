# FFI `.ns` Examples

This folder contains CPU host-bridge examples:

* `hello_ffi.ns`
* `hello_c_ffi.ns`
* `hello_cli_host_facades.ns`
* `hello_native_cli_runtime.ns`
* `hello_native_command_runtime.ns`
* `hello_native_input_tool.ns`
* `hello_native_cli_pipeline.ns`
* `hello_native_tool_runner.ns`
* `hello_native_workflow_runtime.ns`
* `hello_clock_test_facades.ns`

Reading guidance:

* `hello_ffi.ns`
  current `extern "nurs" interface`-style host bridge
* `hello_c_ffi.ns`
  plain `extern "c"` route kept as the lower-level baseline
* `hello_cli_host_facades.ns`
  a tooling-oriented `extern "c"` example that groups argv/env/cwd/stdout/diagnostic
  style host facades in one place; it now mirrors both
  [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
  ,
  [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
  , and
  [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  from the current `stdlib/std` host-backed tooling direction
* `hello_native_cli_runtime.ns`
  a more concrete native-backed CLI example that leans on the current AOT shim
  batch for `argv/env/cwd/path/fs/stdout/process`, so it is a better guide when
  you want to see which `std` host facades have started to become real system
  integration instead of pure placeholder shape
* `hello_native_command_runtime.ns`
  a focused native-backed command example that shows the current early
  `command/subprocess` staging contract directly:
  `program_handle <- argv`, `argv_handle <- shell-style argv tail built from
  multiple source arguments`, `env_handle <- KEY=VALUE prefix text`; it is the
  repo-local example that most directly mirrors
  [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
* `hello_native_input_tool.ns`
  a small input-driven native example that takes a file path from `argv`,
  performs one native file read, performs one native stdin read, and folds the
  observed byte counts into its own result; it is the clearest repo-local sample
  for the current `file/stdin` AOT-backed host path
* `hello_native_cli_pipeline.ns`
  a combined native CLI sample that first reads file/stdin input and then,
  when input is present, triggers the current command/subprocess bridge and uses
  direct-exit observers in the same flow
* `hello_native_tool_runner.ns`
  a more tool-shaped native example that reads `argv`, launches a command and a
  subprocess, and then uses direct-exit observers to decide its own result; it
  is the clearest repo-local sample for “small native CLI workflow” thinking
* `hello_native_workflow_runtime.ns`
  a native-backed workflow example that leans on the current AOT shim batch for
  `cwd/directory/temp/process/command/subprocess/stdout`, so it is the best
  repo-local source example when you want to see how the current
  [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
  direction starts to touch real host workflow primitives; note that the
  current `command/subprocess` path still uses a small shell-oriented staging
  contract where `argv_handle` is a raw argument-tail text handle and
  `env_handle` is a raw environment-prefix text handle
* `hello_clock_test_facades.ns`
  a clock/timing-oriented `extern "c"` example that mirrors
  [clock_domain_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime.ns)
  ,
  [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  and the current `nuis test` time semantics; it includes a `should_fail=true`
  async test with `clock_domain="global"` so the front-door runner prints the
  resolved host clock domain during execution
  Future direction note:
  [examples/ns/ffi/FUTURE_CLOCK_NEGOTIATION_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/FUTURE_CLOCK_NEGOTIATION_SKETCH.md)

Current note:

* the source language already distinguishes the Rust-oriented `NURS` surface from the raw C ABI bridge, even though today the concrete bridge is still C-compatible underneath
