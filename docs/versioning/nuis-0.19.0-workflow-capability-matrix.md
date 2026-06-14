# `nuis` 0.19.0 Workflow Capability Matrix

This file is the short current-state matrix for the `0.19.0` workflow and
compile frontdoor line.

It exists to answer one practical question quickly:

`which workflow pieces are already proved together as one current route, instead of only existing as separate commands or docs?`

## Short Rule

Read this file as a route-combination map.

The main question is not only:

`which command exists?`

The main question is:

`which command families, checked-in docs, std lanes, and regression anchors already form one teachable path?`

## Current Combined Capability Matrix

### CLI frontdoor orientation

Current truth:

* `status` and `help` act as the orientation pair
* `workflow` acts as route classification
* `project-doctor`, `project-status`, and `scheduler-view` act as grouped
  preflight detail
* `check`, `test`, `build`, and `release-check` remain the action spine

Primary anchors:

* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)

Short rule:

`the current CLI story is now a frontdoor family, not a loose command list`

### Compiler-facing compile order

Current truth:

* the frontend route is described as one ordered path
* helper/alias assembly, higher-order expansion, generic validation, lowering,
  and project-aware continuity are all part of that same path
* the current docs already describe compiler-facing order separately from
  source-facing CLI order

Primary anchors:

* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-frontend-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-frontend-capability-matrix.md)

Short rule:

`current compile truth is one ordered route from surface source to verifier-facing output`

### Source-style versus lowered truth

Current truth:

* source-facing address syntax is documented separately from lowered builtin
  vocabulary
* current docs already point source authors toward `.value`, `.next`, `.len`,
  and `[index]`
* lowered builtin names remain implementation-facing truth in NIR/YIR/verifier
  discussion

Primary anchors:

* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

Short rule:

`source style and lowered truth are both current, but they no longer claim to live at the same layer`

### `std` tooling workflow ladder

Current truth:

* command, subprocess, workflow, CLI, report, and automation lanes already form
  a checked-in ladder
* the ladder is strong enough to prove workflow direction inside `std`
* the remaining gap is shared authoring shape, not raw expressivity

Primary anchors:

* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)
* [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)

Short rule:

`the current std tooling lane already proves workflow authorship direction, even if it is not yet the final reusable architecture`

### Regression gates and project-backed anchors

Current truth:

* the current mainline gate names are documented honestly, even where some
  scripts still carry `0.18` history
* frontend unit families and project-backed compile anchors are already named as
  one matrix
* state, task, memory, shader, and network remain the mainline reality checks

Primary anchors:

* [nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
* [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)
* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)
* [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)
* [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

Short rule:

`the current line is defended by named checked-in gates plus named project anchors`

## Current Proven Routes

These are the shortest “already real together” routes worth remembering.

### Route A

`status/help -> workflow -> preflight trio -> action spine`

Anchors:

* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)

### Route B

`surface syntax contract -> frontend order -> project-aware lowering continuity`

Anchors:

* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-frontend-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-frontend-capability-matrix.md)

### Route C

`command/subprocess -> workflow -> CLI workflow -> build/report ladder`

Anchors:

* [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)

### Route D

`frontend/core gates -> project compile anchors -> current mainline claim`

Anchors:

* [nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)

## Current Boundaries

These boundaries matter because they keep the current story honest.

* the current std tooling lane proves direction, not final abstraction quality
* current workflow truth is broader than one shell script, but not yet a fully
  self-hosted compile pipeline
* current gate names still carry some version history, so docs must explain
  what they actually defend today
* current source-facing teaching order and compiler-facing execution order are
  related, but they should still be described separately

Short rule:

`0.19.0 workflow maturity means clearer route ownership, not pretending every layer is already final`

## Usage Rule

When updating current workflow/mainline docs:

1. identify whether the change affects frontdoor reading order, compiler order,
   std workflow authorship, or regression gate ownership
2. update the matching route above first
3. then update the more detailed companion doc

If a new workflow claim cannot be placed on this matrix, it probably is not yet
described as current mainline truth.
