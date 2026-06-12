# `nuis` 0.18.0 Mainline Regression Matrix

This file is the compact regression gate for the `0.18.0` line.

It exists to answer one practical question:

`which checked-in test families now defend the actual 0.18 compile story?`

Use it together with:

* [nuis-0.18.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-control-flow-completion-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-control-flow-completion-plan.md)
* [nuis-0.18.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-compile-workflow.md)

## Reading Rule

Interpret this matrix conservatively:

* `smoke`:
  the smallest family that should catch a current mainline break quickly
* `core`:
  the family that currently carries one whole layer of the compiler story
* `anchor`:
  the real-project proof that keeps the story honest outside unit-style probes

Short rule:

`0.18.0` maturity claims should increasingly be backed by project anchors, not only frontend/lowering-local greens`

## Minimal `0.18.0` Gate

When we want a fast but still honest mainline check, the current minimum should
be read as:

```bash
cargo test -q -p nuisc tests_control_flow
cargo test -q -p nuisc tests_loop_flow
cargo test -q -p nuisc tests_loop_post_flow
cargo test -q -p nuisc --test state_compile
cargo test -q -p nuisc --test task_compile
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc --test network_compile
```

Short rule:

`frontend control flow + lowering loop families + state/task/shader/network project gates = today’s smallest believable 0.18 mainline check`

## Matrix

### 1. Frontend Control-Flow Composition

Role:

* `if` / `match` expression-position truth
* branch-local environment continuity
* control-flow-aware generic/helper rewrites

Primary family:

* `core`:
  [tests_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_control_flow.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_control_flow
```

### 2. Lowering Loop-Family Core

Role:

* counted/carry/basic loop lowering
* flow/post-flow lowering
* branch/carry loop-family closure

Primary families:

* `core`:
  [tests_loop_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_loop_flow.rs)
* `core`:
  [tests_loop_post_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_loop_post_flow.rs)

Recommended commands:

```bash
cargo test -q -p nuisc tests_loop_flow
cargo test -q -p nuisc tests_loop_post_flow
```

### 3. State Project Control-Flow Anchors

Role:

* project-backed `while` family truth
* cond/flow/post-flow loop-family survival in real examples
* carry and branch-local state updates surviving project compilation

Primary family:

* `anchor`:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

This family now protects:

* basic loop routes:
  counted, inequality
* chain routes:
  accumulating, chained
* cond routes:
  branching, match, bool-match, lambda-match, double-branching
* flow routes:
  flow-continue, equality-branching, lambda-match-flow, lambda-match-or-flow
* post-flow routes:
  bounded, equality, post-flow break, post-flow continue, post-flow branch,
  post-flow branching continue
* guarded/control routes:
  match-guarded, carried-breaking

Recommended command:

```bash
cargo test -q -p nuisc --test state_compile
```

### 4. Task Project Async/Control-Flow Anchors

Role:

* task result selection truth
* lifecycle/timeout/fallback branch truth
* batch/windowed batch summary truth
* recursive/generic async project closure

Primary family:

* `anchor`:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

This family now protects:

* lifecycle/result-family/policy/fallback routes
* batch/result-batch/windowed/result-windowed routes
* recursive, generic, mutual-recursive, and memory/session task routes

Recommended command:

```bash
cargo test -q -p nuisc --test task_compile
```

### 5. Shader Helper-Mediated Project Closure

Role:

* packet/bridge/result truth
* helper-mediated shader project lowering
* async shader summary routes

Primary family:

* `anchor`:
  [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)

Recommended command:

```bash
cargo test -q -p nuisc shader_nova_contracts
```

### 6. Memory/Address Pointer Closure

Role:

* current `ref` address-family compile truth
* structural pointer + buffer address lowering truth
* explicit borrow-closure-before-owner-write truth

Primary family:

* `anchor`:
  [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)

This family now protects:

* `ref Node` structural allocation / next-link / payload-load routes
* `ref Buffer` indexed address staging routes
* ref-carrying struct field truth
* direct source-level address examples used by the current front door

Recommended command:

```bash
cargo test -q -p nuisc --test memory_compile
```

### 7. Network/HTTP/Session Project Closure

Role:

* helper-heavy project-aware network lowering
* HTTP/session/request project closure
* HTTP-ish packet/session bridge truth

Primary family:

* `anchor`:
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test network_compile
```

### 8. Wider Integration Complements

Role:

* helper-aware multidomain project closure beyond the main control-flow gate
* async network lowering closure beyond the state/task project anchor layer

Primary families:

* `core`:
  [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)
* `core`:
  [tests_async_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_runtime.rs)
* `core`:
  [tests_async_network_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_network_runtime.rs)

Short rule:

`these are still important, but 0.18’s new honesty mostly comes from the project-backed state/task gates joining the older shader/network story`
