# `nuis` 0.19.0 Project Capability Matrix

This file is the short current-state matrix for project-backed examples and
compile anchors in the `0.19.0` line.

It exists to answer one practical question quickly:

`which project/example families already carry current mainline proof, and what role does each family play?`

## Short Rule

Read this file as a project-proof map, not as a full inventory.

The main question is not only:

`which example exists?`

The main question is:

`which checked-in project routes already prove one whole slice of the current language/workflow story?`

## Current Combined Capability Matrix

### State and control-flow projects

Current truth:

* state projects remain the shortest checked-in proof for synchronous
  control-flow composition
* the route already covers chained `while`, branching `match`, and post-flow
  continuation shapes
* generic method-bound control-flow companions already live nearby

Primary anchors:

* [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo)
* [match_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo)
* [flow_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo)
* [post_flow_breaking_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo)
* [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
* [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

Short rule:

`state projects are still the shortest project-backed proof for synchronous control-flow truth`

### Task and async projects

Current truth:

* task projects remain the mainline async/control-flow proving ground
* recursion, result-family, observe/status, and async control-flow shapes are
  already represented
* task projects also carry several bridge routes into memory- and
  http-like/session-style shapes
* staged thread/lock handle routing now has a project-backed compile anchor

Primary anchors:

* [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo)
* [task_thread_mutex_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_thread_mutex_demo)
* [task_recursive_async_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo)
* [task_result_policy_branch_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo)
* [task_status_observe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo)
* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

Short rule:

`task projects are the current async/recursion backbone, and now also the staged thread/lock project anchor, of the mainline`

### Memory and address projects

Current truth:

* memory/address project anchors carry the believable current proof for
  pointer/borrow/address surface closure
* this lane is where source-facing pointer syntax and lowered memory-model
  truth meet real project compile gates

Primary anchors:

* [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* [address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

Short rule:

`memory currentness claims should still pass through the project-backed memory anchor, not only through source snippets`

### Shader and kernel domain projects

Current truth:

* shader/kernel domain projects carry the helper-mediated non-CPU profile story
* shared helper modules already connect domain profiles to async/task-shaped
  support lanes
* shader/kernel are no longer just isolated demos; they are part of the current
  domain-backed mainline story

Primary anchors:

* [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
* [shared](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shared)
* [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)

Short rule:

`shader and kernel projects are the current proof that helper-mediated domain profiles really belong to the mainline`

### Network and HTTP/session projects

Current truth:

* network examples are now read as ladders, not as one flat subtree
* current front-door network routes already connect profile, request/response,
  session, and result/task bridges
* the compile anchor is not only “network exists”, but “network/http/session
  closure still compiles as part of the mainline”

Primary anchors:

* [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
* [net_http_client_get_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_get_recipe_demo)
* [net_http_session_loop_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo)
* [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

Short rule:

`network projects now prove a layered route from profile to session bridge, not just a pocket of exploratory demos`

### Tooling and workflow projects

Current truth:

* tooling examples now have an explicit frontdoor and companion structure
* command/subprocess/workflow/CLI/report lanes already exist as checked-in
  project examples
* this lane is the project-backed companion to the std tooling workflow ladder

Primary anchors:

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)
* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)

Short rule:

`tooling projects are now frontdoor workflow-reading material, not only low-level experiments`

## Current Proven Routes

These are the shortest “already real together” project routes worth remembering.

### Route A

`state route -> control-flow compile anchor`

Anchors:

* [chained_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo)
* [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

### Route B

`task route -> async/recursive compile anchor`

Anchors:

* [task_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo)
* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

### Route C

`domain profile -> helper-mediated closure -> project test anchor`

Anchors:

* [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* [kernel_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/kernel_profile_demo)
* [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)

### Route D

`network profile -> http/session route -> network compile anchor`

Anchors:

* [network_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_profile_demo)
* [net_http_session_loop_bridge_recipe_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo)
* [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

### Route E

`tooling frontdoor -> std tooling ladder companion`

Anchors:

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)
* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)

## Current Boundaries

These boundaries matter because they keep the project story honest.

* not every project demo is frontdoor reading; many remain companion or probe
  routes
* domains should be read as ladders, especially network, not as one flat list
* project anchors prove current mainline closure, but they do not replace
  lower-level frontend or lowering regression families
* freshness routing and capability routing are related, but they serve
  different jobs

Short rule:

`project currentness means named proof routes plus named anchors, not “every example is equally current”`

## Usage Rule

When updating examples, project compile anchors, or project routing docs:

1. identify which project family carries the change
2. update the matching route above first
3. then update subtree README, freshness audit, or regression matrix as needed

If a claimed current project route cannot be placed on this matrix, it probably
is not yet described clearly enough as mainline proof.
