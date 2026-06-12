# `nuis` 0.18.0 Mainline Regression Matrix

This file is the compact regression gate for the `0.18.0` line.

It exists to answer one practical question:

`which checked-in test families now defend the actual 0.18 compile story?`

Use it together with:

* [nuis-0.18.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-control-flow-completion-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-control-flow-completion-plan.md)
* [nuis-0.18.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-compile-workflow.md)
* [nuis-0.18.0-loop-memory-read-contract-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-loop-memory-read-contract-sketch.md)
* [nuis-0.18.0-loop-memory-carry-blockers.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-loop-memory-carry-blockers.md)
* [nuis-0.18.0-host-boundary-address-abi.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-host-boundary-address-abi.md)

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
cargo test -q -p nuisc generic_method_bounds
cargo test -q -p nuisc --test state_compile
cargo test -q -p nuisc --test task_compile
cargo test -q -p nuisc --test memory_compile
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc --test network_compile
```

Equivalent checked-in script:

```bash
scripts/check-0.18-mainline.sh
```

Short rule:

`frontend control flow + generic-bound diagnostics + lowering loop families + state/task/memory/shader/network project gates = today’s smallest believable 0.18 mainline check`

For a heavier pre-release pass, use:

```bash
scripts/check-0.18-release.sh
```

That script is intentionally the current compiler-facing release gate.

The wider repo-level `cargo test -q -p nuis -p nuisc` smoke is still a broader
closure check and may surface stdlib/project issues outside this narrower
compiler gate.

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
* current source-level owned-vs-borrowed address contract closure
* current promise boundary that internal pointer semantics exist before host
  pointer ABI is opened up

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
* memory ordering truth below verifier-only ownership checks

Primary families:

* `core`:
  [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)
* `core`:
  [tests_async_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_runtime.rs)
* `core`:
  [tests_async_network_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_network_runtime.rs)

These families now also cover:

* `borrow_end -> free` ordering
* `store_at -> free` ordering
* borrowed `load_next` traversal ordering before `borrow_end` / `free`
* current loop-lowering boundary for memory/address backedge `while` bodies

Current truth:

* verifier now tracks conservative post-loop borrow/move state for `while`
* minimal lowering still rejects general memory/address iterative backedge loops
  outside the existing guarded/counted loop families
* guarded `while` lowering already accepts small read-only address payloads
  inside terminal guarded bodies
* chained/counting loop-node contracts now encode fixed read carry sources for
  loop-invariant `load_value(...)` and `load_at(buffer, index)` updates
* verifier now recognizes the same `load_value(...)` / `load_at(...)` shapes as
  fixed readable carry-source candidates, while still leaving loop-invariance
  gating to prepare/lowering
* broader memory-read carry expressions are still intentionally rejected,
  including loop-variant indices, structural traversal carries, and write-like
  backedge memory effects

Short rule:

`these are still important, but 0.18’s new honesty mostly comes from the project-backed state/task gates joining the older shader/network story`
