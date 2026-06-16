# Alpha Mainline Boundary Index

This file is the shortest current index for the mainline boundaries that matter
most before `alpha-0.0.1`.

It is not a full architecture manual.

It is the reading page for:

`if I need to understand what nuis already treats as real before alpha, where should I look first?`

## Short Rule

Before `alpha-0.0.1`, the mainline should be read through a small set of
explicit boundaries:

* source compile truth
* control-flow lowering truth
* ownership / `GLM` truth
* staged thread/lock truth
* compile/workflow truth

If one layer says “yes” and another still says “no”, treat that as a boundary
fact, not as a contradiction to blur away.

## 1. Compile-Workflow Boundary

Use this when the question is:

`what is the current honest path from source/project to compiled truth?`

Read:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [nuis-0.20.x-to-alpha-bootstrap-roadmap.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.x-to-alpha-bootstrap-roadmap.md)
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
* [nuis-0.20.0-frontend-cli-boundaries.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-frontend-cli-boundaries.md)

Short rule:

* frontend truth and compile-closure truth are related but not interchangeable
* project/source compile remains the honest default route

## 2. Memory / Ownership Boundary

Use this when the question is:

`what does the verifier currently protect for ref/borrow/move/free and branch/loop ownership?`

Read:

* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

Short rule:

* owner authority must stay explicit
* borrow state merges conservatively across branches and loops
* control flow does not silently restore ownership

## 3. Task Observation / `GLM` Boundary

Use this when the question is:

`what does Task/TaskResult mean today in graph-lifetime terms?`

Read:

* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)

Short rule:

* `join(...)` / `join_result(...)` consume task handles
* `TaskResult<T>` is the current reusable observation root
* `task_*` helpers are observer/extractor operations with lifecycle constraints

## 4. Control-Flow Boundary

Use this when the question is:

`which if/match/while/recursion shapes already lower honestly, and which still do not?`

Read:

* [control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)

Short rule:

* branch-selected values plus shared suffix are the strongest current family
* observer-safe branch-local reads are real
* arbitrary branch-local consuming runtime mini-programs are still intentionally
  blocked

## 5. Thread/Lock Boundary

Use this when the question is:

`how real are Thread/Mutex/MutexGuard today, and where is the staged boundary?`

Read:

* [cpu-thread-lock-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-boundary.md)
* [cpu-thread-lock-staging-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-staging-sketch.md)

Short rule:

* thread/lock families are already real enough to have positive compile
  anchors, negative boundary anchors, and verifier-backed ownership truth
* they are still not a finished concurrent runtime model

## 6. Nustar Capability Boundary

Use this when the question is:

`what should already be split into stable registered capability contracts, and what is still just bootstrap/compiler-core knowledge?`

Read:

* [nustar-capability-split-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-capability-split-boundary.md)
* [annotation-intrinsic-stdlib-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/annotation-intrinsic-stdlib-sketch.md)
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)

Short rule:

* compiler-owned surface spelling can stay replaceable
* stable capability/package truth should increasingly live in `nustar`
* bootstrap shims are acceptable only when the registered contract is already
  the clearer long-lived source of truth

## 7. Example Entry Boundary

Use this when the question is:

`which checked-in examples are the shortest front doors for the current line?`

Read:

* [examples/ns/memory/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/README.md)
* [examples/invalid/ns/memory/README.md](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/README.md)
* [examples/projects/task/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task/README.md)

Short rule:

* use single-file `.ns` anchors first
* use invalid anchors to understand the real boundary
* use project demos when you need the wider facade/workflow shape

## Practical Reading Order Before `alpha-0.0.1`

If you need the shortest full route, use this order:

1. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
2. [alpha-mainline-boundary-index.md](/Users/Shared/chroot/dev/nuislang/docs/reference/alpha-mainline-boundary-index.md)
3. the one specific boundary doc that matches the feature you are touching
4. the nearest positive and negative example anchors
5. the matching regression test file

## Why This Exists

Before `alpha`, the repository should stop making people reconstruct the
mainline from memory.

This index exists so the current line can be read as:

* one toolchain
* a small number of explicit boundaries
* a small number of checked-in anchors that prove those boundaries

That is the level of compression the repository should increasingly prefer on
the way into `alpha-0.0.1`.
