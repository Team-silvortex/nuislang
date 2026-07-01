# `nuis` 0.17.0 Lowering Capability Map

This file is the compact lowering-facing capability map for the `0.17.0`
line.

It is narrower than the broad compile-workflow file and more concrete than the
snapshot/goals docs.

Use it when the question is:

`which lowering routes are already test-backed enough to teach as real compiler behavior, and where are the current edges?`

Read it together with:

* [nuis-0.17.0-compile-workflow.md](nuis-0.17.0-compile-workflow.md)
* [nuis-0.17.0-mainline-regression-matrix.md](nuis-0.17.0-mainline-regression-matrix.md)
* [../reference/std-net-layering-contract.md](../../docs/reference/std-net-layering-contract.md)
* [../reference/network-domain-contract.md](../../docs/reference/network-domain-contract.md)

## Reading Rule

Interpret this map conservatively:

* `mature path` means the lowering shape has multiple checked-in tests and is
  now part of the honest compiler story
* `bridge path` means the route is real and useful, but still best taught as a
  boundary or integration layer
* `edge` means the current repository has a clear limitation that should not be
  overstated away

Short rule:

`0.17.0` lowering truth should be described in terms of named, test-backed paths, not feature slogans`

## Capability Layers

### 1. Core Async Loop Lowering

Status:

* `mature path`

What is solid now:

* async counted/chained/flow/post-flow loop lowering
* recursive boolean control lowering for `and` / `or`
* conditional carry updates
* self-tail async recursion rewriting into loop forms

Primary anchors:

* [tests_async_runtime.rs](../../tools/nuisc/src/lowering/tests_async_runtime.rs)
* [tail_recursion.rs](../../tools/nuisc/src/lowering/tail_recursion.rs)
* [loop_preparation.rs](../../tools/nuisc/src/lowering/loop_preparation.rs)
* [loop_flow_nodes.rs](../../tools/nuisc/src/lowering/loop_flow_nodes.rs)
* [task_compile.rs](../../tools/nuisc/tests/task_compile.rs)
* [task_generic_recursive_async_demo](../../examples/projects/task/task_generic_recursive_async_demo)
* [task_memory_session_policy_demo](../../examples/projects/task/task_memory_session_policy_demo)
* [state_compile.rs](../../tools/nuisc/tests/state_compile.rs)
* [flow_branching_while_demo](../../examples/projects/state/flow_branching_while_demo)
* [post_flow_branching_while_demo](../../examples/projects/state/post_flow_branching_while_demo)
* [post_flow_branching_continuing_while_demo](../../examples/projects/state/post_flow_branching_continuing_while_demo)

Recommended command:

```bash
cargo test -q -p nuisc lowering::tests_async_runtime
```

Project-facing lowering anchors now also cover:

* generic async specialization surviving the project pipeline
* explicit `spawn / await` async structure surviving the project pipeline
* task/memory/session staging routes surviving the project pipeline
* ordinary `while` loops that lower into `flow_cond_chain`
* post-body `break` control that lowers into `post_flow_cond_chain`
* post-body `continue` control that lowers into `post_flow_cond_chain`

Named project proofs:

* `lowers_flow_branching_while_state_project_with_flow_cond_loop_shape`
* `lowers_post_flow_branching_while_state_project_with_post_flow_cond_loop_shape`
* `lowers_post_flow_branching_continuing_while_state_project_with_post_flow_continue_cond_loop_shape`

### 2. Async Network Observer Steps

Status:

* `mature path`

What is solid now:

* async `step(...)` helpers can read `NetworkResult`
* network observer/value probes survive into executable async loop lowering
* observer-only network steps can feed async carry chains

Primary anchors:

* [tests_async_network_runtime.rs](../../tools/nuisc/src/lowering/tests_async_network_runtime.rs)
* [network_exprs.rs](../../tools/nuisc/src/lowering/network_exprs.rs)

Named proof:

* `lowers_async_network_observer_step_into_async_loop_carry_chain`

### 3. Owned Network Session Steps

Status:

* `mature path`

What is solid now:

* `open/send/recv/close`-style host session steps can live inside one async
  helper
* those helpers can lower into `async_post_flow_chain`
* ready/value observation survives alongside host-owned session calls

Primary anchors:

* [tests_async_network_runtime.rs](../../tools/nuisc/src/lowering/tests_async_network_runtime.rs)

Named proof:

* `lowers_async_owned_network_session_step_into_async_post_flow_break_chain`

### 4. Budgeted Network Polling Loops

Status:

* `mature path`

What is solid now:

* async network steps can drive multi-carry loops
* one carry can model attempts/retries while another models cumulative bytes
* post-flow break can key off a later carry instead of the first carry
* retry/timeout budgets are now part of the honest lowering story

Primary anchors:

* [tests_async_network_runtime.rs](../../tools/nuisc/src/lowering/tests_async_network_runtime.rs)
* [tests_loop_post_flow.rs](../../tools/nuisc/src/lowering/tests_loop_post_flow.rs)

Named proofs:

* `lowers_async_network_poll_step_with_retry_budget_into_async_post_flow_cond_chain`
* `lowers_async_owned_network_session_step_with_retry_budget_into_async_post_flow_cond_chain`
* `lowers_async_owned_network_session_step_with_timeout_budget_into_async_post_flow_cond_chain`

### 5. HTTP-Like Request Session Naming

Status:

* `bridge path`

What is solid now:

* the same owned-session lowering route can already be expressed with
  higher-level request-oriented naming
* `request_step / request_progress / request_attempts / response_bytes`
  now read as one coherent lowering-backed story

What is not being claimed yet:

* no first-class HTTP protocol surface
* no checked-in request parser/serializer semantics
* no guarantee yet that all HTTP-shaped control states deserve distinct
  lowering paths

Primary anchor:

* [tests_async_network_runtime.rs](../../tools/nuisc/src/lowering/tests_async_network_runtime.rs)
* [net_http_session_loop_bridge_recipe_demo](../../examples/projects/domains/net_http_session_loop_bridge_recipe_demo)
* [network_compile.rs](../../tools/nuisc/tests/network_compile.rs)

Named proof:

* `lowers_async_http_client_request_session_into_async_post_flow_cond_chain`

Practical companion:

* the checked-in example directory above is the current project-facing bridge
  from lowering-backed request-session naming to a real multi-file domain
  sample
* `network_compile.rs` now also keeps a project-backed
  `net_httpish_header_session_recipe_demo` anchor honest for
  request-plan/header/session packet staging plus explicit buffer
  `alloc/store/load/free` flow

## Current Edges

These are the most important current edges to keep honest:

* loop conditional metadata is strongest when conditions are expressed as
  compare-style sources like `current`, `carry`, `prev_current`, or
  `prev_carry`
* raw boolean observer calls such as `network_send_ready(...)` are well-covered
  inside async step helpers, but are not yet the best-supported direct source
  shape for every conditional-carry/control route
* the checked-in story is currently “HTTP-like request session lowering,” not a
  complete high-level HTTP client API

Short rule:

`we should currently teach network/http lowering as a session-and-budget compile story, not as a fully general protocol runtime story`

## Practical 0.17.0 Rule

For the current line, the shortest honest statement is:

`async lowering is now strong enough to carry observer-driven loops, owned network session steps, budgeted polling loops, and request-shaped naming through one continuous compile story`
