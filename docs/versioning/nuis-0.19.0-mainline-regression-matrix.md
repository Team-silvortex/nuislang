# `nuis` 0.19.0 Mainline Regression Matrix

This file is the compact regression gate for the `0.19.0` line.

It exists to answer one practical question:

`which checked-in test families now defend the current 0.19 mainline, even if some gate names still carry 0.18 history?`

Use it together with:

* [nuis-0.19.0-mainline-goals.md](nuis-0.19.0-mainline-goals.md)
* [nuis-0.19.0-compile-workflow.md](nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-project-capability-matrix.md](nuis-0.19.0-project-capability-matrix.md)
* [nuis-0.19.0-release-checklist.md](nuis-0.19.0-release-checklist.md)

## Reading Rule

Interpret this matrix conservatively:

* `smoke`:
  the smallest family that should catch a current mainline break quickly
* `core`:
  the family that currently carries one whole compiler layer
* `anchor`:
  the real-project proof that keeps the story honest outside unit-style probes

Short rule:

`0.19.0` currentness claims should be backed by named checked-in gates, not by version labels alone`

## Minimal `0.19.0` Gate

The current practical minimum is:

```bash
bash scripts/check-0.19-mainline.sh
```

That gate currently covers:

```bash
cargo test -q -p nuisc tests_control_flow
cargo test -q -p nuisc tests_loop_flow
cargo test -q -p nuisc tests_loop_post_flow
cargo test -q -p nuisc generic_method_bounds
cargo test -q -p nuisc --test state_compile
cargo test -q -p nuisc --test task_compile
cargo test -q -p nuisc --test memory_compile
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc --test network_compile
```

For the heavier compiler-facing gate, use:

```bash
bash scripts/check-0.19-release.sh
```

## Matrix

### 1. Frontend Control-Flow Composition

Primary family:

* `core`:
  [tests_control_flow.rs](../../tools/nuisc/src/frontend/tests_control_flow.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_control_flow
```

### 1a. Lowering Branch-Local Runtime Boundary

Primary family:

* `core`:
  [tests_branch_helpers.rs](../../tools/nuisc/src/lowering/tests_branch_helpers.rs)

Recommended command:

```bash
cargo test -q -p nuisc lowering::tests_branch_helpers
```

Current boundary defended here:

* branch-local recursive `if` / lowered `match` value paths
* shared-suffix return-chain and shared-binding lowering
* branch-local runtime observer acceptance:
  `task_completed`, `task_timed_out`, `task_cancelled`, `task_value`,
  `mutex_value`
* branch-local consuming runtime rejection:
  `join_result`, `thread_join_result`, `mutex_lock`, `mutex_unlock`,
  `spawn` / `join` / `timeout`-family shapes

### 2. Lowering Loop-Family Core

Primary families:

* `core`:
  [tests_loop_flow.rs](../../tools/nuisc/src/lowering/tests_loop_flow.rs)
* `core`:
  [tests_loop_post_flow.rs](../../tools/nuisc/src/lowering/tests_loop_post_flow.rs)

Recommended commands:

```bash
cargo test -q -p nuisc tests_loop_flow
cargo test -q -p nuisc tests_loop_post_flow
```

### 3. Generic-Bound Diagnostic Truth

Primary family:

* `core`:
  [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)

Recommended command:

```bash
cargo test -q -p nuisc generic_method_bounds
```

### 4. State Project Control-Flow Anchors

Primary family:

* `anchor`:
  [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test state_compile
```

### 5. Task Project Async/Control-Flow Anchors

Primary family:

* `anchor`:
  [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)
* current staged thread/lock project sample:
  [task_thread_mutex_demo](../../examples/projects/task/task_thread_mutex_demo)

Recommended command:

```bash
cargo test -q -p nuisc --test task_compile
```

Useful frontdoor companion:

```bash
nuis project-doctor examples/projects/task/task_thread_mutex_demo
nuis check examples/projects/task/task_thread_mutex_demo
nuis test examples/projects/task/task_thread_mutex_demo
```

Current note:

`task_thread_mutex_demo` now carries an explicit project smoke test and that
smoke executes successfully through the staged AOT thread/lock path.

### 6. Memory/Address Pointer Closure

Primary family:

* `anchor`:
  [memory_compile.rs](../../tools/nuisc/tests/memory_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test memory_compile
```

### 7. Shader Helper-Mediated Project Closure

Primary family:

* `anchor`:
  [shader_nova_contracts.rs](../../tools/nuisc/src/project/tests/shader_nova_contracts.rs)

Recommended command:

```bash
cargo test -q -p nuisc shader_nova_contracts
```

### 8. Network/HTTP/Session Project Closure

Primary family:

* `anchor`:
  [network_compile.rs](../../tools/nuisc/tests/network_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test network_compile
```

### 9. Wider Integration Complements

Primary families:

* `core`:
  [multidomain_async.rs](../../tools/nuisc/src/project/tests/multidomain_async.rs)
* `core`:
  [tests_async_runtime.rs](../../tools/nuisc/src/lowering/tests_async_runtime.rs)
* `core`:
  [tests_async_network_runtime.rs](../../tools/nuisc/src/lowering/tests_async_network_runtime.rs)

Recommended commands:

```bash
cargo test -q -p nuisc multidomain_async
cargo test -q -p nuisc tests_async_runtime
cargo test -q -p nuisc tests_async_network_runtime
```

## Rule Of Thumb

The current line should be judged by the tests that defend the real workflow,
not by prose alone.
