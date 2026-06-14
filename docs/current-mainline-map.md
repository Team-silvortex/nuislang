# Current Mainline Map

This file is the canonical short reading map for the current repository spine.

If several READMEs seem to overlap, prefer this file first, then drill into the
local README for the area you are actively touching.

## Start Here

* repo status and current toolchain spine:
  [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
* current phase snapshot:
  [versioning/nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
* current phase checklist:
  [versioning/nuis-0.19.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-release-checklist.md)
* minor-version snapshot rule:
  [versioning/nuis-minor-snapshot-rule.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-minor-snapshot-rule.md)
* current minor-line snapshot:
  [versioning/nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
* current minor-line checklist:
  [versioning/nuis-0.19.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-release-checklist.md)
* current mainline goals:
  [versioning/nuis-0.19.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-goals.md)
* active carryover control-flow completion plan:
  [versioning/nuis-0.18.0-control-flow-completion-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-control-flow-completion-plan.md)
* active carryover loop-memory read extension sketch:
  [versioning/nuis-0.18.0-loop-memory-read-contract-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-loop-memory-read-contract-sketch.md)
* current compile workflow anchor:
  [versioning/nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
  current CLI frontdoor rule:
  `nuis status/help -> nuis workflow -> nuis project-doctor/project-status/scheduler-view -> check/test/build/release-check`
* current example-routing anchor:
  [versioning/nuis-0.18.0-example-routing-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-example-routing-snapshot.md)
  current example-tree rule:
  `frontdoor first -> grouped companions next -> explicit probe or legacy routes after that`
* current lowering capability anchor:
  [versioning/nuis-0.17.0-lowering-capability-map.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-lowering-capability-map.md)
* current network/http readiness anchor:
  [versioning/nuis-0.17.0-network-http-readiness-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-network-http-readiness-checklist.md)
* current mainline regression matrix:
  [versioning/nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
* current self-hosted gate plan:
  [versioning/nuis-0.17.0-self-hosted-mainline-gate-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-self-hosted-mainline-gate-plan.md)
* current generic completion plan:
  [versioning/nuis-0.17.0-generic-completion-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-generic-completion-plan.md)
* previous compile workflow anchor:
  [versioning/nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md)
* previous binary-compile maturity anchor:
  [versioning/nuis-0.16.0-binary-compile-maturity.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-binary-compile-maturity.md)
* previous generic-constraint coverage anchor:
  [versioning/nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)
* previous generic specialization surface audit:
  [versioning/nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)
* current generic-constraint follow-up checklist:
  [versioning/nuis-0.16.0-generic-constraint-gaps.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-gaps.md)
  current constructor truth:
  direct payload inference + transparent alias payload inference are in;
  non-transparent alias constructor inference is intentionally still narrow
* implementation-truth docs:
  [reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
  current control-flow lowering contract:
  [reference/control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)
* repo structure:
  [repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

## Current Truth By Layer

* source examples:
  [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* project examples:
  [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  current cleanup/status board:
  [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
  current shortest control-flow sync route:
  [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo) ->
  [match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo) ->
  [flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo) ->
  [post_flow_breaking_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo) ->
  [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
  current shortest control-flow async route:
  [task_async_observer_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo) ->
  [task_async_if_expression_positions_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_if_expression_positions_demo) ->
  [task_async_await_match_operand_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_await_match_operand_demo) ->
  [task_async_match_call_argument_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_match_call_argument_demo) ->
  [task_async_struct_field_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_struct_field_match_demo) ->
  [task_async_method_receiver_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_method_receiver_match_demo) ->
  [task_async_helper_expanded_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_helper_expanded_match_demo) ->
  [task_async_post_flow_shared_suffix_loop_control_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo) ->
  [task_recursive_async_shared_suffix_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_shared_suffix_demo) ->
  [task_async_while_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo) ->
  [task_async_while_post_flow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo) ->
  [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo) ->
  [task_async_while_post_flow_compound_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo)
  current structured async-while boundary:
  [reference/control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)
  current shortest generic-bound route:
  [generic_method_bound_if_binding_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_if_binding_demo) ->
  [generic_method_bound_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_nested_match_demo) ->
  [generic_method_bound_guarded_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_guarded_nested_match_demo)
  current shortest generic-helper/higher-order real project route:
  [net_http_session_loop_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo)
  with compile proof in
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)
* `std` growth path:
  [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* current tooling/workflow contract:
  [reference/std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)
  shortest rule:
  `command/subprocess -> workflow -> cli toolchain samples is now a checked-in std ladder`
  current `nuis` frontdoor consumption rule:
  `status/help/workflow/project-status/project-doctor/scheduler-view now expose one grouped frontdoor summary family`
* annotation / intrinsic future edge:
  [reference/annotation-intrinsic-stdlib-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/annotation-intrinsic-stdlib-sketch.md)
  shortest rule:
  `official annotations are preferred frontend conventions; registered nustar capability contracts are the stable truth`
* launcher / container / linker future edge:
  [reference/nuis-launcher-container-linker-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-launcher-container-linker-sketch.md)
  shortest rule:
  `the operating system launches the program; nuis owns the program's real structure`
* AOT lifecycle future edge:
  [reference/nuis-aot-lifecycle-loop-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-aot-lifecycle-loop-sketch.md)
  shortest rule:
  `host main starts the process; nuis owns the lifecycle loop`
* `nustar` ABI grain future edge:
  [reference/nustar-abi-grain-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-abi-grain-sketch.md)
  shortest rule:
  `one family package, many registered targets, one concrete built artifact`
* packaging / lifecycle responsibility edge:
  [reference/nuis-packaging-lifecycle-responsibility-map.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-packaging-lifecycle-responsibility-map.md)
  shortest rule:
  `host launches, linker assembles, container carries, lifecycle runs`
* trait/generic future edge:
  [reference/trait-generic-monomorphization-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/trait-generic-monomorphization-sketch.md)
* current address/pointer anchor:
  [versioning/nuis-0.19.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-address-pointer-mainline.md)
  shortest rule:
  `current pointer core = ref Node + ref Buffer + borrow/borrow_end + field/index surface syntax lowering to the verified builtin core`
  current surface syntax contract:
  [reference/address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)
  shortest rule:
  `ordinary .ns source should use .value/.next/.len and [index]; builtin names are lowering truth, not the recommended source spelling`
  current host-boundary ABI rule:
  [versioning/nuis-0.18.0-host-boundary-address-abi.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-host-boundary-address-abi.md)
  shortest rule:
  `internal ref is real; ordinary extern ABI is still value-only`
  current verifier/runtime contract:
  [reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
  shortest rule:
  `read through borrowed aliases is allowed; write/move/free authority remains owned-only`
  current surface-design comparison:
  [versioning/nuis-0.18.0-address-surface-options.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-address-surface-options.md)
  current owner/borrow split draft:
  [versioning/nuis-0.18.0-owned-borrowed-address-draft.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-owned-borrowed-address-draft.md)
  current internal implementation plan:
  [versioning/nuis-0.18.0-internal-address-class-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-internal-address-class-plan.md)

## Pure-To-Composite Clusters

Use these when you want the shortest explanation of how the current layers stack.

* task:
  semantic core ->
  async control ->
  async result ->
  [task_clock_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_clock_recipe.ns) /
  [task_scheduler_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_scheduler_recipe.ns) ->
  [task_cli_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_cli_recipe.ns)
* host I/O:
  [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns) ->
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns) /
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns) ->
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns) ->
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns) ->
  [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns) ->
  [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns) /
  [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* `std net`:
  grouped rule:
  `profile core -> transport edge -> syscall edge -> socket edge -> control edge -> protocol edge -> http edge -> result spine -> task spine -> session`
  detailed router:
  [stdlib/std/network/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/network/README.md)
  contract:
  [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
  current compile-closure anchor:
  `generic helper -> bridge packet/envelope -> lifted lambda -> session summary`
* `nustar` replaceability:
  frontend surface syntax may vary ->
  registration completeness / standards legality validate ->
  loader-contract binds stable capability truth
  detailed route:
  [reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md) /
  [reference/annotation-intrinsic-stdlib-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/annotation-intrinsic-stdlib-sketch.md)
* text/data:
  [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns) ->
  [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns) ->
  [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns) ->
  [text_json_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_json_recipe.ns)
* command/tooling:
  [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns) ->
  [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns) ->
  [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns) ->
  [workflow_frontdoor_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_frontdoor_runtime_recipe.ns) ->
  [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns) ->
  [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns) ->
  [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns) ->
  [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)
* filesystem metadata:
  [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns) ->
  [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns) ->
  [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns) ->
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* data/window/fabric:
  [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns) ->
  [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns) ->
  [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns) ->
  [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns) ->
  [window_fabric_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_fabric_recipe.ns)
* project-first domain profiles:
  [shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared) ->
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo) ->
  shader: `surface -> packet -> bridge` ->
  [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  and
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo) ->
  kernel: `async base -> async tensor -> tensor lane` ->
  [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  with async alignment:
  shader = `result -> policy -> fallback -> batch -> windowed`
  kernel = `result -> batch -> policy -> fallback -> windowed -> roundtrip`
  shared task = `semantic core -> async control -> async result`
  detailed route:
  [README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)
  and contract:
  [std-shader-kernel-project-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-shader-kernel-project-contract.md)
* emerging domain skeleton:
  [network-domain-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-domain-contract.md)
  ->
  [network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md)
  short rule:
  `profile core -> endpoint/timing -> host control/runtime transport -> shared helper -> result observe -> session -> result-policy/result-batch/result-windowed/policy/fallback -> batch/windowed`
  transport ladder:
  `transport result -> transport policy -> transport split -> transport batch split -> transport windowed split -> transport batch -> transport windowed -> transport/session bridge`
  ->
  [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
  ->
  [network_endpoint_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_endpoint_profile_demo)
  ->
  [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo)
  ->
  [network_host_handle_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_demo)
  ->
  [network_host_handle_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_transport_runtime_demo)
  ->
  [network_owned_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_demo)
  ->
  [network_owned_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_policy_demo)
  ->
  [network_owned_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_batch_demo)
  ->
  [network_owned_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_task_windowed_batch_demo)
  ->
  [network_owned_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_owned_transport_result_session_bridge_demo)
  ->
  [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo)
  ->
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo)
  ->
  [network_transport_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_policy_demo)
  ->
  [network_transport_result_policy_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_policy_split_demo)
  ->
  [network_transport_result_batch_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_batch_split_demo)
  ->
  [network_transport_result_windowed_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_windowed_split_demo)
  ->
  [network_transport_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_batch_demo)
  ->
  [network_transport_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_task_windowed_batch_demo)
  ->
  [network_transport_result_session_bridge_split_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_split_demo)
  ->
  [network_transport_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_session_bridge_demo)
  ->
  [network_task_async_shapes.ns](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared/network_task_async_shapes.ns)
  ->
  result ladder ->
  session/task ladder
  owned transport rule:
  `owned transport result -> owned policy -> owned batch -> owned windowed -> owned session bridge`
  result ladder:
  [network_result_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_profile_demo) ->
  [network_connect_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_result_demo) ->
  [network_accept_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_accept_result_demo) ->
  [network_connect_accept_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_policy_demo) ->
  [network_connect_accept_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_batch_demo) ->
  [network_connect_accept_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_connect_accept_task_windowed_batch_demo) ->
  [network_result_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_policy_demo) ->
  [network_result_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_batch_demo) ->
  [network_result_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_task_windowed_batch_demo) ->
  [network_result_session_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_result_session_bridge_demo)
  session/task ladder:
  [network_profile_summary_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_summary_demo) ->
  [network_profile_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_session_demo) ->
  [network_profile_task_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_policy_demo) ->
  [network_profile_task_fallback_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_fallback_demo) ->
  [network_profile_task_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_batch_demo) ->
  [network_profile_task_windowed_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_task_windowed_batch_demo)
  runtime validation probes:
  [network_loopback_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_loopback_runtime_demo) ->
  [network_host_handle_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_probe_demo) ->
  [net_tcp_send_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_send_runtime_probe_demo) ->
  [net_tcp_socket_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_runtime_probe_demo) ->
  [net_http_status_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_status_runtime_probe_demo) ->
  [net_http_client_runtime_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_runtime_probe_demo)
  exploratory protocol route:
  [net_protocol_experiment_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_protocol_experiment_recipe_demo)
  connect/accept control rule:
  `connect result -> accept result -> connect/accept policy -> connect/accept batch -> connect/accept windowed`
  shared helper rule:
  `async_session_summary -> async_policy/fallback_summary -> async_batch/windowed_summary`
  ->
  [index.toml](/Users/Shared/chroot/dev/nuislang/nustar-packages/index.toml) /
  [network.toml](/Users/Shared/chroot/dev/nuislang/nustar-packages/network.toml)

## Task-Facing `std`

Short reading rule:

* semantic core:
  [task_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_runtime_recipe.ns) ->
  [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns) ->
  [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns) ->
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns) ->
  [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* async control:
  [task_fallback_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_fallback_recipe.ns) ->
  [task_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_policy_recipe.ns) ->
  [task_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_batch_recipe.ns) ->
  [task_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_windowed_batch_recipe.ns)
* async result:
  [task_result_family_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_family_recipe.ns) ->
  [task_result_policy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_policy_recipe.ns) ->
  [task_result_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_batch_recipe.ns) ->
  [task_result_windowed_batch_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_result_windowed_batch_recipe.ns)

Frontdoor project route:

* [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo)
* [task_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo)
* [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo)
* [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo)

Best companions:

* recursion and async shape:
  [task_mutual_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_mutual_recursive_async_demo),
  [task_generic_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo),
  [task_generic_mutual_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_mutual_recursive_async_demo),
  [task_recursive_async_result_family_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_result_family_demo),
  [task_recursive_async_payload_alias_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_payload_alias_hof_demo),
  [task_async_observer_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo)
* async/control-flow crossover:
  [task_async_if_expression_positions_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_if_expression_positions_demo),
  [task_async_await_match_operand_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_await_match_operand_demo),
  [task_async_match_call_argument_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_match_call_argument_demo),
  [task_async_struct_field_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_struct_field_match_demo),
  [task_async_method_receiver_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_method_receiver_match_demo),
  [task_async_helper_expanded_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_helper_expanded_match_demo),
  [task_async_while_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo),
  [task_async_while_post_flow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo),
  [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo),
  [task_async_while_post_flow_compound_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo)
* branch, policy, and lifecycle:
  [task_lifecycle_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo),
  [task_cancel_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo),
  [task_fallback_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo),
  [task_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_policy_branch_demo),
  [task_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo),
  [task_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo),
  [task_result_family_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo),
  [task_result_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo),
  [task_result_windowed_batch_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo)
* memory and http-like session routes:
  [task_memory_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_roundtrip_demo),
  [task_memory_result_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_branch_demo),
  [task_memory_result_batch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_batch_demo),
  [task_memory_session_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo),
  [task_memory_session_packet_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_packet_demo),
  [task_httpish_response_packet_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_packet_demo),
  [task_httpish_response_slots_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_slots_demo),
  [task_httpish_session_policy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_session_policy_demo),
  [task_httpish_header_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_header_session_demo)
* observe and tooling bridge:
  [task_completed_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo),
  [task_compare_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo),
  [task_clock_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo),
  [task_cli_tooling_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo)
* future/probe route:
  [task_join_nonconsuming_probe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_join_nonconsuming_probe_demo)

Detailed route:

* contract:
  [std-task-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-task-layering-contract.md)
* source/project companions:
  [README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/README.md)

## Host I/O Mainline

Read in this order:

* execution:
  [argv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/argv_runtime_recipe.ns),
  [env_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/env_runtime_recipe.ns),
  [process_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/process_runtime_recipe.ns),
  [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns),
  [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns),
  [host_text_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/host_text_runtime_recipe.ns),
  [json_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/json_runtime_recipe.ns),
  [text_format_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/text_format_runtime_recipe.ns),
  [error_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/error_runtime_recipe.ns),
  [result_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/result_runtime_recipe.ns),
  [diagnostic_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/diagnostic_runtime_recipe.ns),
  [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns),
  [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns),
  [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns),
  [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* input observation:
  [stdin_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stdin_runtime_recipe.ns),
  [tty_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/tty_runtime_recipe.ns),
  [input_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/input_runtime_recipe.ns)
* terminal shaping:
  [io_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/io_runtime_recipe.ns),
  [terminal_io_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/terminal_io_recipe.ns),
  [line_input_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/line_input_recipe.ns),
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)

Frontdoor project route:

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)

Time/clock order inside this lane:

* [time_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/time_runtime_recipe.ns)
* [sleep_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/sleep_runtime_recipe.ns)
* [clock_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_runtime_recipe.ns)
* [clock_domain_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_domain_runtime_recipe.ns)
* [clock_test_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/clock_test_recipe.ns)

Best companions:

* source-level:
  [hello_argv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_argv_runtime_facades.ns),
  [hello_env_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_env_runtime_facades.ns),
  [hello_process_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_process_runtime_facades.ns),
  [hello_command_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_command_runtime_facades.ns),
  [hello_subprocess_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_subprocess_runtime_facades.ns),
  [hello_host_text_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_host_text_runtime_facades.ns),
  [hello_json_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_json_runtime_facades.ns),
  [hello_text_format_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_text_format_runtime_facades.ns),
  [hello_error_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_error_runtime_facades.ns),
  [hello_result_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_result_runtime_facades.ns),
  [hello_diagnostic_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_diagnostic_runtime_facades.ns),
  [hello_time_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_time_runtime_facades.ns),
  [hello_sleep_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_sleep_runtime_facades.ns),
  [hello_clock_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_runtime_facades.ns),
  [hello_clock_domain_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_domain_runtime_facades.ns),
  [hello_clock_test_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_clock_test_facades.ns),
  [hello_stdin_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_stdin_runtime_facades.ns),
  [hello_tty_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_tty_runtime_facades.ns),
  [hello_input_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_input_runtime_facades.ns),
  [hello_io_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_io_runtime_facades.ns),
  [hello_terminal_io_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_terminal_io_facades.ns)
* project-level:
  [argv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/argv_runtime_demo),
  [env_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/env_runtime_demo),
  [process_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/process_runtime_demo),
  [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo),
  [subprocess_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/subprocess_runtime_demo),
  [host_text_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/host_text_runtime_demo),
  [json_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/json_runtime_demo),
  [text_json_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_json_demo),
  [text_format_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/text_format_runtime_demo),
  [error_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/result_runtime_demo),
  [diagnostic_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/diagnostic_runtime_demo),
  [time_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/time_runtime_demo),
  [sleep_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/sleep_runtime_demo),
  [clock_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_runtime_demo),
  [clock_domain_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/clock_domain_runtime_demo),
  [stdin_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/tty_runtime_demo),
  [input_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/input_runtime_demo),
  [io_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/io_runtime_demo),
  [terminal_io_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/terminal_io_demo),
  [cli_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_session_demo),
  [cli_shell_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_shell_session_demo),
  [cli_report_session_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_report_session_demo)

Archived historical probes:

* [examples/legacy/tooling](/Users/Shared/chroot/dev/nuislang/examples/legacy/tooling)
  older low-level shell, line-input, automation, and report routes now live
  here instead of inside `examples/projects/tooling/`

Detailed route:

* companion router:
  [examples/projects/tooling/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/README.md)

## State / Location / Persistence

Read in this order:

* location roots:
  [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns),
  [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns),
  [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns)
* bundle:
  [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)
* persistence:
  [kv_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/kv_runtime_recipe.ns),
  [cache_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cache_runtime_recipe.ns),
  [config_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_runtime_recipe.ns),
  [config_cache_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/config_cache_recipe.ns)

Best companions:

* source-level:
  [hello_cwd_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns),
  [hello_temp_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_temp_runtime_facades.ns),
  [hello_home_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_home_runtime_facades.ns),
  [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns),
  [hello_kv_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_kv_runtime_facades.ns),
  [hello_cache_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cache_runtime_facades.ns),
  [hello_config_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_runtime_facades.ns),
  [hello_config_cache_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_config_cache_facades.ns)
* project-level:
  [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cwd_runtime_demo),
  [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/temp_runtime_demo),
  [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/home_runtime_demo),
  [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/location_runtime_demo),
  [kv_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/kv_runtime_demo),
  [cache_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cache_runtime_demo),
  [config_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_runtime_demo),
  [config_cache_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/config_cache_demo)

## Path / Filesystem

Use local READMEs for the long tail, but start from these anchors:

* path naming:
  [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
* path structure:
  [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns),
  [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
* path name parts:
  [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns),
  [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns),
  [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
* filesystem mutate/read/write:
  [window_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime_recipe.ns),
  [pipe_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime_recipe.ns),
  [fabric_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime_recipe.ns),
  [handle_table_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime_recipe.ns),
  [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns),
  [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns),
  [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns),
  [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns),
  [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns),
  [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns),
  [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns),
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)

Frontdoor project route:

* [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_runtime_demo)
* [file_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_runtime_demo)
* [directory_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_runtime_demo)

Best companions:

* mutation/path operations:
  [path_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_copy_demo),
  [path_rename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_rename_demo),
  [path_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_remove_demo)
* file and directory operations:
  [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_output_demo),
  [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_create_demo),
  [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_remove_demo),
  [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_stat_demo),
  [stat_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/stat_runtime_demo),
  [fs_metadata_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/fs_metadata_runtime_demo)
* runtime edge surfaces:
  [window_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/window_runtime_demo),
  [pipe_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/pipe_runtime_demo),
  [fabric_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/fabric_runtime_demo),
  [handle_table_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/handle_table_runtime_demo)

Detailed route:

* companion router:
  [examples/projects/filesystem/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/README.md)

## Cleanup Rule

When a local README and this file differ:

* use this file for the shortest current entry path
* use the local README only for area-specific detail
* treat anything outside these paths as secondary unless you are actively
  working in that subsystem
