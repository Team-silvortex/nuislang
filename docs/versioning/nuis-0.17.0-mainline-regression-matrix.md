# `nuis` 0.17.0 Mainline Regression Matrix

This file is the compact regression gate for the `0.17.0` line.

It exists to answer one practical question:

`which test families actually defend the current compiler story?`

Use it together with:

* [nuis-0.17.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-goals.md)
* [nuis-0.17.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-compile-workflow.md)
* [nuis-0.17.0-lowering-capability-map.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-lowering-capability-map.md)
* [nuis-0.17.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-release-checklist.md)

## Reading Rule

Interpret this matrix conservatively:

* `smoke`:
  the smallest family that should catch a mainline break quickly
* `core`:
  the family that currently carries one whole layer of the compiler story
* `anchor`:
  the real-project proof that keeps the story honest outside unit-style probes

If a route is only defended by one narrow probe, it is not yet strongly
internalized.

## Minimal `0.17.0` Gate

When we want a fast but still honest mainline check, this is the current
minimum:

```bash
scripts/check-0.17-mainline.sh
```

Short rule:

`frontend generics + frontend higher-order + control-flow + async lowering + project compile anchors = today’s smallest believable integration gate`

For the current line, that minimum is now best read with one extra truth:

`helper-aware project integration probes are part of the honest gate, not optional side checks`

Expanded command list:

```bash
cargo test -q -p nuisc tests_generics
cargo test -q -p nuisc tests_higher_order
cargo test -q -p nuisc tests_generic_constraints
cargo test -q -p nuisc tests_control_flow
cargo test -q -p nuisc tests_async_runtime
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc multidomain_async
cargo test -q -p nuisc --test task_compile
cargo test -q -p nuisc --test network_compile
cargo test -q -p nuisc --test state_compile
```

## Matrix

### 1. Generic Rewrite Core

Role:

* expected-type propagation
* alias-aware constructor/field routes
* nested helper specialization
* async-facing generic specialization inputs

Primary family:

* `core`:
  [tests_generics.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generics.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_generics
```

If this fails, suspect:

* generic substitution inference
* expected-type propagation
* alias-aware struct/payload inference
* generic specialization recursion

### 2. Higher-Order + Lambda Closure

Role:

* lambda lifting
* higher-order specialization
* generic helper + callable crossover
* recursive async higher-order generic closure

Primary family:

* `core`:
  [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_higher_order
```

Important current anchor inside this family:

* `smoke`:
  [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs#L530)

If this fails, suspect:

* lambda specialization shape
* higher-order template expansion timing
* specialized-function postprocessing
* match-arm binding types not reaching generic rewrite

### 3. Generic Constraint Truth

Role:

* method-bound enforcement
* source-facing generic rejection quality
* trait-bound crossover with helper/lambda shapes

Primary family:

* `core`:
  [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_generic_constraints
```

If this fails, suspect:

* bound lookup drift
* validation ordering problems
* generic route support widening without matching diagnostics

### 4. Control-Flow Composition

Role:

* `if` / `match` branch-local typing truth
* specialization inside control-flow bodies
* control-flow-local reconstruction feeding later calls

Primary family:

* `core`:
  [tests_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_control_flow.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_control_flow
```

If this fails, suspect:

* branch-local environment assembly
* control-flow-local rewrite continuity
* rewritten-body return or expected-type drift

### 5. Async Lowering Bridge

Role:

* async recursive lowering
* task/runtime-facing lowering closure
* generic + higher-order routes surviving into executable async lowering

Primary family:

* `core`:
  [tests_async_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_runtime.rs)
* `core`:
  [tests_async_network_runtime.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_async_network_runtime.rs)

Recommended command:

```bash
cargo test -q -p nuisc tests_async_runtime
```

If this fails, suspect:

* async helper lowering
* recursive async call shaping
* schedule / lane emission drift
* frontend truth not surviving lowering

### 6. Task Project Anchors

Role:

* checked-in project compile truth for async/generic/recursive/task routes

Primary family:

* `anchor`:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test task_compile
```

This family currently protects:

* recursive async demos
* generic recursive async demos
* mutual recursion demos
* payload-alias higher-order async demos
* task/memory/session project closure with explicit timeout and staging helpers
* project-backed generic async specialization with compiled async call / await
  structure

If this fails, suspect:

* project pipeline glue
* multi-file specialization closure
* task-facing example drift relative to compiler truth

### 7. Project-Aware Bridge / Helper Integration

Role:

* helper-aware project lowering
* helper-mediated route payload inference
* helper-mediated project-link NIR validation
* cross-domain closure staying truthful outside entry-local direct calls

Primary families:

* `core`:
  [shader_nova_contracts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/shader_nova_contracts.rs)
* `core`:
  [multidomain_async.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/project/tests/multidomain_async.rs)

Recommended commands:

```bash
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc multidomain_async
```

This layer currently protects:

* `cpu helper -> shader/data` packet + bridge closure
* `cpu helper -> kernel/data` bridge payload closure
* `cpu helper -> network` profile-usage closure
* project analyses that must not silently fall back to isolated module lowering

### 8. State Project Generic/Higher-Order Anchors

Role:

* real-project generic callable forwarding truth
* higher-order helper expansion surviving multi-file project compilation
* project-facing `Fn2` / `Fn3` callable forwarding closure

Primary family:

* `anchor`:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test state_compile
```

Important current anchors inside this family:

* `anchor`:
  [generic_payload_alias_method_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_method_hof_demo)
* `anchor`:
  [generic_callable_forwarding_hof_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_callable_forwarding_hof_demo)

This layer currently protects:

* payload-alias generic higher-order project closure
* project-shaped `Fn2` / `Fn3` callable forwarding through nested
  `relay -> chain -> apply` helper specialization
* helper naming / reachability truth in project compilation, not only in
  unit-style frontend probes

### 9. State Project Lowering Anchors

Role:

* project-backed `flow_cond_chain` truth
* project-backed `post_flow_cond_chain` truth
* lowering-local control-flow shapes surviving real project compilation

Primary family:

* `anchor`:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test state_compile
```

Important current anchors inside this family:

* `anchor`:
  [flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo)
* `anchor`:
  [post_flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_while_demo)
* `anchor`:
  [post_flow_branching_continuing_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo)
* `anchor`:
  [tail_recursive_branching_cross_carry_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo)

This layer currently protects:

* structured `while` lowering into `loop_while_i64_flow_cond_chain`
* post-body `break` lowering into `loop_while_i64_post_flow_cond_chain`
* post-body `continue` lowering into `loop_while_i64_post_flow_cond_chain`
* tail-recursive branching carry lowering into `loop_while_i64_cond_chain`

If this fails, suspect:

* project-aware lowering drift
* helper visibility/signature registration drift
* payload inference only seeing entry-local direct statements
* bridge/profile validation only seeing shallow expression shapes

### 8. Network Project Anchors

Role:

* network/profile/transport/session compile closure
* generic helper stories surviving outside pure frontend probes
* request/header/session packet staging surviving as project-backed compile truth

Primary family:

* `anchor`:
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test network_compile
```

If this fails, suspect:

* alias-heavy project assembly
* helper-bridge generic routes
* network-facing examples drifting away from the shared compiler spine
* memory-staged session packet routes drifting from domain examples

### 9. State Project Anchors

Role:

* non-network project compile truth for generic/control-flow/state-heavy routes

Primary family:

* `anchor`:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

Recommended command:

```bash
cargo test -q -p nuisc --test state_compile
```

If this fails, suspect:

* state-oriented project examples no longer matching frontend claims
* generic/control-flow truth becoming too network-specific

## Escalation Order When Something Breaks

Use this drill:

1. run the smallest failing family from this matrix
2. classify the break:
   generic rewrite, higher-order closure, control-flow environment, async lowering, or project integration
3. compare against
   [nuis-0.17.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-compile-workflow.md)
   to see which stage owns the route
4. only widen the run to all of `cargo test -q -p nuisc -p nuis` after the local category is clear

## Recommended Release Gate Interpretation

For `0.17.0`, the release checklist should treat these categories as separate
truths:

* frontend generic/higher-order coherence
* control-flow composition
* lowering continuity
* helper-aware project integration closure
* task project closure
* network project closure
* state/non-network project closure

Short rule:

`if only one category stays green, the line is not integrated enough`
