# FFI `.ns` Examples

This folder contains CPU host-bridge examples:

* `hello_ffi.ns`
* `hello_c_ffi.ns`
* `hello_cli_host_facades.ns`
* `hello_native_cli_runtime.ns`
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

Current note:

* the source language already distinguishes the Rust-oriented `NURS` surface from the raw C ABI bridge, even though today the concrete bridge is still C-compatible underneath
