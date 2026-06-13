# `std` Tooling Workflow Contract

This file defines the workflow-oriented `std` contract that makes it reasonable
to build a checked-in self-hosted mainline gate in `nuis`.

It is not the final tooling architecture.

It is the minimum contract we should be able to explain and implement without
falling back to shell glue as the only real workflow surface.

It now also serves as the shortest reference for the current checked-in
tooling/workflow sample chain in `std`.

## Why This Contract Exists

The repository already has checked-in command, subprocess, CLI, report, and
automation-facing source lanes.

Current narrow sources include:

* [command_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/subprocess_runtime_recipe.ns)
* [command_shell_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_shell_recipe.ns)
* [cli_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_runtime_recipe.ns)
* [report_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/report_runtime_recipe.ns)
* [automation_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/automation_runtime_recipe.ns)
* [workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime.ns)
* [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)

Those lanes are real enough to prove direction.

They are now strong enough to say:

`nuis can already express a readable checked-in workflow gate shape inside std`

They are not yet strong enough to say:

`the self-hosted mainline gate is complete and no longer needs shell-side support`

This contract exists to define the gap precisely.

## Current Contract Status

The old problem was:

`the command/tooling lane was still too raw for workflow authorship`

That statement is no longer fully true.

The current truth is narrower:

* command/subprocess/workflow now share a context-aware source shape
* checked-in workflow recipes now model launched/executed/blocked state
* checked-in build-oriented recipes now carry project/artifact/report vocabulary
* the remaining gap is shared reusable authoring shape, not basic expressivity

Short rule:

`the current std tooling lane is readable enough to prove direction, but still too copy-shaped to call finished`

## Current Problem Statement

Today the command/tooling lane is no longer too raw to express workflow
authorship.

Today the bigger problem is repeated local skeletons across the checked-in
tooling samples.

The dominant current source shape is:

* explicit host externs
* near-identical command context/request/result structs
* near-identical step/report summarizers
* per-file naming differences around one shared gate pattern
* integer-oriented handles and summaries at the host boundary

That is acceptable for a bridge/facade phase.

It is already good enough for writing:

* ordered tooling gates
* build-oriented pipelines
* project-facing report flows

It is not yet the final contract for writing:

* ordered test gates
* release gates
* project health pipelines
* self-hosted tooling orchestration

## Minimal Contract Goal

The smallest believable target is now:

`one nuis project can run a sequence of named toolchain commands, classify pass/fail, stop or continue by policy, and emit one summary report`

That target now has checked-in proof in `std`.

The current sample ladder is:

* base workflow proof:
  [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
* smallest integrated toolchain-shaped proof:
  [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
* build-oriented proof:
  [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
* project/report-oriented proof:
  [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)

That target still implies four source-level value families.

## 1. Command Request

The command layer should stop at a typed request value before it reaches the
raw host bridge.

Current checked-in shape:

```text
CommandRequest
  program
  argv
  cwd
  timeout
  inherit flags
```

Minimum semantic meaning:

* `program`
  the executable or front-door tool to invoke
* `argv`
  the ordered argument vector
* `cwd`
  optional working-directory selection
* `timeout`
  no-timeout vs finite timeout
* `inherit flags`
  current narrow inheritance control for cwd/env-style routing

The important contract is not the exact field names.

The important contract is:

* workflow code should describe a command as source values
* host-specific argument packing should happen below that layer

Current checked-in anchors:

* [command_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/command_runtime.ns)
* [workflow_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime.ns)

## 2. Command Result

The subprocess layer should surface a typed result value rather than only raw
join/status integers.

Current checked-in shape:

```text
CommandResult
  launched
  status
  wait_code
  wait_exit
  success
  cwd
  timeout
  inherit flags
```

Minimum semantic meaning:

* `launched`
  launched vs failed-to-launch
* `status` / `wait_code` / `wait_exit`
  current narrow host-side execution summaries
* `success`
  one workflow-friendly boolean or equivalent narrow status
* `cwd` / `timeout`
  enough execution context truth to support report layers and gate reasoning
* `inherit flags`
  enough execution-shape truth to distinguish inherited vs explicit routing

Short rule:

`workflow code should not have to reverse-engineer success/failure from several unrelated integer fields`

## 3. Workflow Step

One workflow command is not enough.

A self-hosted gate needs a step layer.

Current checked-in shape:

```text
WorkflowStep
  name
  request
  fail_fast
```

Minimum semantic meaning:

* `name`
  stable human-readable step identity
* `request`
  the command to run
* `fail_fast`
  current narrow gate policy, with room for richer policies later

This is now already the first layer where a release gate or regression matrix
starts becoming source-readable.

## 4. Workflow Report

The workflow layer should end in one aggregate report rather than scattered
prints and ad hoc integer accumulation.

Current checked-in shape:

```text
WorkflowReport
  step reports
  first_failure
  overall_success
  executed_steps
```

Minimum semantic meaning:

* `step_reports`
  one result per step
* `first_failure`
  direct pointer to the first failed step, if any
* `overall_success`
  source-level workflow answer
* `executed_steps`
  one narrow gate-progress summary that is easy to consume from CLI/report
  layers

Short rule:

`a workflow contract is not complete until it can describe failure as clearly as it describes execution`

Current checked-in proof:

* [workflow_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)

## Layering Rule

This contract should fit the existing `std` layering philosophy:

```text
raw host command/subprocess facade
-> narrow typed command/subprocess runtime layer
-> workflow recipe layer
-> CLI/report/session or self-hosted gate project
```

That means:

* do not jump directly from raw host externs to one giant workflow framework
* first add a readable typed command/subprocess layer
* only then add the workflow composition layer

The current checked-in `std` tooling chain now reads like:

```text
command/subprocess runtime
-> workflow runtime
-> workflow runtime recipe
-> cli_workflow_automation recipe
-> cli_build_pipeline recipe
-> cli_project_build_report recipe
```

Short rule:

`the next step should improve shared authoring shape inside this ladder, not replace the ladder`

## Non-Goals For This Stage

This contract still does not require:

* full shell parsing semantics
* a general job scheduler
* distributed execution
* advanced parallel process orchestration
* a permanent stable package architecture for all tooling

It only requires enough structure to make one honest self-hosted validation
pipeline readable and maintainable.

The repository now already has enough structure to make several honest
validation/build/report samples readable and maintainable.

## Immediate Repository Implications

If we continue following this contract, the next concrete repository work should
look like:

1. preserve the current command/subprocess/workflow request/result truth as the stable narrow shape
2. reduce repeated local skeletons across the CLI/build/project samples
3. build one checked-in `nuis` project that re-expresses the current mainline regression matrix using this shape
4. keep shell-side helpers as convenience wrappers until the self-hosted route is clearly stronger

## Current Shared Shape

Across the current checked-in tooling samples, the repeated authoring pattern is:

```text
session capture
-> automation/artifact capture
-> build or workflow plan/manifest
-> four-step fail-fast command gate
-> report/diagnostic emission
-> one integer summary sink for compile proof
```

The three current reference samples occupy different points on that ladder:

* [workflow_frontdoor_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/workflow_frontdoor_runtime_recipe.ns)
  narrow grouped front-door surface reference
* [cli_workflow_automation_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_workflow_automation_recipe.ns)
  smallest integrated sample
* [cli_build_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_build_pipeline_recipe.ns)
  build-oriented stage naming plus shared front-door surface
* [cli_project_build_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_project_build_report_recipe.ns)
  project/artifact/manifest/report naming plus shared front-door surface
* [cli_compile_workflow_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cli_compile_workflow_recipe.ns)
  front-door compile workflow naming with nested project build plan/report and
  recommendation-style workflow hints plus source-kind workflow profiles and
  debug-workflow mirror fields grouped as one front-door surface

Short rule:

`new std tooling recipes should vary vocabulary by domain, but not silently invent a new gate skeleton`

## Success Signal

We should say this contract is real only when all of these become true:

* the current `std` command/tooling lane no longer reads mostly like raw handle arithmetic
* the checked-in workflow/build/project samples remain recognizably one family
* one checked-in workflow project becomes easier to read than the shell equivalent
* docs can point to that project as the preferred self-hosted gate route
* the repository’s mainline story gets stronger, not merely more elaborate
