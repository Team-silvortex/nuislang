# `nuis` 0.16.0 Generic Constraint Coverage Map

This file is the short coverage map for the `0.16.0` generic constraint and
method-bound validation surface.

It answers two questions:

* which generic-binding routes are already covered strongly enough to lean on
* which routes are still intentionally partial or not yet covered

Use it as the practical companion to:

* [nuis-0.16.0-snapshot.md](nuis-0.16.0-snapshot.md)
* [nuis-0.16.0-compile-workflow.md](nuis-0.16.0-compile-workflow.md)
* [nuis-0.16.0-binary-compile-maturity.md](nuis-0.16.0-binary-compile-maturity.md)
* [nuis-0.16.0-generic-constraint-gaps.md](nuis-0.16.0-generic-constraint-gaps.md)
* [nuis-0.16.0-generic-surface-audit.md](nuis-0.16.0-generic-surface-audit.md)

## Short Rule

For `0.16.0`, generic method-bound validation is now strong on:

* alias-aware generic receivers
* control-flow-local environments
* lambda / higher-order synthesized helpers
* payload / struct pattern bindings
* call-inferred locals and call receivers
* call-root destructuring bindings

The remaining gaps are no longer “basic generic method calls”.

They are now mostly about broader ergonomics and future pattern/type-inference
surface growth.

## Covered Strongly

These routes are covered strongly enough to treat as current compiler truth.

### Direct Generic Receivers

Covered:

* `value.add(value)` on `T`
* correct diagnostics for:
  missing bound
* correct diagnostics for:
  wrong bound
* correct diagnostics for:
  ambiguous candidate bounds

Primary tests:

* [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)

### Alias-Aware Generic Receivers

Covered:

* single alias chain `Alias<T>`
* nested alias chains such as `Outer<T> -> Alias<T> -> Inner<T>`
* user-facing diagnostics include alias-chain context instead of silently
  collapsing the route

Primary tests:

* [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)

### Control-Flow Local Bindings

Covered:

* `if` locals
* `while` locals
* `match` locals
* nested `match`
* guarded payload-style `match`
* alias-aware payload-style `match`

Primary tests:

* [tests_generic_method_bounds_if_bindings.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs)
* [tests_generic_method_bounds_nested_match.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs)
* [tests_generic_method_bounds_control_flow.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)

### Lambda And Higher-Order Synthesized Helpers

Covered:

* synthesized `__lambda_*` functions inherit outer generic parameters
* generic method-bound validation runs inside lambda bodies
* diagnostics map `__lambda_*` helper names back to source-facing
  `function <name> body lambda`
* synthesized `__hof_*` higher-order helpers map back to
  `function <name> body higher-order specialization`
* `Fn1`, `Fn2`, and `Fn3` generic lambda method-call routes are covered

Primary tests:

* [tests_higher_order.rs](../../tools/nuisc/src/frontend/tests_higher_order.rs)

Representative demos:

* [generic_lambda_method_bound_hof_demo](../../examples/projects/state/generic_lambda_method_bound_hof_demo)
* [generic_lambda_method_bound_fn3_hof_demo](../../examples/projects/state/generic_lambda_method_bound_fn3_hof_demo)
* [generic_payload_alias_method_hof_demo](../../examples/projects/state/generic_payload_alias_method_hof_demo)

### Call-Inferred Receivers And Locals

Covered:

* direct call receiver:
  `id(value).add(value)`
* call-inferred local:
  `let local = id(value); local.add(value);`
* higher-order-specialized local:
  `let local = f(x); local.add(x);`

Primary tests:

* [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* [tests_higher_order.rs](../../tools/nuisc/src/frontend/tests_higher_order.rs)

### Pattern-Bound Generic Payloads And Struct Fields

Covered:

* payload-style bindings:
  `Just<T>(payload)`
* alias-aware payload bindings:
  `JustAlias<T>(payload)`
* struct-field shorthand bindings:
  `{ value: payload }`
* nested struct-field bindings:
  `Outer<T> { inner: { value: payload }, ... }`
* call-scrutinee payload binding:
  `match wrap(value) { Just<T>(payload) => ... }`

Primary tests:

* [tests_generic_method_bounds_control_flow.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)

### Destructure Bindings From Call Roots

Covered:

* shorthand destructuring from expression roots:
  `let { value: payload } = wrap(value);`
* root-type inference now feeds generic method-bound validation instead of
  only explicit root annotations or variable-only roots

Primary tests:

* [tests_generic_destructure_let.rs](../../tools/nuisc/src/frontend/tests_generic_destructure_let.rs)

## Covered But Still Narrow

These routes work, but still have intentionally narrower ergonomics than a
fully mature future surface.

### Generic Payload Construction

Current rule:

* payload-style generic construction is practical
* single-field payload constructors can now infer generic arguments from the
  payload expression itself in the common direct-constructor route
* transparent forwarding generic aliases can reuse that same inference route
* unsupported alias-constructor shapes now fail with a specific alias-surface
  diagnostic instead of falling through to a misleading unknown-function error

Examples:

* `Just<i64>(7)`
* `Just(7)`
* `JustAlias<i64>(7)`
* `JustAlias(7)` where `type JustAlias<T> = Just<T>`

Constructor matrix:

* direct generic payload constructor with explicit type args:
  `Just<i64>(7)` -> covered
* direct generic payload constructor with expected type:
  `let payload: Just<i64> = Just(7);` -> covered
* direct generic payload constructor with inferred payload type:
  `let payload = Just(7);` -> covered
* transparent forwarding generic alias with explicit type args:
  `JustAlias<i64>(7)` -> covered
* transparent forwarding generic alias with inferred payload type:
  `let payload = JustAlias(7);` -> covered
* explicit alias generic arity mismatch:
  `JustAlias<i64, bool>(7)` -> covered with direct alias-arity diagnostic
* non-transparent generic alias constructor:
  `type WrappedAlias<T> = Just<Boxed<T>>; WrappedAlias(Boxed { value: 7 })`
  -> covered with explicit “not yet supported” diagnostic

Primary tests:

* [tests_generic_structs.rs](../../tools/nuisc/src/frontend/tests_generic_structs.rs)

This is good enough for `0.16.0`, but it is still not “free unconstrained
generic payload inference across arbitrary constructor and alias shapes”.

### Pattern System Breadth

Current rule:

* practical payload/struct/generic pattern routes are covered
* the system is still not trying to claim full ADT-pattern completeness

This is a maturity boundary, not a regression.

## Not Yet Aimed At `0.16.0`

These are reasonable future growth directions, but they should not be claimed
as already mature `0.16.0` coverage.

* broader unconstrained generic inference across all payload constructor forms
* richer pattern language completeness beyond today’s practical struct/payload
  route
* more source-location-rich diagnostics than current function-context
  restoration
* non-MVP lambda surface growth such as nested inline lambda bodies or capture
  support

## Practical Ship Rule

For `0.16.0`, we can honestly say the generic constraint surface is strong
enough when all of the following stay true:

* direct generic receivers diagnose clearly
* alias-chain contexts stay visible
* control-flow-local bindings keep their generic receiver truth
* lambda / higher-order helpers do not leak raw synthesized helper names in
  user-facing diagnostics
* call-return-type inference continues feeding method-bound validation for
  locals, receivers, and destructure roots
* the representative project demos continue to pass `nuis check`

If one of those stops being true, the docs should narrow or the compiler should
be fixed before the `0.16.0` story is called stable.
