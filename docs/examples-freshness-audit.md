# Examples Freshness Audit

This file is the current cleanup-oriented freshness audit for example routes.

It is intentionally narrower than the full mainline map. The goal here is not
to list everything that exists, but to decide:

* what should stay as front-door reading
* what should stay only as narrow regression or companion coverage
* what already feels overshadowed and should be reconsidered before the next
  cleanup pass

Companion current-state matrix:

* [versioning/nuis-alpha-0.4-system-inventory.md](versioning/nuis-alpha-0.4-system-inventory.md)
* [versioning/nuis-alpha-0.4-mainline-hardening-plan.md](versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* predecessor context:
  [versioning/nuis-0.19.0-project-capability-matrix.md](versioning/nuis-0.19.0-project-capability-matrix.md),
  [versioning/nuis-alpha-0.0.1-closeout-board.md](versioning/nuis-alpha-0.0.1-closeout-board.md)

Current policy:

* do not delete an example only because it is old
* do demote examples that no longer deserve homepage or subtree-README space
* do not archive or delete examples that still carry active doc, workflow, or
  regression value until those references are intentionally moved

Alpha hardening rule:

* this file is the example-tree companion for the repo-level `alpha-0.4.*`
  inventory and hardening plan
* use it to decide which example routes stay frontdoor, which only stay as
  companions, and which should be demoted before current docs make stronger
  claims

## Buckets

Use five buckets:

* `keep frontdoor`
  first-stop examples that should continue to appear in short routing docs
* `keep companion`
  still useful, but mainly as narrow feature, contract, or regression anchors
* `decision needed`
  plausible demotion/archive candidates, but still referenced by docs, sketches,
  or workflow expectations
* `archived`
  older routes intentionally moved under `examples/legacy/` because they still
  have historical value, but no longer belong in current project routers
* `stale wording only`
  the example itself is fine; the problem was mostly README emphasis or
  description drift

## Subtree Board

This section is the short operational board for example cleanup. Use it to see
which subtree already has a clear frontdoor/companion story, and which one
still needs structural work.

### Tooling

Status:

* `done for current pass`

Completed in this pass:

* frontdoor route narrowed to `cli_runtime_demo`,
  `command_runtime_demo`, and `workflow_runtime_demo`
* companion routing updated to include missing current examples such as
  `argv_runtime_demo`, `env_runtime_demo`, and `text_json_demo`
* older low-level probes were retired from the checked-in examples tree
* `current-mainline-map`, tooling README, self-hosted gate planning, and
  tooling workflow contract now all point at the same structure

Next likely work:

* only revisit if another current tooling route turns out to be probe-only or
  if more historical material needs to move under `legacy/`

### Filesystem

Status:

* `done for current pass`

Completed in this pass:

* frontdoor route kept narrow around `path_runtime_demo`,
  `file_runtime_demo`, and `directory_runtime_demo`
* companion routing now explicitly includes current mutate/output examples such
  as `path_copy_demo`, `path_rename_demo`, `path_remove_demo`,
  `file_output_demo`, and `directory_remove_demo`
* file read/write/copy/roundtrip/output and directory create/remove demos now
  consume `StdFsContracts` through `std=workspace` and return process-style
  `fs_ok` / `fs_error` exits
* filesystem report, report-to-file, and filesystem/console report demos now
  act as std contract consumers instead of raw probe-total examples
* `current-mainline-map`, filesystem README, and this audit now agree on the
  current route

Next likely work:

* keep the new frontdoor-versus-micro-probe split stable before deciding
  whether the dense `path_*` family deserves a grouped subrouter or a future
  legacy/probe bucket
* continue the std contract-consumer pattern into any remaining filesystem
  companions only when the example can either run against temp-backed host
  paths or clearly stays labeled as lowering-only

### Domains

Status:

* `done for current pass`

Completed in this pass:

* frontdoor domain route remains `shader_profile_demo`,
  `kernel_profile_demo`, `network_profile_demo`, and
  `net_http_client_get_recipe_demo`
* network runtime validation probes are now explicitly called out as validation
  material rather than current front-door reading
* experiment-labeled network routes are now explicitly treated as exploratory
  instead of equal-entry recommendations
* `current-mainline-map`, domains README, and this audit now distinguish
  ladder routes from probe/experiment routes

Next likely work:

* keep the new `frontdoor -> companion -> validation-only -> exploration-only`
  split stable across docs before deciding whether runtime probes should move
  under a dedicated validation subtree

### Task

Status:

* `done for current pass`

Completed in this pass:

* frontdoor route remains `task_runtime_demo`, `task_recursive_async_demo`,
  `task_result_policy_branch_demo`, and `task_status_observe_demo`
* task README and repo-level mainline map now include the missing current
  companion families instead of only a partial subset
* `task_join_nonconsuming_probe_demo` is now explicitly treated as
  future/probe material instead of reading like an ordinary companion

Next likely work:

* review whether task probes should remain inside the main task tree or move
  behind a narrower probe/archive router once the GLM and hot-sync reference
  docs are easier to retarget

### State

Status:

* `active`

Current assessment:

* the control-flow route is strong and still belongs in current docs
* the subtree is dense enough that several narrow branch/inference micro-demos
  should probably stay companion-only instead of reading like equal-entry
  routes
* generics/control-flow crossover examples are current, but they should keep
  reading as a guided ladder instead of a flat inventory

Next likely work:

* narrow the README-first state route more aggressively around one sync ladder,
  one recursion anchor, and one generic/control-flow ladder
* identify any micro-demos that no longer carry distinct regression or doc
  value
* keep the long-tail state demos as companion coverage unless they become the
  strongest evidence for a current contract claim

### Source `.ns`

Status:

* `active`

Current assessment:

* the `.ns` tree still works well as a source-surface anchor set
* its best current role is narrow semantic anchoring, not competing with
  project-route onboarding
* the core/type/memory/ffi frontdoor is already recognizable, but the audit
  should eventually classify more of the long tail as companion-only

Next likely work:

* continue narrowing the source `.ns` first-stop route around one
  basic-language ladder, one ownership/task ladder, and one host-facade ladder
* demote any single-file demo that now exists mainly to mirror a stronger
  project-route example

## Alpha-0.4 Result

Operational note:

* this is not yet a full archive plan for every subtree
* for `alpha-0.4.*`, the important thing is that frontdoor routes match tested
  workflow and runtime probes before the remaining long tail is fully
  reclassified

### Keep Frontdoor

These should remain in the shortest-path docs.

* top-level projects:
  [window_controls_demo](../examples/projects/window_controls_demo),
  [kernel_tensor_demo](../examples/projects/kernel_tensor_demo)
* task:
  [task_runtime_demo](../examples/projects/task/task_runtime_demo),
  [task_recursive_async_demo](../examples/projects/task/task_recursive_async_demo),
  [task_result_policy_branch_demo](../examples/projects/task/task_result_policy_branch_demo),
  [task_status_observe_demo](../examples/projects/task/task_status_observe_demo)
* tooling:
  [cli_runtime_demo](../examples/projects/tooling/cli_runtime_demo),
  [command_runtime_demo](../examples/projects/tooling/command_runtime_demo),
  [workflow_runtime_demo](../examples/projects/tooling/workflow_runtime_demo)
* filesystem:
  [path_runtime_demo](../examples/projects/filesystem/path_runtime_demo),
  [file_runtime_demo](../examples/projects/filesystem/file_runtime_demo),
  [directory_runtime_demo](../examples/projects/filesystem/directory_runtime_demo)
* domains:
  [shader_profile_demo](../examples/projects/domains/shader_profile_demo),
  [kernel_profile_demo](../examples/projects/domains/kernel_profile_demo),
  [network_profile_demo](../examples/projects/domains/network_profile_demo),
  [net_http_client_get_recipe_demo](../examples/projects/domains/net_http_client_get_recipe_demo)
* source/YIR anchors:
  [hello_world.ns](../examples/ns/core/hello_world.ns),
  [hello_ref_struct.ns](../examples/ns/types/hello_ref_struct.ns),
  [hello_task_glm_value_path.ns](../examples/ns/memory/hello_task_glm_value_path.ns),
  [hello_yir.yir](../examples/yir/demos/hello_yir.yir),
  [data_fabric_demo.yir](../examples/yir/data/data_fabric_demo.yir),
  [kernel_tensor_demo.yir](../examples/yir/kernel/kernel_tensor_demo.yir)

### Keep Companion

These are good examples, but they are better treated as subtree-local anchors
than as front-door reading.

Current control-flow reading rule:

* sync route first:
  [chained_while_demo](../examples/projects/state/chained_while_demo) ->
  [match_branching_while_demo](../examples/projects/state/match_branching_while_demo) ->
  [flow_continuing_while_demo](../examples/projects/state/flow_continuing_while_demo) ->
  [post_flow_breaking_while_demo](../examples/projects/state/post_flow_breaking_while_demo) ->
  [post_flow_branching_continuing_while_demo](../examples/projects/state/post_flow_branching_continuing_while_demo)
* async route next:
  [task_async_observer_bridge_demo](../examples/projects/task/task_async_observer_bridge_demo) ->
  [task_async_if_expression_positions_demo](../examples/projects/task/task_async_if_expression_positions_demo) ->
  [task_async_await_match_operand_demo](../examples/projects/task/task_async_await_match_operand_demo) ->
  [task_async_match_call_argument_demo](../examples/projects/task/task_async_match_call_argument_demo) ->
  [task_async_struct_field_match_demo](../examples/projects/task/task_async_struct_field_match_demo) ->
  [task_async_method_receiver_match_demo](../examples/projects/task/task_async_method_receiver_match_demo) ->
  [task_async_helper_expanded_match_demo](../examples/projects/task/task_async_helper_expanded_match_demo) ->
  [task_async_while_flow_cond_demo](../examples/projects/task/task_async_while_flow_cond_demo) ->
  [task_async_while_post_flow_demo](../examples/projects/task/task_async_while_post_flow_demo) ->
  [task_async_while_post_flow_cond_demo](../examples/projects/task/task_async_while_post_flow_cond_demo) ->
  [task_async_while_post_flow_compound_demo](../examples/projects/task/task_async_while_post_flow_compound_demo)
* generic route after that:
  [generic_method_bound_if_binding_demo](../examples/projects/state/generic_method_bound_if_binding_demo) ->
  [generic_method_bound_nested_match_demo](../examples/projects/state/generic_method_bound_nested_match_demo) ->
  [generic_method_bound_guarded_nested_match_demo](../examples/projects/state/generic_method_bound_guarded_nested_match_demo)

* task recursion and specialization:
  [task_mutual_recursive_async_demo](../examples/projects/task/task_mutual_recursive_async_demo),
  [task_generic_recursive_async_demo](../examples/projects/task/task_generic_recursive_async_demo),
  [task_generic_mutual_recursive_async_demo](../examples/projects/task/task_generic_mutual_recursive_async_demo),
  [task_recursive_async_result_family_demo](../examples/projects/task/task_recursive_async_result_family_demo),
  [task_recursive_async_payload_alias_hof_demo](../examples/projects/task/task_recursive_async_payload_alias_hof_demo),
  [task_async_observer_bridge_demo](../examples/projects/task/task_async_observer_bridge_demo),
  [task_async_if_expression_positions_demo](../examples/projects/task/task_async_if_expression_positions_demo),
  [task_async_await_match_operand_demo](../examples/projects/task/task_async_await_match_operand_demo),
  [task_async_match_call_argument_demo](../examples/projects/task/task_async_match_call_argument_demo),
  [task_async_struct_field_match_demo](../examples/projects/task/task_async_struct_field_match_demo),
  [task_async_method_receiver_match_demo](../examples/projects/task/task_async_method_receiver_match_demo),
  [task_async_helper_expanded_match_demo](../examples/projects/task/task_async_helper_expanded_match_demo),
  [task_async_while_flow_cond_demo](../examples/projects/task/task_async_while_flow_cond_demo),
  [task_async_while_post_flow_demo](../examples/projects/task/task_async_while_post_flow_demo),
  [task_async_while_post_flow_cond_demo](../examples/projects/task/task_async_while_post_flow_cond_demo),
  [task_async_while_post_flow_compound_demo](../examples/projects/task/task_async_while_post_flow_compound_demo)
* task timeout/fallback/batch families:
  [task_lifecycle_branch_demo](../examples/projects/task/task_lifecycle_branch_demo),
  [task_fallback_branch_demo](../examples/projects/task/task_fallback_branch_demo),
  [task_policy_branch_demo](../examples/projects/task/task_policy_branch_demo),
  [task_batch_branch_demo](../examples/projects/task/task_batch_branch_demo),
  [task_windowed_batch_branch_demo](../examples/projects/task/task_windowed_batch_branch_demo),
  [task_result_family_branch_demo](../examples/projects/task/task_result_family_branch_demo),
  [task_result_batch_branch_demo](../examples/projects/task/task_result_batch_branch_demo),
  [task_result_windowed_batch_branch_demo](../examples/projects/task/task_result_windowed_batch_branch_demo)
* task memory/http-like routes:
  [task_memory_roundtrip_demo](../examples/projects/task/task_memory_roundtrip_demo),
  [task_memory_result_branch_demo](../examples/projects/task/task_memory_result_branch_demo),
  [task_memory_result_batch_demo](../examples/projects/task/task_memory_result_batch_demo),
  [task_memory_session_policy_demo](../examples/projects/task/task_memory_session_policy_demo),
  [task_memory_session_packet_demo](../examples/projects/task/task_memory_session_packet_demo),
  [task_httpish_response_packet_demo](../examples/projects/task/task_httpish_response_packet_demo),
  [task_httpish_session_policy_demo](../examples/projects/task/task_httpish_session_policy_demo),
  [task_httpish_response_slots_demo](../examples/projects/task/task_httpish_response_slots_demo),
  [task_httpish_header_session_demo](../examples/projects/task/task_httpish_header_session_demo)
* task observe/clock:
  [task_completed_observe_demo](../examples/projects/task/task_completed_observe_demo),
  [task_compare_observe_demo](../examples/projects/task/task_compare_observe_demo),
  [task_clock_observe_demo](../examples/projects/task/task_clock_observe_demo),
  [task_scheduler_observe_demo](../examples/projects/task/task_scheduler_observe_demo)
* tooling companions:
  [argv_runtime_demo](../examples/projects/tooling/argv_runtime_demo),
  [env_runtime_demo](../examples/projects/tooling/env_runtime_demo),
  [process_runtime_demo](../examples/projects/tooling/process_runtime_demo),
  [subprocess_runtime_demo](../examples/projects/tooling/subprocess_runtime_demo),
  [host_text_runtime_demo](../examples/projects/tooling/host_text_runtime_demo),
  [json_runtime_demo](../examples/projects/tooling/json_runtime_demo),
  [text_json_demo](../examples/projects/tooling/text_json_demo),
  [text_format_runtime_demo](../examples/projects/tooling/text_format_runtime_demo),
  [error_runtime_demo](../examples/projects/tooling/error_runtime_demo),
  [result_runtime_demo](../examples/projects/tooling/result_runtime_demo),
  [result_diagnostic_demo](../examples/projects/tooling/result_diagnostic_demo),
  [input_runtime_demo](../examples/projects/tooling/input_runtime_demo),
  [io_runtime_demo](../examples/projects/tooling/io_runtime_demo),
  [stdin_runtime_demo](../examples/projects/tooling/stdin_runtime_demo),
  [tty_runtime_demo](../examples/projects/tooling/tty_runtime_demo),
  [terminal_io_demo](../examples/projects/tooling/terminal_io_demo),
  [time_runtime_demo](../examples/projects/tooling/time_runtime_demo),
  [sleep_runtime_demo](../examples/projects/tooling/sleep_runtime_demo),
  [clock_runtime_demo](../examples/projects/tooling/clock_runtime_demo),
  [clock_domain_runtime_demo](../examples/projects/tooling/clock_domain_runtime_demo),
  [cli_session_demo](../examples/projects/tooling/cli_session_demo),
  [cli_shell_session_demo](../examples/projects/tooling/cli_shell_session_demo),
  [cli_report_session_demo](../examples/projects/tooling/cli_report_session_demo)
* filesystem companions:
  standard-contract smoke set:
  [file_read_demo](../examples/projects/filesystem/file_read_demo),
  [file_write_demo](../examples/projects/filesystem/file_write_demo),
  [file_copy_demo](../examples/projects/filesystem/file_copy_demo),
  [file_roundtrip_demo](../examples/projects/filesystem/file_roundtrip_demo),
  [file_output_demo](../examples/projects/filesystem/file_output_demo),
  [directory_create_demo](../examples/projects/filesystem/directory_create_demo),
  [directory_remove_demo](../examples/projects/filesystem/directory_remove_demo),
  report consumers:
  [filesystem_report_demo](../examples/projects/filesystem/filesystem_report_demo),
  [filesystem_report_file_demo](../examples/projects/filesystem/filesystem_report_file_demo),
  [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo),
  remaining filesystem companions:
  [fs_metadata_runtime_demo](../examples/projects/filesystem/fs_metadata_runtime_demo),
  [stat_runtime_demo](../examples/projects/filesystem/stat_runtime_demo),
  [directory_stat_demo](../examples/projects/filesystem/directory_stat_demo),
  [path_copy_demo](../examples/projects/filesystem/path_copy_demo),
  [path_rename_demo](../examples/projects/filesystem/path_rename_demo),
  [path_remove_demo](../examples/projects/filesystem/path_remove_demo),
  [window_runtime_demo](../examples/projects/filesystem/window_runtime_demo),
  [pipe_runtime_demo](../examples/projects/filesystem/pipe_runtime_demo),
  [fabric_runtime_demo](../examples/projects/filesystem/fabric_runtime_demo),
  [handle_table_runtime_demo](../examples/projects/filesystem/handle_table_runtime_demo)
* state companions:
  the majority of `examples/projects/state/*` fit here:
  loop families, match families, lambda families, recursive call-graph demos,
  generic bound demos, and GLM traversal demos are still useful, but should be
  treated as narrow coverage anchors instead of homepage material
* domains companions:
  the majority of `examples/projects/domains/*` fit here:
  shader/kernel profile ladders, network result/transport ladders, and recipe
  ladders are still active, but should be chosen by ladder instead of read as a
  giant flat inventory

### Decision Needed

These are the strongest candidates for future demotion, archive moves, or
wording cleanup, but they should not be deleted blindly yet.

Next-batch candidate board:

* `task probe candidate`
  path:
  [task_join_nonconsuming_probe_demo](../examples/projects/task/task_join_nonconsuming_probe_demo)
  current role:
  probe-only
  suggested next action:
  keep in tree, but eventually move behind a narrower task probe/archive route
  once GLM and hot-sync docs stop depending on the current path
  blocker:
  still referenced by forward-looking task/GLM material
* `network validation probe cluster`
  paths:
  [network_host_handle_runtime_probe_demo](../examples/projects/domains/network_host_handle_runtime_probe_demo),
  [network_loopback_runtime_demo](../examples/projects/domains/network_loopback_runtime_demo),
  [net_tcp_send_runtime_probe_demo](../examples/projects/domains/net_tcp_send_runtime_probe_demo),
  [net_tcp_socket_runtime_probe_demo](../examples/projects/domains/net_tcp_socket_runtime_probe_demo),
  [net_http_status_runtime_probe_demo](../examples/projects/domains/net_http_status_runtime_probe_demo),
  [net_http_client_runtime_probe_demo](../examples/projects/domains/net_http_client_runtime_probe_demo)
  current role:
  validation-only
  suggested next action:
  keep as a visible cluster for now, then consider moving under a dedicated
  `validation/` or `probes/` route once doc references are easier to isolate
  blocker:
  still used by runtime-host verification and network validation discussion
* `network experiment candidate`
  path:
  [net_protocol_experiment_recipe_demo](../examples/projects/domains/net_protocol_experiment_recipe_demo)
  current role:
  exploration-only
  suggested next action:
  keep named as experiment, and eventually move beside other explicitly
  exploratory material instead of the ordinary domain ladders
  blocker:
  still useful as a living design-space marker while protocol layering remains
  active
* `kernel explicit target-config long tail`
  paths:
  `examples/projects/domains/kernel_*/*kernel_unit.ns`,
  `examples/ns/demos/kernel_*.ns`
  current role:
  companion-only cleanup debt
  suggested next action:
  keep frontdoor kernel routes on the auto-materialized registered-ABI target
  config path, then migrate the long tail in a focused pass instead of mixing
  it into unrelated docs cleanup
  blocker:
  many files still carry narrow compile/probe value, and only the frontdoor
  routes need to be build-clean for this pass
* `filesystem path micro-probe family`
  paths:
  `examples/projects/filesystem/path_*_demo`
  current role:
  companion-only micro-probes
  suggested next action:
  keep the frontdoor trio where it is, but consider grouping the remaining
  `path_*` family under a narrower subrouter or future probe/archive route
  blocker:
  the family is dense, but not yet cleanly separable by doc usage
* `single-file mirror-heavy ns demos`
  paths:
  [window_controls_demo.ns](../examples/ns/demos/window_controls_demo.ns),
  [shader_profile_demo.ns](../examples/ns/demos/shader_profile_demo.ns),
  [kernel_profile_demo.ns](../examples/ns/demos/kernel_profile_demo.ns)
  current role:
  mirror-only
  suggested next action:
  keep as compact source mirrors, but avoid promoting their deeper siblings as
  equal-entry routes; later consider a stronger `mirror-only` local grouping if
  the subtree keeps growing
  blocker:
  still useful as compact source-shaped mirrors of the main project routes

The candidate board above is now the authoritative detailed list for this
bucket.

### Archived

These routes were intentionally removed from the checked-in examples tree after
their current replacements became clearer and better aligned with the current
tooling front door.

* `command_shell_demo`
  replaced in practice by `command_runtime_demo`, `workflow_runtime_demo`, and
  `cli_runtime_demo`
* `automation_runtime_demo`
  replaced in practice by workflow and CLI-oriented routes
* `line_input_demo`
  replaced in practice by `input_runtime_demo` and terminal/CLI session routes
* `report_runtime_demo`
  replaced in practice by diagnostic/result/report session routes

### Stale Wording Only

These were the clearest first-pass wording problems addressed in the current
cleanup:

* [examples/README.md](../examples/README.md)
* [examples/projects/README.md](../examples/projects/README.md)
* [examples/projects/task/README.md](../examples/projects/task/README.md)
* [examples/projects/tooling/README.md](../examples/projects/tooling/README.md)
* [examples/projects/filesystem/README.md](../examples/projects/filesystem/README.md)
* [examples/projects/domains/README.md](../examples/projects/domains/README.md)

The issue in these files was mostly:

* overly long inventory-style routing
* too many examples treated as equal-entry recommendations
* README-level emphasis lagging behind the actual current mainline

## Next Pass

The next cleanup pass should focus on one subtree at a time.

Recommended order:

1. `examples/projects/domains/`
   execute the network validation/exploration split from the candidate board
   once their reference-doc usage is more isolated
2. `examples/projects/filesystem/`
   decide whether the dense `path_*` micro-probe family should keep living as a
   flat set of companions or gain a narrower grouped subrouter
3. `examples/projects/task/`
   revisit when GLM/hot-sync documentation is ready for a narrower dedicated
   probe/archive route for `task_join_nonconsuming_probe_demo`
4. `examples/projects/tooling/`
   revisit only if more probe-style or historical routes accumulate and the
   current `legacy/tooling` split stops being sufficient
5. `examples/ns/demos/`
   revisit if the mirror-only subtree grows further and needs a stronger local
   mirror/archive split
