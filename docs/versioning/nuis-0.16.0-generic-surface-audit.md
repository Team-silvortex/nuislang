# `nuis` 0.16.0 Generic Surface Audit

This file is the compiler-facing audit checklist for the current `0.16.0`
generic surface.

It is narrower than the snapshot docs and more concrete than the gaps list.

Use it when the practical question is:

* which generic surface combinations already have test-backed closure
* which combinations are still only partially covered
* where should the next probe test land before widening claims

Use it alongside:

* [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)
* [nuis-0.16.0-generic-constraint-gaps.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-gaps.md)
* [nuis-0.16.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-release-checklist.md)

## Reading Rule

Interpret each row like this:

* `closed` means the route is exercised strongly enough to lean on in compiler work
* `partial` means the route works in some practical forms but should not be over-claimed
* `open` means we should assume it still needs dedicated probes before widening docs

## Constructor And Literal Matrix

### Payload Constructors

* `closed`:
  direct payload constructor with explicit type args
  `Just<i64>(7)`
* `closed`:
  direct payload constructor with expected type
  `let payload: Just<i64> = Just(7);`
* `closed`:
  direct payload constructor with inferred payload type
  `let payload = Just(7);`
* `closed`:
  transparent alias payload constructor with explicit type args
  `JustAlias<i64>(7)`
* `closed`:
  transparent alias payload constructor with inferred payload type
  `JustAlias(7)`
* `closed`:
  non-transparent alias payload constructor when the target pattern is still fully inferable
  `type WrappedAlias<T> = Just<Boxed<T>>; WrappedAlias(Boxed { value: 7 })`
* `closed`:
  alias constructor failure with generic-arity mismatch
* `closed`:
  alias constructor failure with target-field shape mismatch
* `closed`:
  alias constructor failure with unresolved generic parameter

Primary tests:

* [tests_generic_structs.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_structs.rs)

### Struct Literals

* `closed`:
  direct generic struct literal with explicit type args
  `Boxed<i64> { value: 7 }`
* `closed`:
  direct generic struct literal with expected type
  `let boxed: Boxed<i64> = Boxed { value: 7 };`
* `closed`:
  direct generic struct literal with inferred field types
  `let boxed = Boxed { value: 7 };`
* `closed`:
  multi-field direct generic struct literal inference
  `Pair { left: 7, right: 9 }`
* `closed`:
  nested direct generic struct literal inference
  `Wrapper { inner: Boxed { value: 7 }, tag: 1 }`
* `closed`:
  transparent alias struct literal with inferred field types
  `BoxAlias { value: 7 }`
* `closed`:
  non-transparent alias struct literal when the target pattern is still fully inferable
  `type WrappedStructAlias<T> = Wrapper<Boxed<T>>;`
  `WrappedStructAlias { inner: Boxed { value: 7 }, tag: 1 }`
* `closed`:
  field-insufficient direct generic struct literal failure
  `Phantom { value: 7, tag: 1 }`
* `closed`:
  field-insufficient generic alias struct literal failure with unresolved alias generic
  `PhantomAlias { value: 7, tag: 1 }`

Primary tests:

* [tests_generic_structs.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_structs.rs)

## Specialization Matrix

### Generic Function Calls

* `closed`:
  direct generic function specialization from direct scalar arguments
* `closed`:
  specialization from inferred direct struct literal arguments
  `unwrap_box(Boxed { value: 7 })`
* `closed`:
  specialization from inferred transparent alias struct literal arguments
  `unwrap_box(BoxAlias { value: 7 })`
* `closed`:
  specialization from inferred non-transparent alias struct literal arguments
  `unwrap_wrapped(WrappedStructAlias { inner: Boxed { value: 7 }, tag: 1 })`
* `closed`:
  specialization from inferred direct payload constructor arguments
  `unwrap_just(Just(7))`
* `closed`:
  specialization from inferred transparent alias payload constructor arguments
  `unwrap_just(JustAlias(7))`
* `closed`:
  zero-arg generic specialization from async return expectation through `await`
  `return await typed_zero();`
* `closed`:
  zero-arg generic async specialization through awaited nested alias wrapper into generic call argument
  `keep_response(Response(await typed_box()))`
* `closed`:
  generic nested alias task payload through `spawn` / `join` and branch return
  `keep_response(join(task))` where `task: Task<Response<i64>>`
* `closed`:
  generic response unwrap through `Task<Response<T>>`, `join(task)`, and branch-local constructors
  `unwrap_response(join(task))` and `unwrap_response(Response { ... })`
* `closed`:
  network-shaped request/exchange/result flow through aliases, `spawn` / `join`, and branch-local result construction
  `let task: Task<HttpResult<i64>> = spawn(exchange(request));`
  `read_body(join(task))` and `read_body(HttpResult { ... })`
* `closed`:
  `std net` facade-shaped HTTP session flow through `net_http_request`, `net_http_client_exchange`, `net_result`, and `net_session`
  `let task: Task<NetSession<i64>> = spawn(net_session(request));`
  `net_http_response_value(join(task))` and `net_http_response_value(NetSession { ... })`
* `closed`:
  `std net` demo-shaped summary/session flow through exchange summary, session summary, and `spawn` / `join`
  `let task: Task<NetSessionSummary<i64>> = spawn(capture_net_session_summary(request));`
  `summarize_net_session(join(task))` and `summarize_net_session(SessionSummary { ... })`
* `closed`:
  nested generic async summary helpers through alias-heavy struct literals with expected-type-driven field propagation
  `summary: await capture_net_http_client_exchange_summary(request)` inside `capture_net_session_summary<T>`
* `closed`:
  `match`-lowered control flow over std-net-shaped summary/session tasks with alias-heavy branch-local reconstruction
  `return SessionSummary { summary: join(summary_task), session_value: 99 };`
  inside `match mode { ... }`
* `closed`:
  `while`-body control flow over std-net-shaped summary/session tasks with alias-heavy branch-local reconstruction
  `while seed > 0 { return SessionSummary { summary: join(summary_task), session_value: 99 }; }`
* `closed`:
  nested `while -> match` control flow over std-net-shaped summary/session tasks with alias-heavy branch-local reconstruction
  `while seed > 0 { match mode { 1 => return SessionSummary { summary: join(summary_task), ... }, _ => ... } }`
* `closed`:
  higher-order scrutinee control flow over std-net-shaped summary/session tasks with hoisted `match` input
  `match apply(mode, |x| x + 1) { 2 => return SessionSummary { summary: join(summary_task), ... }, _ => ... }`

Primary tests:

* [tests_generics.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generics.rs)
* [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)
* [mod.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs)

### Control-Flow Closure

Current practical closure for the `std net`-shaped generic/task/session route:

* `closed`:
  straight-line expected-type propagation through alias-heavy summary/session reconstruction
* `closed`:
  `if` branch-local reconstruction with `spawn` / `join`
* `closed`:
  `match`-lowered branch-local reconstruction with `spawn` / `join`
* `closed`:
  `while`-body reconstruction with `spawn` / `join`
* `closed`:
  nested `while -> match` reconstruction with `spawn` / `join`
* `closed`:
  higher-order scrutinee hoisting feeding nested control-flow reconstruction

Practical reading rule:

* the current `0.16.0` compiler truth is no longer “control flow works in simple scalar examples”
* the stronger claim we can now lean on is:
  alias-heavy summary/session reconstruction remains stable across `if`, `match`, `while`,
  nested `while -> match`, and hoisted higher-order scrutinee routes

### Lowering Closure Companion

The rows above are frontend-facing generic closure.

The current backend-facing companion truth is now stronger than it was earlier
in the `0.16.0` line:

* `closed`:
  loop-family lowering accepts branch-local `break` / `continue` before carry updates
* `closed`:
  loop-family lowering accepts branch-local `break` / `continue` after carry updates
* `closed`:
  match-hoisted control temps still feed loop-family lowering correctly
* `closed`:
  nested `if -> break` loop control lowers into `and` compound loop predicates
* `closed`:
  nested `match` / branch-local `continue` loop control lowers into `or` compound loop predicates
* `partial`:
  generic/front-end control-flow closure is broader than executable loop lowering in one important way:
  arbitrary iterative/backedge loops that do not fit the counted/carry/flow/post-flow families are still out of scope

Practical reading rule:

* frontend generic closure should not be confused with “all loops lower”
* the honest current claim is narrower and better:
  control-flow-heavy generic routes are strong,
  and the executable lowering subset underneath them is now explicitly wider across
  counted/carry/flow/post-flow families, including nested `and` / `or` loop predicates

### Higher-Order Specialization

* `closed`:
  generic higher-order specialization through direct generic values
* `closed`:
  generic higher-order specialization through explicit alias payload constructors
  `apply_payload(JustAlias<i64>(6), |x| ...)`
* `closed`:
  generic higher-order specialization through inferred alias payload constructors
  `apply_payload(JustAlias(6), |x| ...)`
* `closed`:
  async-awaited zero-arg generic flowing through inferred alias payload constructor into higher-order specialization
  `apply_payload(JustAlias(await typed_zero()), |x| ...)`

Primary tests:

* [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)

## Pattern And Binding Matrix

### Payload Patterns

* `closed`:
  explicit generic payload binding
  `Just<i64>(payload)`
* `closed`:
  alias-aware generic payload binding
  `JustAlias<i64>(payload)`
* `closed`:
  inferred alias payload constructor flowing into guarded payload match binding
  `let value = JustAlias(2); match value { JustAlias<i64>(payload) if ... }`

Primary tests:

* [tests_match_payload_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_match_payload_bindings.rs)
* [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)

### Struct Patterns

* `closed`:
  direct generic struct binding with shorthand fields
* `closed`:
  aliased generic struct binding with explicit source value type
* `closed`:
  inferred alias struct literal flowing into guarded struct match binding
  `let value = BoxAlias { value: 7 }; match value { BoxAlias<i64> { value: payload } if ... }`

Primary tests:

* [tests_match_struct_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_match_struct_bindings.rs)

### Destructure Bindings

* `closed`:
  direct generic struct shorthand destructure
  `let { value: payload } = boxed;`
* `closed`:
  call-root generic destructure feeding later method-bound validation
  `let { value: payload } = wrap(value);`
* `closed`:
  inferred alias struct literal flowing into shorthand destructure
  `let value = BoxAlias { value: 7 }; let { value: payload } = value;`

Primary tests:

* [tests_generic_destructure_let.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_destructure_let.rs)

## Method-Bound Matrix

### Generic Receivers And Locals

* `closed`:
  direct generic receivers
* `closed`:
  alias-aware generic receivers
* `closed`:
  call-inferred locals and call receivers
* `closed`:
  destructure roots inferred from calls
* `closed`:
  payload-pattern-bound locals
* `closed`:
  control-flow-local routes across `if`, `while`, `match`, and guarded `match`

Primary tests:

* [tests_generic_method_bounds.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)
* [tests_generic_destructure_let.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_destructure_let.rs)

### Lambda And Higher-Order Context

* `closed`:
  lambda bodies inherit generic parameters for method-bound validation
* `closed`:
  higher-order synthesized helpers preserve method-bound diagnostics
* `closed`:
  `Fn1`, `Fn2`, and `Fn3` method-bound routes

Primary tests:

* [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)

## Still Partial

These routes are meaningful, but they should still be treated as intentionally
partial in `0.16.0`.

* `partial`:
  unconstrained constructor inference across every alias/constructor shape
* `partial`:
  pattern-language completeness beyond today’s practical struct/payload surface
* `partial`:
  richer source-location diagnostics beyond current function-context restoration
* `partial`:
  broader lambda surface such as captures and nested inline forms

## Diagnostic Guardrails

* `closed`:
  network-style sync summary builders reject direct async helper calls with stable async-context diagnostics
* `closed`:
  network-style task staging rejects `spawn(...)` on sync summary builders with stable task-entry diagnostics
* `closed`:
  nested control-flow higher-order scrutinees reject lambda capture misuse with stable lambda-capture diagnostics
* `closed`:
  nested control-flow higher-order specialization still reports missing generic method bounds through source-facing specialization context

## Working Audit Rule

Before widening the generic surface claim, prefer this order:

1. add the probe test
2. move the route from `open` or `partial` to `closed`
3. only then widen the coverage or snapshot docs

If a route is not named here, assume it still deserves a dedicated probe before
it becomes part of the `0.16.0` story.
