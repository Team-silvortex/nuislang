# `std`

`std` is the practical systems layer above `core`.

Canonical short map:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  Use that file for the shortest current reading path across task, host I/O,
  persistence, and filesystem surfaces. Use this README for local detail only.
* [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
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

Current frontdoor docs:

* [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
* [docs/reference/std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* [docs/reference/std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
* [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
* [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
* [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)

## Current Mainline Lanes

Use these as the primary cluster names when placing new work:

* command/workflow/tooling
  - local router:
    [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
  - frontdoor chain:
    [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
    ->
    [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
    ->
    [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
    ->
    [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)
  - current image-preprocess bridge:
    [docs/reference/tooling-image-preprocess-lane.md](/Users/Shared/chroot/dev/nuislang/docs/reference/tooling-image-preprocess-lane.md)
* host I/O and text
  - local router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
  - shortest lane route:
    `io_runtime_recipe -> terminal_io_recipe -> host_text_runtime_recipe -> text_format_runtime_recipe -> json_runtime_recipe -> text_pipeline_recipe -> text_report_builder_recipe -> io_report_recipe -> text_json_recipe`
* task/runtime
  - local router:
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
* filesystem/path/location
  - local router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
  - shortest lane route:
    `path_runtime_recipe -> file_read_recipe -> file_write_recipe -> file_copy_recipe -> directory_stat_recipe -> file_runtime_recipe -> filesystem_report_recipe -> filesystem_io_report_recipe -> filesystem_report_file_recipe -> location_runtime_recipe`
  - persistence companion:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
* net/session
  - local router:
    [network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)
  - contract:
    [docs/reference/std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
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

* [lib/task_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/lib/task_contracts.ns)
  exposes the initial `StdTaskContracts` helper surface for project-level
  stdlib galaxy injection
* [lib/io_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/lib/io_contracts.ns)
  exposes the initial `StdIoContracts` helper surface for normalizing host I/O
  byte counts, flush statuses, and process-style exit codes

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
  - [task_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime.ns)
  - [command_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime.ns)
  - [subprocess_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime.ns)
  - [workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime.ns)
  - current focus:
    command/subprocess/workflow now share a context-aware execution shape with
    launch metadata, cwd routing, timeout routing, and fail-fast workflow
    composition
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
  - [error_model_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_model_runtime.ns)
  - [error_bridge_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_bridge_runtime.ns)
  - [error_codes_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_codes_runtime.ns)
  - [error_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime.ns)
  - [result_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime.ns)
  - [result_enum_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_enum_runtime.ns)
  - [diagnostic_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime.ns)
  - bridge contract:
    [std-result-bridge-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-result-bridge-contract.md)
* config and persistence
  - [config_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime.ns)
  - [kv_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime.ns)
  - [cache_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime.ns)

Recipe modules:

* data/window routing
  - [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns)
  - [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns)
  - [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns)
  - [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns)
  - [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* CLI/tooling runtime
  - [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
  - [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
  - [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
  - [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
  - [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
  - [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)
  - [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)
* net/runtime staging
  - grouped rule:
    `profile core -> transport edge -> syscall edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  - reading router:
    [stdlib/std/network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)
  - contract:
    [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
  - current narrow frontdoor:
    `net_http_client_session_recipe -> net_httpish_header_session_recipe -> net_http_client_lane_recipe`
  - shortest route:
    [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  - cross-lane companions:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md),
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
* checker/reporter tooling
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* result/diagnostic staging
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
  - shared bridge rule:
    [std-result-bridge-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-result-bridge-contract.md)
* directory/stat staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* directory/create staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* automation/workflow tooling
  - local lane router:
    [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
  - current project-form companion ladder:
    [cli_workflow_automation_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_workflow_automation_demo) ->
    [cli_build_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_build_pipeline_demo) ->
    [cli_project_build_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_project_build_report_demo) ->
    [cli_compile_workflow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_compile_workflow_demo)
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
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* kv/runtime staging
  - use the local companion router:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
* cache/runtime staging
  - use the local companion router:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
* config/cache staging
  - use the local companion router:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
* shell-oriented command bridge
  - use the local lane router:
    [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
* path/runtime staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* path/rename staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* path/remove staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* file/output staging
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* terminal/io staging
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* line-input staging
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* text/json staging
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* clock/test timing alignment
  - local timing base:
    [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
    [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
    [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - task timing bridge:
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
* time/clock
  - shortest local route:
    `time_runtime_recipe -> sleep_runtime_recipe -> clock_runtime_recipe -> clock_domain_runtime_recipe -> clock_test_recipe`

## Local Detail

Use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
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
  [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
* lane contract:
  [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)
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
  - [net_http_client_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_recipe.ns)
  - [net_http_client_session_async_loop_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_client_session_async_loop_recipe.ns)
  - [net_http_service_lane_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_http_service_lane_recipe.ns)
* packet-shaped async/httpish summaries
  - [net_httpish_client_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_client_session_packet_recipe.ns)
  - [net_httpish_service_session_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_service_session_packet_recipe.ns)
* packet plus session layering
  - [net_httpish_header_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_session_recipe.ns)
  - [net_httpish_header_service_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/net_httpish_header_service_session_recipe.ns)

### First File Per Main Cluster

* task-facing async/task
  - start with the local lane router:
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
* tooling-facing host/runtime
  - start with the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* state/location/persistence
  - start with the local companion router:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
* filesystem read/write/mutate
  - start with the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* inspection/formatting
  - start with [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

### Pure-To-Composite Clusters

Read these lanes as `pure layer -> wider composition layer`.

* task
  - use the local lane router:
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
* host I/O
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
  - shortest lane route:
    `io_runtime_recipe -> terminal_io_recipe -> host_text_runtime_recipe -> report_runtime_recipe`
* text/data
  - use the local lane router:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
* command/tooling
  - use the local lane router:
    [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
  - shortest lane route:
    `command_runtime_recipe -> subprocess_runtime_recipe -> workflow_runtime_recipe -> cli_compile_workflow_recipe`
* net
  - keep the root lane short and use the dedicated router:
    [stdlib/std/network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)
  - grouped rule:
    `profile core -> transport edge -> syscall edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  - contract wording:
    [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
  - companion route:
    [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* time/clock
  - shortest local route:
    `time_runtime_recipe -> sleep_runtime_recipe -> clock_runtime_recipe -> clock_domain_runtime_recipe -> clock_test_recipe`
* filesystem metadata
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
* data/window/fabric
  - shortest local route:
    `window_runtime_recipe -> pipe_runtime_recipe -> fabric_runtime_recipe -> handle_table_runtime_recipe -> window_fabric_recipe`

### Local Mini-Maps

* path naming/inspection
  - use the local lane router:
    [filesystem/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem/README.md)
  - shortest route inside the lane:
    `path_runtime_recipe -> path_parent_recipe/path_depth_recipe -> path_filename_recipe/path_stem_recipe/path_extension_recipe`
* host I/O and local runtime
  - process-facing local runtime:
    [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns),
    [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns),
    [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns)
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - host read/write and text/report routing:
    [host/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/host/README.md)
  - command/workflow-specific routing:
    [tooling/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/tooling/README.md)
* time/clock
  - [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
  - [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
  - [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
  - [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
  - [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)
  - source mirrors:
    [hello_time_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_time_runtime_facades.ns),
    [hello_sleep_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_sleep_runtime_facades.ns),
    [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns),
    [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns),
    [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns)
  - project mirrors:
    [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
    [sleep_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/sleep_runtime_demo),
    [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
    [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo)
* persistence
  - use the local companion router:
    [persistence/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/persistence/README.md)
  - shortest subgroup route:
    `cwd_runtime_recipe -> location_runtime_recipe -> kv_runtime_recipe -> config_cache_recipe`
* task-facing
  - use the local lane router:
    [task/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/task/README.md)
  - shortest lane route:
    `task_runtime_recipe -> task_lifecycle_recipe -> task_result_policy_recipe -> task_scheduler_recipe -> task_cli_recipe`
  - timing bridge base:
    [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
    [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
    [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)

### Reading Rule

* use this README for module inventory and local clustering
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  for the shortest repo-level route
* use the example READMEs for `recipe -> facade -> project` mirrors
* for concrete mirrors, use:
  - [examples/ns/ffi/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/README.md)
  - [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/std/module.toml)
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
