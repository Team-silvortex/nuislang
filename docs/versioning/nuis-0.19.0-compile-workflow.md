# `nuis` Compile Workflow For `0.19.0`

This file is the current compiler-facing workflow anchor for the `0.19.0`
line.

The `0.18.0` workflow file explained the move from partial control-flow truth
to a more believable project-backed mainline.

This `0.19.0` file narrows the story to one sharper question:

`what is the honest current compile route once the mainline already exists and the next job is to internalize it?`

Use it when the question is not only “what compiles?”, but:

* which ordered workflow we should now teach first
* which source-style rules belong at the source layer
* which docs explain lowered truth instead
* which checked-in gates still defend the story today

Use it together with:

* [nuis-0.19.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-goals.md)
* [nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
* [nuis-0.19.0-workflow-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-workflow-capability-matrix.md)
* [nuis-0.19.0-generic-constraint-validator.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-generic-constraint-validator.md)
* [nuis-0.19.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-address-pointer-mainline.md)
* [nuis-0.18.0-loop-memory-read-contract-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-loop-memory-read-contract-sketch.md)

## Core Rule

For the current `0.19.0` line, the mainline should now be read as:

```text
source-facing route
  -> status / help
  -> workflow
  -> project-doctor / project-status / scheduler-view
  -> check / test / build / release-check

compiler-facing route
  -> surface syntax
  -> lambda lifting
  -> helper / alias assembly
  -> higher-order expansion
  -> generic validation + specialization
  -> frontend lowering to NIR
  -> project-aware lowering continuity
  -> YIR verifier / loop-family / async / contract steps
```

Short rule:

`0.19.0` compile truth is now as much about keeping the route legible as it is about keeping the route green`

## Current CLI Frontdoor

Today the honest outermost reading order is:

```text
nuis status / nuis help
  -> nuis workflow
  -> nuis project-doctor / nuis project-status / nuis scheduler-view
  -> nuis check
  -> nuis test
  -> nuis build
  -> nuis release-check
```

The practical reason is simple:

* `status` and `help` are now the orientation pair
* `workflow` classifies single-file vs project-facing routes
* `project-doctor`, `project-status`, and `scheduler-view` expose grouped
  preflight detail instead of disconnected command-local stories
* `check/test/build/release-check` remain the action spine

## Current Source-Syntax Boundary

At the source level, the current address story should now be taught with:

* `ptr.value`
* `ptr.next`
* `buffer.len`
* `buffer[index]`

That is current `.ns` source truth.

Builtin names such as:

* `load_value(...)`
* `load_next(...)`
* `load_at(...)`
* `store_at(...)`

remain the lowered implementation vocabulary used by:

* verifier-facing docs
* NIR discussion
* YIR/CPU language references
* carry/lowering-contract sketches

Short rule:

`0.19.0` source workflow starts from the surface syntax contract, not from the lowered helper names`

## Canonical Frontend Order

The real frontend entry remains:

* [lower_project_ast_to_nir](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs#L157)

The checked-in order still matters:

1. parse source
2. expand module lambdas
3. assemble visible helpers and aliases
4. expand higher-order functions
5. normalize effectful `match` scrutinees
6. validate annotations / exports / bridges
7. build signatures with generic-template separation
8. build impl lookup and validate constraints
9. assemble consts
10. lower functions and impls to NIR
11. validate declared NIR types
12. keep project-aware lowering context alive into YIR

Short rule:

`the current mainline is one ordered compiler path, not one frontend path plus many loosely related project exceptions`

## Current Project Compile Spine

Today the believable project-facing compile spine should be read as:

```text
parse project modules
  -> validate module/unit/link/abi declarations
  -> lower entry with visible local helpers
  -> preserve helper-aware context during control-flow-sensitive lowering
  -> validate project links/contracts against NIR
  -> lower to YIR
  -> materialize loop-family / async / verifier / contract truth
  -> validate project links/contracts against YIR
```

## Current Practical Gates

For the current line, the practical regression route is:

* quick current compiler-facing mainline gate:
  `bash scripts/check-0.19-mainline.sh`
* heavier current compiler-facing gate:
  `bash scripts/check-0.19-release.sh`
* broader repo-level smoke:
  `cargo test -q -p nuis -p nuisc`

## Current Project-Backed Anchors

The current believable mainline should still be taught through:

* control-flow/state proof:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)
* async/task proof:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)
  current staged thread/lock project anchor:
  [task_thread_mutex_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_thread_mutex_demo)
  current frontdoor sample:
  `nuis project-doctor examples/projects/task/task_thread_mutex_demo`
  `nuis check examples/projects/task/task_thread_mutex_demo`
  `nuis test examples/projects/task/task_thread_mutex_demo`
  current honesty rule:
  project test is explicit, frontdoor-visible, and now executes successfully
  through the staged AOT thread/lock path
* memory/address proof:
  [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* shader/helper-mediated proof:
  [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)
* network/http/session proof:
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

## Rule Of Thumb

If `0.18.*` was about getting many routes to line up, `0.19.*` is about making
that alignment easy to follow without guesswork.
