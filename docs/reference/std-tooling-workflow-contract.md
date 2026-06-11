# `std` Tooling Workflow Contract

This file defines the smallest workflow-oriented `std` contract that would make
it reasonable to build a checked-in self-hosted mainline gate in `nuis`.

It is not the final tooling architecture.

It is the minimum contract we should be able to explain and implement without
falling back to shell glue as the only real workflow surface.

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

Those lanes are real enough to prove direction.

They are not yet enough to say:

`nuis can already express its own mainline workflow gate cleanly`

This contract exists to define the gap precisely.

## Current Problem Statement

Today the command/tooling lane is still too raw for workflow authorship.

The dominant current source shape is:

* explicit host externs
* `i64` program handles
* `i64` argv/env handles
* `i64` process handles
* integer status/join summaries

That is acceptable for a bridge/facade phase.

It is not yet a good contract for writing:

* ordered test gates
* release gates
* project health pipelines
* self-hosted tooling orchestration

## Minimal Contract Goal

The smallest believable target is:

`one nuis project can run a sequence of named toolchain commands, classify pass/fail, stop or continue by policy, and emit one summary report`

That target implies four source-level value families.

## 1. Command Request

The command layer should stop at a typed request value before it reaches the
raw host bridge.

Minimum shape:

```text
CommandRequest
  program
  argv
  env_policy
  cwd
  timeout_policy
```

Minimum semantic meaning:

* `program`
  the executable or front-door tool to invoke
* `argv`
  the ordered argument vector
* `env_policy`
  inherited environment, explicit overlay, or explicit empty policy
* `cwd`
  optional working-directory selection
* `timeout_policy`
  no-timeout, finite-timeout, or future richer timing policy

The important contract is not the exact field names.

The important contract is:

* workflow code should describe a command as source values
* host-specific argument packing should happen below that layer

## 2. Command Result

The subprocess layer should surface a typed result value rather than only raw
join/status integers.

Minimum shape:

```text
CommandResult
  launch_status
  exit_status
  success
  stdout_view
  stderr_view
  timing
```

Minimum semantic meaning:

* `launch_status`
  launched vs failed-to-launch
* `exit_status`
  exited with code, signaled, timed out, or interrupted
* `success`
  one workflow-friendly boolean or equivalent narrow status
* `stdout_view` / `stderr_view`
  optional output/report handles or output summaries
* `timing`
  enough timing truth to support reporting and later policy growth

Short rule:

`workflow code should not have to reverse-engineer success/failure from several unrelated integer fields`

## 3. Workflow Step

One workflow command is not enough.

A self-hosted gate needs a step layer.

Minimum shape:

```text
WorkflowStep
  name
  request
  fail_policy
```

Minimum semantic meaning:

* `name`
  stable human-readable step identity
* `request`
  the command to run
* `fail_policy`
  fail-fast, continue-on-failure, or future richer gating policy

This should be the first layer where a release gate or regression matrix
becomes source-readable.

## 4. Workflow Report

The workflow layer should end in one aggregate report rather than scattered
prints and ad hoc integer accumulation.

Minimum shape:

```text
WorkflowReport
  step_reports
  first_failure
  overall_success
  summary
```

Minimum semantic meaning:

* `step_reports`
  one result per step
* `first_failure`
  direct pointer to the first failed step, if any
* `overall_success`
  source-level workflow answer
* `summary`
  one report-friendly view for CLI/reporting output

Short rule:

`a workflow contract is not complete until it can describe failure as clearly as it describes execution`

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

## Non-Goals For This Stage

This contract does not require:

* full shell parsing semantics
* a general job scheduler
* distributed execution
* advanced parallel process orchestration
* a permanent stable package architecture for all tooling

It only requires enough structure to make one honest self-hosted validation
pipeline readable and maintainable.

## Immediate Repository Implications

If we follow this contract, the next concrete repository work should look like:

1. add typed source-level command/subprocess request/result shapes in `std`
2. add one narrow workflow recipe above them
3. build one checked-in `nuis` project that re-expresses the current mainline
   regression matrix
4. keep the shell script as a convenience wrapper until the self-hosted route
   is clearly stronger

## Success Signal

We should say this contract is real only when all of these become true:

* the current `std` command/tooling lane no longer reads mostly like raw handle arithmetic
* one checked-in workflow project is easier to read than the shell equivalent
* docs can point to that project as the preferred self-hosted gate route
* the repository’s mainline story gets stronger, not merely more elaborate
