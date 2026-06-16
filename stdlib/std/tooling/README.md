# `std/tooling`

This directory is the reading router for the `std` command/workflow/tooling
lane.

Keep the current recipe sources in
[`stdlib/std`](/Users/Shared/chroot/dev/nuislang/stdlib/std) for now; this
file exists to give the tooling surface one lane-shaped front door before we
do any higher-risk filesystem reshuffle.

Canonical companions:

* `std` refactor frontdoor:
  [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
* tooling/workflow contract:
  [docs/reference/std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)
* mainline layering rule:
  [docs/reference/std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)

## Current Lane Shape

Read the current tooling surface in this order:

```text
command runtime
-> subprocess runtime
-> workflow runtime
-> workflow recipe
-> cli/report/session companions
-> build/project/workflow frontdoors
```

Short rule:

* command/subprocess should stay the narrow request/result layer
* workflow should stay the first explicit gate/plan/report layer
* CLI/report/build/project recipes should reuse that gate shape rather than
  silently inventing a new one

## Source Router

### Runtime Edge

* [command_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime.ns)
* [subprocess_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime.ns)
* [workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime.ns)

### Narrow Recipe Layer

* [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
* [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
* [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
* [command_text_builder_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_text_builder_recipe.ns)

### Session / Report / Automation Companions

* [cli_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_session_recipe.ns)
* [cli_shell_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_shell_session_recipe.ns)
* [cli_report_session_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_report_session_recipe.ns)
* [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
* [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
* [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)

### Workflow Frontdoor Recipes

* [workflow_frontdoor_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_frontdoor_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)
* [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)

## Current Reading Rule

If you only want one pass:

1. start with [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
2. follow into [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
3. then [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
4. then one frontdoor recipe:
   [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
   or
   [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)

## Companion Validation Router

Use [examples/projects/tooling/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/README.md)
as the project-form companion set.

Shortest grouped route:

* [cli_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/workflow_runtime_demo)

Short rule:

* prefer these three frontdoor project routes first
* only then drop into the broader tooling companion router

## Current Refactor Meaning

For the current `0.20.*` line, this router means:

* the tooling lane is now treated as one owned `std` cluster
* the next source-level cleanup should happen here before the richer net lane
* repeated command/workflow skeletons are now a lane-level cleanup target, not
  just a local style quirk
