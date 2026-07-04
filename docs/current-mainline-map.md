# Current Mainline Map

This file is the canonical short reading map for the current repository spine.

If several READMEs seem to overlap, prefer this file first, then drill into the
local README for the area you are actively touching.

## Fast Reading Order

If you only need the shortest current `alpha-0.7.*` reading route, use this order:

1. [versioning/nuis-alpha-0.7-mainline-entry.md](versioning/nuis-alpha-0.7-mainline-entry.md)
2. [reference/std-mainline-layering-contract.md](reference/std-mainline-layering-contract.md)
3. [reference/toolchain-galaxy-core-boundary.md](reference/toolchain-galaxy-core-boundary.md)
4. [reference/nsld-linker-frontdoor.md](reference/nsld-linker-frontdoor.md)
5. [reference/nsld-binary-assembly-gap-map.md](reference/nsld-binary-assembly-gap-map.md)
6. [versioning/nuis-alpha-0.6-mainline-entry.md](versioning/nuis-alpha-0.6-mainline-entry.md)
7. [versioning/nuis-alpha-0.4-system-inventory.md](versioning/nuis-alpha-0.4-system-inventory.md)
8. [versioning/nuis-alpha-0.4-mainline-hardening-plan.md](versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
9. [versioning/nuis-alpha-0.4-doc-sync-inventory.md](versioning/nuis-alpha-0.4-doc-sync-inventory.md)
10. [reference/nuis-frontdoor-surface-reference.md](reference/nuis-frontdoor-surface-reference.md)
11. [reference/nuis-native-artifact-workflow.md](reference/nuis-native-artifact-workflow.md)
12. [reference/nuis-binary-format-protocol.md](reference/nuis-binary-format-protocol.md)
13. [reference/yir-tools-reference.md](reference/yir-tools-reference.md)
14. [reference/ffi-pointer-safety-boundary.md](reference/ffi-pointer-safety-boundary.md)
15. [reference/nustar-capability-split-boundary.md](reference/nustar-capability-split-boundary.md)
16. [versioning/nuis-long-range-heterogeneous-os-roadmap.md](versioning/nuis-long-range-heterogeneous-os-roadmap.md)
17. [versioning/nuis-alpha-0.1-mainline-status.md](versioning/nuis-alpha-0.1-mainline-status.md)
18. [versioning/nuis-0.20.0-abi-compile-vocabulary.md](versioning/nuis-0.20.0-abi-compile-vocabulary.md)
19. [versioning/nuis-0.20.0-std-refactor-frontdoor.md](versioning/nuis-0.20.0-std-refactor-frontdoor.md)
20. [versioning/nuis-0.20.0-compile-gap-checklist.md](versioning/nuis-0.20.0-compile-gap-checklist.md)

Short rule:

`inventory says what exists; hardening plan says what to optimize; workflow says how the route composes; predecessor docs explain how the current shape got here`

## Start Here

* repo status and current toolchain spine:
  [README.md](../README.md)
* current `alpha-0.7.*` entry:
  [versioning/nuis-alpha-0.7-mainline-entry.md](versioning/nuis-alpha-0.7-mainline-entry.md)
* predecessor `alpha-0.6.*` linker/std smoke entry:
  [versioning/nuis-alpha-0.6-mainline-entry.md](versioning/nuis-alpha-0.6-mainline-entry.md)
* `alpha-0.4.*` hardening baseline plan:
  [versioning/nuis-alpha-0.4-mainline-hardening-plan.md](versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* `alpha-0.4.*` hardening baseline inventory:
  [versioning/nuis-alpha-0.4-system-inventory.md](versioning/nuis-alpha-0.4-system-inventory.md)
* `alpha-0.4.*` documentation sync baseline:
  [versioning/nuis-alpha-0.4-doc-sync-inventory.md](versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* long-range heterogeneous OS roadmap:
  [versioning/nuis-long-range-heterogeneous-os-roadmap.md](versioning/nuis-long-range-heterogeneous-os-roadmap.md)
* predecessor `alpha-0.1.*` status:
  [versioning/nuis-alpha-0.1-mainline-status.md](versioning/nuis-alpha-0.1-mainline-status.md)
* current frontdoor surface reference:
  [reference/nuis-frontdoor-surface-reference.md](reference/nuis-frontdoor-surface-reference.md)
* current native artifact workflow:
  [reference/nuis-native-artifact-workflow.md](reference/nuis-native-artifact-workflow.md)
* current binary format protocol:
  [reference/nuis-binary-format-protocol.md](reference/nuis-binary-format-protocol.md)
* predecessor ABI vocabulary bridge into `0.20.*`:
  [versioning/nuis-0.20.0-abi-compile-vocabulary.md](versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* immediate predecessor alpha closeout set:
  [versioning/nuis-alpha-0.0.1-preflight-report.md](versioning/nuis-alpha-0.0.1-preflight-report.md),
  [versioning/nuis-alpha-0.0.1-closeout-board.md](versioning/nuis-alpha-0.0.1-closeout-board.md),
  [versioning/nuis-alpha-0.0.1-closeout-checklist.md](versioning/nuis-alpha-0.0.1-closeout-checklist.md)
* current alpha mainline boundary index:
  [reference/alpha-mainline-boundary-index.md](reference/alpha-mainline-boundary-index.md)
* current frontend-vs-CLI boundary note:
  [versioning/nuis-0.20.0-frontend-cli-boundaries.md](versioning/nuis-0.20.0-frontend-cli-boundaries.md)
* current branch-runtime lowering matrix:
  [versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md](versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md)
* current generic-validation regression matrix:
  [versioning/nuis-0.20.0-generic-validation-regression-matrix.md](versioning/nuis-0.20.0-generic-validation-regression-matrix.md)
* current receiver-generic regression matrix:
  [versioning/nuis-0.20.0-receiver-generic-regression-matrix.md](versioning/nuis-0.20.0-receiver-generic-regression-matrix.md)
* current `std` refactor frontdoor:
  [versioning/nuis-0.20.0-std-refactor-frontdoor.md](versioning/nuis-0.20.0-std-refactor-frontdoor.md)
* current compile-gap checklist:
  [versioning/nuis-0.20.0-compile-gap-checklist.md](versioning/nuis-0.20.0-compile-gap-checklist.md)
* previous minor-line anchors still worth keeping nearby:
  [versioning/nuis-0.19.0-snapshot.md](versioning/nuis-0.19.0-snapshot.md),
  [versioning/nuis-0.19.0-compile-workflow.md](versioning/nuis-0.19.0-compile-workflow.md),
  [versioning/nuis-0.19.0-workflow-capability-matrix.md](versioning/nuis-0.19.0-workflow-capability-matrix.md),
  [versioning/nuis-0.19.0-project-capability-matrix.md](versioning/nuis-0.19.0-project-capability-matrix.md),
  [versioning/nuis-0.19.0-frontend-capability-matrix.md](versioning/nuis-0.19.0-frontend-capability-matrix.md)
* historical/versioning router:
  [versioning/README.md](versioning/README.md)
* implementation-truth docs:
  [reference/README.md](reference/README.md),
  [reference/generic-diagnostic-ownership-contract.md](reference/generic-diagnostic-ownership-contract.md),
  [reference/control-flow-lowering-contract.md](reference/control-flow-lowering-contract.md)
* repo structure:
  [repo-layout.md](repo-layout.md)

Current CLI frontdoor rule:
`nuis status/help -> nuis workflow -> nuis project-doctor/project-status/scheduler-view -> check/test/build -> artifact-doctor/run-artifact -> release-check`

Current example-tree rule:
`frontdoor first -> grouped companions next -> explicit probe routes after that`

## Current Truth By Layer

Use this section as a router, not as a full inventory.

* source-level examples:
  [examples/ns/README.md](../examples/ns/README.md)
* project-level examples:
  [examples/projects/README.md](../examples/projects/README.md)
* example freshness / routing audit:
  [examples-freshness-audit.md](examples-freshness-audit.md)
* frontdoor and native artifact closure:
  [reference/nuis-frontdoor-surface-reference.md](reference/nuis-frontdoor-surface-reference.md),
  [reference/nuis-native-artifact-workflow.md](reference/nuis-native-artifact-workflow.md),
  [reference/nuis-binary-format-protocol.md](reference/nuis-binary-format-protocol.md),
  [reference/toolchain-galaxy-core-boundary.md](reference/toolchain-galaxy-core-boundary.md),
  [reference/nsld-binary-assembly-gap-map.md](reference/nsld-binary-assembly-gap-map.md)
  current native control-flow smoke gate:
  [artifact_cli.rs](../tools/nuisc/tests/artifact_cli.rs)
  current host-YIR runtime probe:
  [host_yir.rs](../crates/nuis-runtime/src/host_yir.rs)
* tool/reference surface:
  [reference/yir-tools-reference.md](reference/yir-tools-reference.md)
* std layering and tooling:
  [stdlib/std/README.md](../stdlib/std/README.md),
  [reference/std-tooling-workflow-contract.md](reference/std-tooling-workflow-contract.md)
* PixelMagic frontdoor:
  [stdlib/pixelmagic/README.md](../stdlib/pixelmagic/README.md),
  [reference/pixelmagic-mainline-contract.md](reference/pixelmagic-mainline-contract.md)
  current image-preprocess bridge:
  [reference/tooling-image-preprocess-lane.md](reference/tooling-image-preprocess-lane.md)
  current high-level tooling ladder:
  [cli_compile_workflow_demo](../examples/projects/tooling/cli_compile_workflow_demo),
  [cli_workflow_automation_demo](../examples/projects/tooling/cli_workflow_automation_demo),
  [cli_build_pipeline_demo](../examples/projects/tooling/cli_build_pipeline_demo),
  [cli_project_build_report_demo](../examples/projects/tooling/cli_project_build_report_demo)
* control-flow / memory / task / ownership:
  [reference/control-flow-lowering-contract.md](reference/control-flow-lowering-contract.md),
  [reference/nir-memory-model.md](reference/nir-memory-model.md),
  [reference/cpu-task-glm-contract.md](reference/cpu-task-glm-contract.md),
  [reference/cpu-thread-lock-boundary.md](reference/cpu-thread-lock-boundary.md),
  [reference/ffi-pointer-safety-boundary.md](reference/ffi-pointer-safety-boundary.md)
* domain and project contracts:
  [examples/projects/domains/README.md](../examples/projects/domains/README.md),
  [reference/std-net-layering-contract.md](reference/std-net-layering-contract.md),
  [reference/std-shader-kernel-project-contract.md](reference/std-shader-kernel-project-contract.md)
* capability split and future architecture edges:
  [reference/nustar-capability-split-boundary.md](reference/nustar-capability-split-boundary.md),
  [reference/annotation-intrinsic-stdlib-sketch.md](reference/annotation-intrinsic-stdlib-sketch.md),
  [reference/nuis-launcher-container-linker-sketch.md](reference/nuis-launcher-container-linker-sketch.md),
  [versioning/nuis-long-range-heterogeneous-os-roadmap.md](versioning/nuis-long-range-heterogeneous-os-roadmap.md)

## Fast Example Routes

If you want one shortest checked-in route per question, use:

* sync control-flow:
  [chained_while_demo](../examples/projects/state/chained_while_demo) ->
  [match_branching_while_demo](../examples/projects/state/match_branching_while_demo) ->
  [flow_branching_while_demo](../examples/projects/state/flow_branching_while_demo) ->
  [post_flow_branching_while_demo](../examples/projects/state/post_flow_branching_while_demo)
* async control-flow:
  [task_async_observer_bridge_demo](../examples/projects/task/task_async_observer_bridge_demo) ->
  [task_async_while_post_flow_demo](../examples/projects/task/task_async_while_post_flow_demo) ->
  [task_async_while_post_flow_cond_demo](../examples/projects/task/task_async_while_post_flow_cond_demo) ->
  [task_async_post_flow_shared_suffix_loop_control_demo](../examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo)
* generic/control-flow crossover:
  [generic_method_bound_if_binding_demo](../examples/projects/state/generic_method_bound_if_binding_demo) ->
  [generic_method_bound_guarded_nested_match_demo](../examples/projects/state/generic_method_bound_guarded_nested_match_demo)
* task/thread/lock boundary:
  [hello_thread_mutex_observe.ns](../examples/ns/memory/hello_thread_mutex_observe.ns) ->
  [task_thread_mutex_demo](../examples/projects/task/task_thread_mutex_demo)
* native artifact closure:
  [native_artifact_closure_demo](../examples/projects/tooling/native_artifact_closure_demo)
* tooling compile ladder:
  [cli_workflow_automation_demo](../examples/projects/tooling/cli_workflow_automation_demo) ->
  [cli_build_pipeline_demo](../examples/projects/tooling/cli_build_pipeline_demo) ->
  [cli_project_build_report_demo](../examples/projects/tooling/cli_project_build_report_demo) ->
  [cli_compile_workflow_demo](../examples/projects/tooling/cli_compile_workflow_demo)
* tooling image preprocess lane:
  [cli_pgm_info_demo](../examples/projects/tooling/cli_pgm_info_demo) ->
  [cli_pgm_invert_demo](../examples/projects/tooling/cli_pgm_invert_demo) ->
  [cli_pgm_threshold_demo](../examples/projects/tooling/cli_pgm_threshold_demo) ->
  [reference/tooling-image-preprocess-lane.md](reference/tooling-image-preprocess-lane.md) ->
  [examples/projects/domains/pixelmagic_packet_bridge_demo](../examples/projects/domains/pixelmagic_packet_bridge_demo) ->
  [examples/projects/domains/pixelmagic_texture_resource_demo](../examples/projects/domains/pixelmagic_texture_resource_demo) ->
  [examples/projects/domains/pixelmagic_pipeline_demo](../examples/projects/domains/pixelmagic_pipeline_demo) ->
  [examples/projects/domains/pixelmagic_render_demo](../examples/projects/domains/pixelmagic_render_demo) ->
  [reference/pixelmagic-mainline-contract.md](reference/pixelmagic-mainline-contract.md) ->
  [reference/galaxy-frontdoor-prep-sketch.md](reference/galaxy-frontdoor-prep-sketch.md) ->
  [reference/galaxy-texture-handoff-contract.md](reference/galaxy-texture-handoff-contract.md)
* std filesystem contract smoke:
  [file_read_demo](../examples/projects/tooling/file_read_demo) ->
  [file_write_demo](../examples/projects/tooling/file_write_demo) ->
  [file_copy_demo](../examples/projects/tooling/file_copy_demo) ->
  [file_output_demo](../examples/projects/tooling/file_output_demo) ->
  [file_roundtrip_demo](../examples/projects/tooling/file_roundtrip_demo) ->
  [directory_create_demo](../examples/projects/tooling/directory_create_demo) ->
  [directory_remove_demo](../examples/projects/tooling/directory_remove_demo) ->
  [filesystem_report_demo](../examples/projects/tooling/filesystem_report_demo) ->
  [filesystem_report_file_demo](../examples/projects/tooling/filesystem_report_file_demo) ->
  [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo) ->
  [path_analysis_demo](../examples/projects/tooling/path_analysis_demo) ->
  [path_copy_remove_demo](../examples/projects/tooling/path_copy_remove_demo)
  current rule:
  these are `std=workspace` contract consumers and should return process-style
  `fs_ok` / `fs_error` instead of raw probe totals
* std tooling report smoke:
  [io_runtime_demo](../examples/projects/tooling/io_runtime_demo) ->
  [terminal_io_demo](../examples/projects/tooling/terminal_io_demo) ->
  [io_report_demo](../examples/projects/tooling/io_report_demo) ->
  [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo) ->
  [benchmark_report_file_demo](../examples/projects/tooling/benchmark_report_file_demo) ->
  [result_runtime_demo](../examples/projects/tooling/result_runtime_demo) ->
  [result_diagnostic_demo](../examples/projects/tooling/result_diagnostic_demo) ->
  [text_report_builder_demo](../examples/projects/tooling/text_report_builder_demo) ->
  [text_report_json_demo](../examples/projects/tooling/text_report_json_demo) ->
  [result_enum_runtime_demo](../examples/projects/tooling/result_enum_runtime_demo)
  current rule:
  these prove cross-contract composition across filesystem, console I/O, text,
  benchmark, result/error, diagnostic, and JSON/report helpers
  current gap:
  Result enum `Ok`/`map`/value extraction, `Err`/`map_err`, and
  branch-selected struct summaries are run-backed; static known-variant pruning
  is still future control-flow hardening work
* shader/kernel showcase:
  [window_controls_demo](../examples/projects/window_controls_demo) /
  [kernel_tensor_demo](../examples/projects/kernel_tensor_demo)
* WitSage kernel-facing ML route:
  [stdlib/witsage/README.md](../stdlib/witsage/README.md) ->
  [examples/projects/domains/witsage_kernel_demo](../examples/projects/domains/witsage_kernel_demo) ->
  [examples/projects/domains/witsage_classifier_demo](../examples/projects/domains/witsage_classifier_demo)
* network/domain route:
  [network_profile_demo](../examples/projects/domains/network_profile_demo) ->
  [net_http_session_loop_bridge_recipe_demo](../examples/projects/domains/net_http_session_loop_bridge_recipe_demo)

## Deep Routers

When the short routes above are not enough, jump straight to the dedicated
router instead of using this file as a long catalog:

* examples tree and freshness:
  [examples/projects/README.md](../examples/projects/README.md),
  [examples/ns/README.md](../examples/ns/README.md),
  [examples-freshness-audit.md](examples-freshness-audit.md)
* std recipe ladders:
  [stdlib/std/README.md](../stdlib/std/README.md),
  [stdlib/std/network/README.md](../stdlib/std/network/README.md)
* domain route catalogs:
  [examples/projects/domains/README.md](../examples/projects/domains/README.md)
* task-facing current contracts:
  [reference/cpu-task-contract.md](reference/cpu-task-contract.md),
  [reference/cpu-task-memory-contract.md](reference/cpu-task-memory-contract.md),
  [reference/cpu-task-glm-contract.md](reference/cpu-task-glm-contract.md)

## Cleanup Rule

When a local README and this file differ:

* use this file for the shortest current entry path
* use the local README only for area-specific detail
* treat anything outside these paths as secondary unless you are actively
  working in that subsystem
