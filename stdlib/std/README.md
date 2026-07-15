# `std`

`std` is the practical systems layer above `core`.

Canonical short map:

* [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
  Use that file for the shortest current reading path across task, host I/O,
  persistence, and filesystem surfaces. Use this README for local detail only.
* [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](../../docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
  Use that file for the current `0.20.*` refactor order and the current
  “which lane do we normalize first?” answer.

Current source-style rule:

* checked-in `std` `.ns` modules now prefer the address surface spellings
  `ptr.value`, `ptr.next`, `buffer.len`, and `buffer[index]`
* builtin helper names remain relevant as lowering/runtime vocabulary, not as
  the preferred source-level style

## Current Refactor Frontdoor

For the current `0.20.*` line, do not read `std` as one flat bucket first.

Read it in this order:

1. command/workflow/tooling
2. host I/O and text
3. task/runtime
4. filesystem/path/location
5. net/session

Current practical rule:

* normalize one lane frontdoor first
* only then widen or relocate that lane
* keep net using the dedicated local router instead of re-listing every file
  as if it were just another small family
* filesystem examples that claim run-artifact smoke value should consume
  `StdFsContracts` through `std=workspace` and return `fs_ok` / `fs_error`
  rather than leaking raw probe totals as process exits

Current frontdoor docs:

* [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](../../docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
* [docs/reference/std-mainline-layering-contract.md](../../docs/reference/std-mainline-layering-contract.md)
* [docs/reference/std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
* [tooling/README.md](tooling/README.md)
* [filesystem/README.md](filesystem/README.md)
* [host/README.md](host/README.md)
* [task/README.md](task/README.md)
* [persistence/README.md](persistence/README.md)

## Current Mainline Lanes

Use these as the primary cluster names when placing new work:

* command/workflow/tooling
  - local router:
    [tooling/README.md](tooling/README.md)
  - frontdoor chain:
    [command_runtime_recipe.ns](command_runtime_recipe.ns)
    ->
    [subprocess_runtime_recipe.ns](subprocess_runtime_recipe.ns)
    ->
    [workflow_runtime_recipe.ns](workflow_runtime_recipe.ns)
    ->
    [cli_compile_workflow_recipe.ns](cli_compile_workflow_recipe.ns)
  - current image-preprocess bridge:
    [docs/reference/tooling-image-preprocess-lane.md](../../docs/reference/tooling-image-preprocess-lane.md)
* host I/O and text
  - local router:
    [host/README.md](host/README.md)
  - shortest lane route:
    `io_runtime_recipe -> terminal_io_recipe -> host_text_runtime_recipe -> text_format_runtime_recipe -> json_runtime_recipe -> text_pipeline_recipe -> text_report_builder_recipe -> io_report_recipe -> text_json_recipe`
  - current project smoke:
    `io_runtime_demo -> terminal_io_demo -> stdin_runtime_demo -> host_text_runtime_demo -> text_pipeline_demo -> io_report_demo -> filesystem_io_report_demo`
  - observable CLI smoke:
    `std_tooling_observable_cli_smoke_checks_reports_and_stdin` builds
    `filesystem_io_report_demo`, `stdin_runtime_demo`, and `cli_wc_demo`, checks
    `run-artifact --json` prelaunch readiness, verifies stdout/stderr report
    output from the host IO report lane, and runs the stdin demo binary with
    piped input while checking `host_stdin_read` lowering anchors. It also
    direct-runs the compiled `cli_wc_demo` binary through the
    `argv -> file.read -> buffer` path with a real input file and
    checks the generated `bytes/text_len/lines/words/scan_ns` text report plus
    argv, file-read, and word-count lowering anchors.
* task/runtime
  - local router:
    [task/README.md](task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
* filesystem/path/location
  - local router:
    [filesystem/README.md](filesystem/README.md)
  - shortest lane route:
    `path_runtime_recipe -> file_read_recipe -> file_write_recipe -> file_copy_recipe -> directory_stat_recipe -> file_runtime_recipe -> filesystem_report_recipe -> filesystem_io_report_recipe -> filesystem_report_file_recipe -> location_runtime_recipe`
  - current project smoke:
    `file_read_demo -> file_write_demo -> file_copy_demo -> file_roundtrip_demo -> file_output_demo -> directory_create_demo -> directory_remove_demo -> filesystem_report_demo -> filesystem_report_file_demo`
  - persistence companion:
    [persistence/README.md](persistence/README.md)
* net/session
  - local router:
    [network/README.md](network/README.md)
  - contract:
    [docs/reference/std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
  - current HTTP/session frontdoor:
    `net_http_client_session_recipe -> net_httpish_header_session_recipe -> net_http_client_lane_recipe`
  - current service mirror:
    `net_httpish_service_session_packet_recipe -> net_httpish_header_service_session_recipe -> net_http_service_lane_recipe`

## Current Status

At the current repository stage, `std` is also still mostly a layout/contract
layer, but it now has its first small checked-in `.ns` source set.

That means its role is already important for dependency boundaries, and it is
now starting to accumulate small reusable modules for data/window/pipe helper
patterns.

First auto-injectable library module:

* [lib/task_contracts.ns](lib/task_contracts.ns)
  exposes the initial `StdTaskContracts` helper surface for project-level
  stdlib galaxy injection, including completed/timed-out/cancelled status
  encoding, selected completed values, lifecycle totals, policy totals, and
  task-backed CLI summary totals
* [lib/io_contracts.ns](lib/io_contracts.ns)
  exposes the initial `StdIoContracts` helper surface for normalizing host I/O
  byte counts, flush statuses, report write coverage, and process-style exit
  codes, including console write/flush validation, single-output report
  validation, and terminal readiness gates
* [lib/fs_contracts.ns](lib/fs_contracts.ns)
  exposes the initial `StdFsContracts` helper surface for normalizing
  filesystem metadata/stat probes, file read/write/copy status, directory
  mutation/stat summaries, and path/report/session probe summaries, including
  file report output validation gates, file read/write/chunk-read/roundtrip
  readiness gates, directory mutation plus path copy/rename/remove readiness
  gates, and relative path safety gates for CLI/filesystem frontdoors
* [lib/cli_contracts.ns](lib/cli_contracts.ns)
  exposes the initial `StdCliContracts` helper surface for normalizing
  argv/env/process probes, result/error/diagnostic summaries, command
  requests/results, workflow gates, CLI sessions, and project frontdoor
  recommendations, including command/workflow validation gates plus reusable
  fail-fast, skipped-step, executed-count, failure-stage, and selected-summary
  workflow helpers
* [lib/net_contracts.ns](lib/net_contracts.ns)
  exposes the initial `StdNetContracts` helper surface for normalizing
  network ready states, endpoint/window summaries, HTTP byte estimates,
  owned-transport lifecycle values, HTTP client lane totals, network result
  ready/value probes, and session/task bridge totals
* [lib/text_contracts.ns](lib/text_contracts.ns)
  exposes the initial `StdTextContracts` helper surface for normalizing text
  handles, measured lengths, line/word statistics, report readiness, formatted
  report probes, report-builder line summaries, JSON shape lengths, JSON/text
  consistency gates, text pipeline summaries, and text pipeline readiness gates
* [lib/time_contracts.ns](lib/time_contracts.ns)
  exposes the initial `StdTimeContracts` helper surface for normalizing wall
  time, monotonic time, sleep, clock-domain, benchmark span probes, and
  benchmark validation gates, including four-sample count clamping,
  active-probe filtering, min/max/mid/average helpers, and benchmark triplet
  totals
* [lib/hetero_contracts.ns](lib/hetero_contracts.ns)
  exposes the initial `StdHeteroContracts` helper surface for normalizing
  heterogeneous backend ids, C FFI proxy whitelist checks, accepted/rejected
  proxy status codes, proxy manifest totals, ABI group validation summaries,
  and benchmark/test probe totals

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
  - [window_runtime.ns](window_runtime.ns)
  - [pipe_runtime.ns](pipe_runtime.ns)
  - [fabric_runtime.ns](fabric_runtime.ns)
  - [handle_table_runtime.ns](handle_table_runtime.ns)
* text / file / path
  - [host_text_runtime.ns](host_text_runtime.ns)
  - [file_runtime.ns](file_runtime.ns)
  - [path_runtime.ns](path_runtime.ns)
* CLI and process
  - [argv_runtime.ns](argv_runtime.ns)
  - [env_runtime.ns](env_runtime.ns)
  - [process_runtime.ns](process_runtime.ns)
  - [task_runtime.ns](task_runtime.ns)
  - [command_runtime.ns](command_runtime.ns)
  - [subprocess_runtime.ns](subprocess_runtime.ns)
  - [workflow_runtime.ns](workflow_runtime.ns)
  - current focus:
    command/subprocess/workflow now share a context-aware execution shape with
    launch metadata, cwd routing, timeout routing, and fail-fast workflow
    composition
* terminal and output
  - [io_runtime.ns](io_runtime.ns)
  - [stdin_runtime.ns](stdin_runtime.ns)
  - [line_input_runtime.ns](line_input_runtime.ns)
  - [tty_runtime.ns](tty_runtime.ns)
  - [json_runtime.ns](json_runtime.ns)
  - [text_format_runtime.ns](text_format_runtime.ns)
* filesystem inspection
  - [fs_metadata_runtime.ns](fs_metadata_runtime.ns)
  - [directory_runtime.ns](directory_runtime.ns)
  - [stat_runtime.ns](stat_runtime.ns)
* environment and location
  - [cwd_runtime.ns](cwd_runtime.ns)
  - [temp_runtime.ns](temp_runtime.ns)
  - [home_runtime.ns](home_runtime.ns)
* time and clock
  - [time_runtime.ns](time_runtime.ns)
  - [clock_runtime.ns](clock_runtime.ns)
  - [clock_domain_runtime.ns](clock_domain_runtime.ns)
* error and reporting
  - [error_model_runtime.ns](error_model_runtime.ns)
  - [error_bridge_runtime.ns](error_bridge_runtime.ns)
  - [error_codes_runtime.ns](error_codes_runtime.ns)
  - [error_runtime.ns](error_runtime.ns)
  - [result_runtime.ns](result_runtime.ns)
  - [result_enum_runtime.ns](result_enum_runtime.ns)
  - [diagnostic_runtime.ns](diagnostic_runtime.ns)
  - bridge contract:
    [std-result-bridge-contract.md](../../docs/reference/std-result-bridge-contract.md)
* config and persistence
  - [config_runtime.ns](config_runtime.ns)
  - [kv_runtime.ns](kv_runtime.ns)
  - [cache_runtime.ns](cache_runtime.ns)

Recipe modules:

* data/window routing
  - [window_runtime_recipe.ns](window_runtime_recipe.ns)
  - [pipe_runtime_recipe.ns](pipe_runtime_recipe.ns)
  - [fabric_runtime_recipe.ns](fabric_runtime_recipe.ns)
  - [handle_table_runtime_recipe.ns](handle_table_runtime_recipe.ns)
  - [window_fabric_recipe.ns](window_fabric_recipe.ns)
* CLI/tooling runtime
  - [cli_session_recipe.ns](cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](cli_report_session_recipe.ns)
  - [cli_workflow_automation_recipe.ns](cli_workflow_automation_recipe.ns)
  - [cli_build_pipeline_recipe.ns](cli_build_pipeline_recipe.ns)
  - [cli_project_build_report_recipe.ns](cli_project_build_report_recipe.ns)
  - [cli_compile_workflow_recipe.ns](cli_compile_workflow_recipe.ns)
  - [hetero_proxy_benchmark_recipe.ns](hetero_proxy_benchmark_recipe.ns)
    models a portable heterogeneous benchmark route where a real C FFI host
    bridge acts as the backend proxy under a whitelist signature check, while
    rejected signatures are recorded without dispatching through the host bridge;
    it also scores std-level ABI group and link-allowed validation summaries
* net/runtime staging
  - grouped rule:
    `profile core -> transport edge -> syscall edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  - reading router:
    [stdlib/std/network/README.md](network/README.md)
  - contract:
    [std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
  - current narrow frontdoor:
    `net_http_client_session_recipe -> net_httpish_header_session_recipe -> net_http_client_lane_recipe`
  - shortest route:
    [current-mainline-map.md](../../docs/current-mainline-map.md)
  - cross-lane companions:
    [host/README.md](host/README.md),
    [task/README.md](task/README.md)
* checker/reporter tooling
  - use the local lane router:
    [host/README.md](host/README.md)
* result/diagnostic staging
  - use the local lane router:
    [host/README.md](host/README.md)
  - shared bridge rule:
    [std-result-bridge-contract.md](../../docs/reference/std-result-bridge-contract.md)
* directory/stat staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* directory/create staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* automation/workflow tooling
  - local lane router:
    [tooling/README.md](tooling/README.md)
  - current project-form companion ladder:
    [cli_workflow_automation_demo](../../examples/projects/tooling/cli_workflow_automation_demo) ->
    [cli_build_pipeline_demo](../../examples/projects/tooling/cli_build_pipeline_demo) ->
    [cli_project_build_report_demo](../../examples/projects/tooling/cli_project_build_report_demo) ->
    [cli_compile_workflow_demo](../../examples/projects/tooling/cli_compile_workflow_demo)
  - current recipe shape:
    the checked-in workflow example is now a four-step gate with executed-step
    counting, blocked-step reporting, and per-step launch/cwd/timeout summary
  - shared front-door shape:
    `workflow_frontdoor_runtime_recipe` is the current narrow reference for the
    grouped `frontdoor surface` contract reused by the higher tooling samples
  - integration example:
    `cli_workflow_automation_recipe` is the current smallest checked-in sample
    that ties CLI session, async gate, report emission, automation staging, and
    four-step workflow execution into one toolchain-shaped entry
  - concrete tool example:
    `cli_build_pipeline_recipe` is the current build-oriented sample with
    `prepare/check/compile/package` stage naming, artifact staging,
    pipeline-specific plan summary, and the shared front-door summary surface
  - project-facing report example:
    `cli_project_build_report_recipe` is the current most concrete sample with
    `project/artifact/manifest/build_report` vocabulary and
    `configure/verify/emit/report` stage naming plus the shared front-door
    summary surface
  - front-door compile example:
    `cli_compile_workflow_recipe` is the current highest-level sample with
    `doctor/check/test/build/release-check` orchestration, where the build
    stage reuses the nested project build plan/report contract and the summary
    surface now also carries workflow-entrance recommendation handles plus a
    thin `project` / `single-file` source-kind split, debug-workflow mirror,
    and one grouped front-door summary surface
* location/runtime staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* kv/runtime staging
  - use the local companion router:
    [persistence/README.md](persistence/README.md)
* cache/runtime staging
  - use the local companion router:
    [persistence/README.md](persistence/README.md)
* config/cache staging
  - use the local companion router:
    [persistence/README.md](persistence/README.md)
* shell-oriented command bridge
  - use the local lane router:
    [tooling/README.md](tooling/README.md)
* path/runtime staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* path/rename staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* path/remove staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* file/output staging
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* terminal/io staging
  - use the local lane router:
    [host/README.md](host/README.md)
* line-input staging
  - use the local lane router:
    [host/README.md](host/README.md)
* text/json staging
  - use the local lane router:
    [host/README.md](host/README.md)
* clock/test timing alignment
  - local timing base:
    [time_runtime_recipe.ns](time_runtime_recipe.ns),
    [clock_runtime_recipe.ns](clock_runtime_recipe.ns),
    [clock_domain_runtime_recipe.ns](clock_domain_runtime_recipe.ns)
  - task timing bridge:
    [task/README.md](task/README.md)
* time/clock
  - shortest local route:
    `time_runtime_recipe -> sleep_runtime_recipe -> clock_runtime_recipe -> clock_domain_runtime_recipe -> clock_test_recipe`

## Local Detail

Use [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
for the shortest route. Use this section only when you want the local shape of
`std` itself.

### Current Workflow Shape

For shell-oriented workflow orchestration, the current reusable shape is:

* execution context first
  - each command request can carry env/cwd/timeout intent plus inherit flags
* per-step report next
  - each workflow step reports launched/executed/blocked state, not just exit
    success
* fail-fast gate after that
  - later steps can be skipped explicitly while still preserving summary data
* batch summary last
  - the checked-in runtime example currently demonstrates a four-step gate
    shape

Read these together when working on host command/workflow plumbing:

* local lane router:
  [tooling/README.md](tooling/README.md)
* lane contract:
  [std-tooling-workflow-contract.md](../../docs/reference/std-tooling-workflow-contract.md)
* shortest lane route:
  `command_runtime_recipe -> subprocess_runtime_recipe -> workflow_runtime_recipe -> cli_compile_workflow_recipe`

For the current `std net` and `httpish` recipes, prefer one repeated shape
instead of inventing a new layout every time.

The working rule is:

* workflow helper stage first
  - `open_*`, `accept_*`, `send_*`, `recv_*`, `close_*`
* plan stage next
  - `build_*_plan(...)`
* packet stage after that
  - `stage_*_packet(...)`
  - `compute_packet_value(...)`
* summary stage last
  - `capture_*_summary(...)`
  - `compute_*_value(...)` when a packet value and a higher-level session value
    both exist
  - `summarize_*_recipe(...)`

Use the smaller value names according to layer:

* `packet_value` when the aggregate only describes the packet staging layer
* `session_value` when the aggregate includes transport/session lifecycle data
* avoid mixing `lane_value` into packet-shaped recipes

Current network-facing examples of this rule:

* client/service workflow helpers
  - [net_http_client_session_recipe.ns](net_http_client_session_recipe.ns)
  - [net_http_client_session_async_loop_recipe.ns](net_http_client_session_async_loop_recipe.ns)
  - [net_http_service_lane_recipe.ns](net_http_service_lane_recipe.ns)
* packet-shaped async/httpish summaries
  - [net_httpish_client_session_packet_recipe.ns](net_httpish_client_session_packet_recipe.ns)
  - [net_httpish_service_session_packet_recipe.ns](net_httpish_service_session_packet_recipe.ns)
* packet plus session layering
  - [net_httpish_header_session_recipe.ns](net_httpish_header_session_recipe.ns)
  - [net_httpish_header_service_session_recipe.ns](net_httpish_header_service_session_recipe.ns)

### First File Per Main Cluster

* task-facing async/task
  - start with the local lane router:
    [task/README.md](task/README.md)
* tooling-facing host/runtime
  - start with the local lane router:
    [host/README.md](host/README.md)
* state/location/persistence
  - start with the local companion router:
    [persistence/README.md](persistence/README.md)
* filesystem read/write/mutate
  - start with the local lane router:
    [filesystem/README.md](filesystem/README.md)
* inspection/formatting
  - start with [directory_stat_recipe.ns](directory_stat_recipe.ns)

### Pure-To-Composite Clusters

Read these lanes as `pure layer -> wider composition layer`.

* task
  - use the local lane router:
    [task/README.md](task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
* host I/O
  - use the local lane router:
    [host/README.md](host/README.md)
  - shortest lane route:
    `io_runtime_recipe -> terminal_io_recipe -> host_text_runtime_recipe -> report_runtime_recipe`
* text/data
  - use the local lane router:
    [host/README.md](host/README.md)
* command/tooling
  - use the local lane router:
    [tooling/README.md](tooling/README.md)
  - shortest lane route:
    `command_runtime_recipe -> subprocess_runtime_recipe -> workflow_runtime_recipe -> cli_compile_workflow_recipe`
* net
  - keep the root lane short and use the dedicated router:
    [stdlib/std/network/README.md](network/README.md)
  - grouped rule:
    `profile core -> transport edge -> syscall edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  - contract wording:
    [std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
  - companion route:
    [current-mainline-map.md](../../docs/current-mainline-map.md)
* time/clock
  - shortest local route:
    `time_runtime_recipe -> sleep_runtime_recipe -> clock_runtime_recipe -> clock_domain_runtime_recipe -> clock_test_recipe`
* filesystem metadata
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
* data/window/fabric
  - shortest local route:
    `window_runtime_recipe -> pipe_runtime_recipe -> fabric_runtime_recipe -> handle_table_runtime_recipe -> window_fabric_recipe`

### Local Mini-Maps

* path naming/inspection
  - use the local lane router:
    [filesystem/README.md](filesystem/README.md)
  - shortest route inside the lane:
    `path_runtime_recipe -> path_parent_recipe/path_depth_recipe -> path_filename_recipe/path_stem_recipe/path_extension_recipe`
* host I/O and local runtime
  - process-facing local runtime:
    [argv_runtime_recipe.ns](argv_runtime_recipe.ns),
    [env_runtime_recipe.ns](env_runtime_recipe.ns),
    [process_runtime_recipe.ns](process_runtime_recipe.ns)
  - [time_runtime_recipe.ns](time_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](clock_domain_runtime_recipe.ns)
  - host read/write and text/report routing:
    [host/README.md](host/README.md)
  - command/workflow-specific routing:
    [tooling/README.md](tooling/README.md)
* time/clock
  - [time_runtime_recipe.ns](time_runtime_recipe.ns)
  - [sleep_runtime_recipe.ns](sleep_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](clock_domain_runtime_recipe.ns)
  - [clock_test_recipe.ns](clock_test_recipe.ns)
  - source mirrors:
    [hello_time_runtime_facades.ns](../../examples/ns/ffi/hello_time_runtime_facades.ns),
    [hello_sleep_runtime_facades.ns](../../examples/ns/ffi/hello_sleep_runtime_facades.ns),
    [hello_clock_runtime_facades.ns](../../examples/ns/ffi/hello_clock_runtime_facades.ns),
    [hello_clock_domain_runtime_facades.ns](../../examples/ns/ffi/hello_clock_domain_runtime_facades.ns),
    [hello_clock_test_facades.ns](../../examples/ns/ffi/hello_clock_test_facades.ns)
  - project mirrors:
    [time_runtime_demo](../../examples/projects/tooling/time_runtime_demo),
    [sleep_runtime_demo](../../examples/projects/tooling/sleep_runtime_demo),
    [clock_runtime_demo](../../examples/projects/tooling/clock_runtime_demo),
    [clock_domain_runtime_demo](../../examples/projects/tooling/clock_domain_runtime_demo)
* persistence
  - use the local companion router:
    [persistence/README.md](persistence/README.md)
  - shortest subgroup route:
    `cwd_runtime_recipe -> location_runtime_recipe -> kv_runtime_recipe -> config_cache_recipe`
* task-facing
  - use the local lane router:
    [task/README.md](task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
  - timing bridge base:
    [time_runtime_recipe.ns](time_runtime_recipe.ns),
    [clock_runtime_recipe.ns](clock_runtime_recipe.ns),
    [clock_domain_runtime_recipe.ns](clock_domain_runtime_recipe.ns)

### Reading Rule

* use this README for module inventory and local clustering
* use [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
  for the shortest repo-level route
* use the example READMEs for `recipe -> facade -> project` mirrors
* for concrete mirrors, use:
  - [examples/ns/ffi/README.md](../../examples/ns/ffi/README.md)
  - [examples/projects/README.md](../../examples/projects/README.md)

See metadata:

* [module.toml](module.toml)
* [host-read-bridge.md](../../docs/reference/host-read-bridge.md)
