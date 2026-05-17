# `std`

`std` is the practical systems layer above `core`.

## Current Status

At the current repository stage, `std` is also still mostly a layout/contract
layer, but it now has its first small checked-in `.ns` source set.

That means its role is already important for dependency boundaries, and it is
now starting to accumulate small reusable modules for data/window/pipe helper
patterns.

Intended scope:

* convenience APIs that still preserve the AOT-first and semantics-first nature of `nuis`
* data-plane and host-integration helper surfaces that are too opinionated for `core`
* common utilities that typical `nuis` projects should not have to rebuild each time

Expected areas:

* collections and builder-style helper surfaces
* host FFI helper facades for CPU-side integration
* common data/window/handle-table orchestration helpers
* project/runtime utility APIs that are broadly useful but not framework-specific

Relationship:

* `std` depends on `core`
* `ns-nova` will use `std` as its general-purpose support layer rather than duplicating systems helpers

Source modules are easiest to read in two groups.

Facade modules:

* data/window/pipe helpers
  - [window_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime.ns)
  - [pipe_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime.ns)
  - [fabric_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime.ns)
  - [handle_table_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime.ns)
* text / file / path
  - [host_text_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime.ns)
  - [file_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime.ns)
  - [path_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime.ns)
* CLI and process
  - [argv_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime.ns)
  - [env_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime.ns)
  - [process_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime.ns)
  - [command_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime.ns)
  - [subprocess_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime.ns)
* terminal and output
  - [io_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime.ns)
  - [stdin_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime.ns)
  - [line_input_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_runtime.ns)
  - [tty_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime.ns)
  - [json_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime.ns)
  - [text_format_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime.ns)
* filesystem inspection
  - [fs_metadata_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime.ns)
  - [directory_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime.ns)
  - [stat_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime.ns)
* environment and location
  - [cwd_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime.ns)
  - [temp_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime.ns)
  - [home_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime.ns)
* time and clock
  - [time_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime.ns)
  - [clock_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime.ns)
  - [clock_domain_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime.ns)
* error and reporting
  - [error_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime.ns)
  - [result_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime.ns)
  - [diagnostic_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime.ns)
* config and persistence
  - [config_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime.ns)
  - [kv_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime.ns)
  - [cache_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime.ns)

Recipe modules:

* data/window routing
  - [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* CLI/tooling runtime
  - [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
* checker/reporter tooling
  - [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
* automation/workflow tooling
  - [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
* clock/test timing alignment
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)

Current boundaries:

* `std` is not yet a populated importable source module tree
* `std` is still much thinner than the future practical systems layer it is
  meant to become
* current string/file direction is still host-backed and opaque; there is not
  yet a first-class native `String` or file-path/file-descriptor standard type
* current CLI-facing `argv/env/process` direction is also still host-backed and
  opaque; it is a facade over handles and status integers, not yet a stable
  native command-line/runtime API
* current `path/stdout/stderr` direction is likewise still host-backed and
  opaque; it is useful as a staging facade, but not yet a finalized portable
  path/stream standard library contract
* current `time/clock` direction is intentionally split between basic
  wall/monotonic time facades and a more explicit global clock-scale contract;
  both are still host-backed staging APIs rather than a finalized cross-domain
  timing standard
* current clock/test alignment also includes a small recipe that mirrors the
  front-door runner's current `global -> monotonic` timeout resolution; this
  is a staging contract for async test semantics rather than a final cross-domain
  clock bridge
* `clock_domain_runtime.ns` is the current canonical place where the staging
  clock-domain code mapping is written down explicitly: `0 = monotonic`,
  `1 = wall`, `2 = global`, with `global` currently resolving to `monotonic`
  inside the front-door test runner
* `clock_test_recipe.ns` and the FFI-facing clock test example now mirror that
  canonical mapping using the more explicit field names
  `declared_global_code` and `resolved_global_code`, so the bridge summary reads
  like a contract rather than a pair of unexplained integers
* current `command/subprocess` direction is likewise a host-backed facade over
  opaque process handles and integer status codes; it is useful for CLI/tooling
  experiments but not yet a finalized portable process-management contract
* current `metadata/directory/stat` direction is also still a host-backed file
  system facade; it is enough to sketch CLI and tooling flows, but not yet a
  finalized portable filesystem standard library contract
* current `stdin/line-input/tty` direction is also still a host-backed terminal
  facade; it is enough to sketch command-line interaction flows, but not yet a
  finalized portable terminal standard library contract
* current `json/text-format` direction is also still a host-backed formatting
  facade; it is enough to sketch machine-readable and human-readable output
  flows, but not yet a finalized native text/serialization standard library
  contract
* current `cwd/temp/home` direction is likewise still a host-backed environment
  facade; it is enough to sketch common tooling location flows, but not yet a
  finalized portable runtime-environment standard library contract
* current `error/result/diagnostic` direction is also still a host-backed
  reporting facade; it is enough to sketch tool-facing failure and reporting
  flows, but not yet a finalized native error/diagnostic standard library
  contract
* current `config/kv/cache` direction is also still a host-backed persistence
  facade; it is enough to sketch tool-facing state and lookup flows, but not
  yet a finalized portable configuration/storage standard library contract

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/std/module.toml)
