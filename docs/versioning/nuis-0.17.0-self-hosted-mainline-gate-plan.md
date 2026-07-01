# `nuis` 0.17.0 Self-Hosted Mainline Gate Plan

This file turns one specific maturity question into an explicit `0.17.0`
target:

`when can the current mainline regression gate be written in nuis itself, rather than only in shell?`

The short current answer is:

`not yet`

That is not a philosophical answer.

It is a repository-state answer.

## Why This Matters

The repository now has:

* a current compile workflow anchor
* a current mainline regression matrix
* a shell entrypoint that actually runs that matrix

Those are useful.

But they are not yet the strongest possible maturity signal.

If `nuis` is meant to become a serious project/tooling language as well as a
compiled systems language, then a high-signal internal checkpoint should be:

* the repository can describe and run one of its own mainline validation gates
  in `nuis`

Short rule:

`if the mainline gate cannot eventually be expressed in nuis, the tooling/runtime surface is still not integrated enough`

## Current Truth

Today the repository already exposes host/tooling-oriented source surfaces:

* [command_runtime_recipe.ns](../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../stdlib/std/subprocess_runtime_recipe.ns)
* [command_shell_recipe.ns](../../stdlib/std/command_shell_recipe.ns)
* [automation_runtime_recipe.ns](../../stdlib/std/automation_runtime_recipe.ns)
* [cli_runtime_recipe.ns](../../stdlib/std/cli_runtime_recipe.ns)
* [report_runtime_recipe.ns](../../stdlib/std/report_runtime_recipe.ns)

There are also project-form companions:

* [command_runtime_demo](../../examples/projects/tooling/command_runtime_demo)
* [subprocess_runtime_demo](../../examples/projects/tooling/subprocess_runtime_demo)
* [cli_runtime_demo](../../examples/projects/tooling/cli_runtime_demo)

These are enough to show that the direction is real.

They are not enough to say that the current `0.17.0` mainline matrix should
already be implemented in `nuis`.

The older `command_shell_demo` and `automation_runtime_demo` routes have now
been retired from the checked-in examples tree; they no longer represent the
current recommended tooling path.

## Why The Current Surface Is Still Not Enough

The main issue is not “no host bridge exists”.

The main issue is that the current host/tooling bridge is still too low-level
and too handle-shaped.

Today most checked-in tooling surfaces still read like:

* explicit `extern "c"` host hooks
* `i64` program / argv / env handles
* `i64` process handles
* numeric status/join/signal summaries

That is fine as a narrow runtime contract layer.

It is not yet a good self-hosted workflow surface.

### Missing Capability 1: Structured Command Input

A self-hosted mainline gate should be able to describe:

* program
* argv list
* optional env overlay
* working directory
* reporting label

The current checked-in command/subprocess surfaces do not yet expose that as a
stable, ergonomic source-level structure.

### Missing Capability 2: Structured Command Output

A self-hosted gate needs more than “spawn happened”.

It needs stable access to at least:

* exit code
* success/failure classification
* optional stdout/stderr capture handles or report handles
* timeout / interruption classification

The current checked-in recipes mostly summarize raw host-return codes rather
than exposing a strong automation-facing result contract.

### Missing Capability 3: Workflow Composition

The current `0.17.0` matrix is a sequence of named checks.

A self-hosted gate needs source-level support for:

* ordered step execution
* per-step labeling
* stop-on-failure behavior
* aggregate report emission

The repository has pieces in `cli_runtime` / `report_runtime` / `automation`,
but not yet a clearly blessed “workflow runner” surface.

### Missing Capability 4: Toolchain Self-Invocation Story

The self-hosted target is not just “spawn any subprocess”.

It is specifically:

* invoke `cargo test ...`
* later invoke `nuis check`, `nuis test`, `nuis build`, `nuis release-check`
* keep those invocations understandable as one source-level flow

Today that story would still be too raw and too awkward to call a current
mainline route.

### Missing Capability 5: Mainline Claim

Even if something can be hacked together from today’s low-level surfaces, that
is not the same as saying:

`this is now the recommended self-hosted tooling path`

That stronger claim needs its own contract and regression coverage.

## Minimal Target For “Enough”

For this plan, “enough” does not mean building a giant workflow framework.

It means one narrow, honest target:

`the 0.17.0 mainline regression matrix can be expressed as one checked-in nuis project without dropping down into shell glue for every step`

That target suggests a minimal source/runtime surface like this:

### 1. Command Request

One narrow value-level shape for:

* executable/program
* argv
* env overlay or inherited env policy
* cwd
* optional timeout label/policy

### 2. Command Result

One narrow result-level shape for:

* launched or failed-to-launch
* exit code
* success bool or status enum
* optional stdout/stderr observation handles
* timeout/interruption classification

### 3. Workflow Step

One narrow step-level shape for:

* step name
* command request
* fail-fast policy

### 4. Workflow Report

One narrow aggregate shape for:

* step reports
* first failure
* overall pass/fail
* summary printing/report emission

## Suggested Build Order

This should be approached as a thin ladder, not as a giant leap.

### Stage 1. Lift Raw Command/Subprocess Handles Into Real `std` Values

Goal:

* stop making project/tooling code manually traffic almost entirely in `i64`
  handles

Needed outcome:

* one small `std`-level command/subprocess contract with named request/result
  structs

### Stage 2. Add One Report-Friendly Workflow Layer

Goal:

* make several command runs readable as one `nuis` source-level plan

Needed outcome:

* one thin workflow recipe or module above command/subprocess/report surfaces

### Stage 3. Build The First Self-Hosted Gate Project

Goal:

* re-express today’s shell script gate in `nuis`

Needed outcome:

* one checked-in project that runs the current matrix steps and reports pass/fail

Important rule:

* this project may still rely on host subprocess bridges
* it should not rely on an outer shell script to define the step plan itself

### Stage 4. Promote It Into Mainline Truth

Goal:

* make the self-hosted gate part of the real release story

Needed outcome:

* docs updated
* compile/project harness coverage added
* shell wrapper demoted to compatibility convenience

## What Should Count As Success

We should only say this plan landed when all of these are true:

* the mainline gate exists as a checked-in `nuis` project
* that project uses a stable `std` tooling surface rather than ad hoc raw host
  handles everywhere
* the project can clearly report which matrix step failed
* docs point to that project as the preferred self-hosted form
* the shell script is no longer the only real implementation

## What Should Not Count As Success

These should not be mistaken for completion:

* one demo that spawns a process and returns a sum of integer status handles
* one source file that can technically shell out but has no stable report shape
* a self-hosted gate that is harder to read than the shell script it replaces
* a claim that `nuis` can self-host tooling workflows without a documented
  contract behind it

## Immediate Next Step

The best next move is not “rewrite the whole gate right now”.

The best next move is:

* define the smallest workflow-friendly `std tooling` contract that can power
  one self-hosted gate project

Current contract anchor:

* [std-tooling-workflow-contract.md](../../docs/reference/std-tooling-workflow-contract.md)

Short rule:

`first make command/subprocess/report surfaces readable as source values, then make the mainline gate one honest nuis project`
