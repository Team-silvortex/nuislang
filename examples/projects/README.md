# `nuis` Projects

This folder contains multi-file `nuis` project examples driven by
`nuis.toml`.

The important reading rule now is:

* not every project here is meant to be a first-stop example
* this folder mixes:
  - frontdoor showcase projects
  - narrow feature/regression companions
  - probe/validation routes
  - domain recipe ladders
* if you want the current shortest mainline path first, start with
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* if you want the current cleanup/status board for project routes, use
  [docs/examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)

Current source-style rule:

* project `.ns` modules now follow the same address surface spelling as the
  single-file examples: `ptr.value`, `ptr.next`, `buffer.len`, `buffer[index]`
* explicit builtin helper names are now mainly reserved for lowering/NIR/YIR
  discussion and implementation-facing docs

## What This Folder Is For

Project mode is still the main checked-in route for real end-to-end source
programs in this repository.

Compared with a single `.ns` file, project mode currently gives you:

* `nuis.toml` manifests
* multi-file `cpu / data / shader / kernel` splits
* project-level `links`
* ABI locking or auto-resolution
* project-level compiler and build metadata outputs

Current practical rule:

* use projects to understand the real compile workflow
* use single-file `.ns` examples for narrow language surface reading
* use handwritten `YIR` only when you want the lower semantic layer directly

## Start Here

These are the best current first-entry projects:

* [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  current `cpu + data + shader` showcase and the main documented project flow
* [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
  current `cpu + data + kernel` showcase for the project pipeline

Useful first commands:

```bash
cargo run -p nuis -- project-doctor examples/projects/window_controls_demo
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- test examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
```

## Current Layout

The folder is intentionally split by role:

* showcase projects stay at the root:
  - [window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
  - [kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* grouped companions live under:
  - [task](/Users/Shared/chroot/dev/nuislang/examples/projects/task)
  - [tooling](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling)
  - [state](/Users/Shared/chroot/dev/nuislang/examples/projects/state)
  - [filesystem](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem)
  - [domains](/Users/Shared/chroot/dev/nuislang/examples/projects/domains)
* shared domain helper modules live under:
  - [domains/shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared)

## Pick By Goal

If you are trying to orient quickly, use one representative route instead of
reading an entire subtree.

* control flow / recursion / generics:
  [state](/Users/Shared/chroot/dev/nuislang/examples/projects/state)
  Start with:
  [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo),
  [match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo),
  [tail_recursive_sum_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_sum_demo),
  [generic_method_bound_if_binding_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_if_binding_demo)
* async tasks / task-result control:
  [task](/Users/Shared/chroot/dev/nuislang/examples/projects/task)
  Start with:
  [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo),
  [task_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo),
  [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo)
* CLI / workflow / host tooling:
  [tooling](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling)
  Start with:
  [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo),
  [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo),
  [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)
* path / file / directory surfaces:
  [filesystem](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem)
  Start with:
  [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_runtime_demo),
  [file_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_runtime_demo),
  [directory_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_runtime_demo)
* profile and recipe ladders across domains:
  [domains](/Users/Shared/chroot/dev/nuislang/examples/projects/domains)
  Start with:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo),
  [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo),
  [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo),
  [net_http_client_get_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_get_recipe_demo)

## Control-Flow Mainline

If you want the current project-backed control-flow story specifically, use
this order instead of browsing the whole tree.

* sync control-flow frontdoor:
  [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo) ->
  [match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo) ->
  [flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo) ->
  [post_flow_breaking_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo) ->
  [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
* async control-flow frontdoor:
  [task_async_observer_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo) ->
  [task_async_while_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo) ->
  [task_async_while_post_flow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo) ->
  [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo) ->
  [task_async_while_post_flow_compound_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo)
* generic/control-flow crossover:
  [generic_method_bound_if_binding_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_if_binding_demo) ->
  [generic_method_bound_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_nested_match_demo) ->
  [generic_method_bound_guarded_nested_match_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_method_bound_guarded_nested_match_demo)

Practical reading rule:

* use the sync route to understand the ordinary lowering families first
* use the async route immediately after if you want to see the same ideas
  survive `await`, observer values, and async carries
* use the generic route after that when the question is binding visibility or
  method-bound validation rather than loop form alone

## Freshness Rule

This folder has grown large enough that “present in the tree” no longer means
“equally recommended”.

Use this rule:

* root showcase projects are the strongest current frontdoor entrypoints
* one example from each grouped subtree is usually better than reading a dozen
  nearby companions
* many grouped demos are intentionally narrow compile or regression anchors
* some grouped demos are better read as probes or validation routes rather than
  as standard onboarding examples
* if a local README and the current mainline map feel different, prefer the
  current mainline map first

Practical consequence:

* keep old-but-still-useful companions when they still carry regression value
* keep probe routes when they still support runtime validation or design docs
* stop treating long directory inventories as the first reading route

## Artifact Bundles

Checked-in canonical build outputs still live under:

* [examples/bins](/Users/Shared/chroot/dev/nuislang/examples/bins)

The two current canonical checked-in bundles remain:

* [window_controls_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/window_controls_demo_project/window_controls_demo)
* [kernel_tensor_demo_project](/Users/Shared/chroot/dev/nuislang/examples/bins/kernel_tensor_demo_project/kernel_tensor_demo)
