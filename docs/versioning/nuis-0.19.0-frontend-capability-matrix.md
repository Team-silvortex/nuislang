# `nuis` 0.19.0 Frontend Capability Matrix

This file is the short current-state matrix for the frontend mainline.

It exists to answer one practical question quickly:

`what combinations are already proved together, and which test file owns that proof?`

## Short Rule

Read this file as a feature-combination map, not as a changelog.

If a scenario mixes several frontend layers, the important question is:

`which interactions are already defended as one path?`

## Current Combined Capability Matrix

### Generic bounds and receiver methods

Current truth:

* generic receiver method calls require explicit trait bounds
* missing-bound and wrong-bound diagnostics are both covered
* alias-chain context is preserved in diagnostics
* control-flow shapes already covered:
  plain body, `if`, `while`, nested `match`, guarded `match`, destructure

Primary test spine:

* `tools/nuisc/src/frontend/tests_generic_method_bounds.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs`

Short rule:

`method-bound validation is no longer just a flat-body feature`

### Generic bounds and operator validation

Current truth:

* binary operators covered:
  `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`
* unary operators covered:
  `!`, unary `-`
* both missing-bound and wrong-bound diagnostics are covered
* alias-chain diagnostic context is preserved for operator paths too
* helper-trait qualified bounds such as `Helper.Addable` are accepted for
  operator validation

Primary test spine:

* `tools/nuisc/src/frontend/tests_generic_method_bounds.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs`

Short rule:

`operator bound validation now shares the same richer context story as method calls`

### Higher-order specialization and generic lambdas

Current truth:

* `Fn1` / `Fn2` / `Fn3` helper specialization is proved
* generic named function values can satisfy callable expectations
* callable aliases are covered
* capture threading through higher-order helpers is covered
* recursive async and nested control-flow scenarios are already represented

Primary test spine:

* `tools/nuisc/src/frontend/tests_higher_order.rs`

Short rule:

`if __hof_* exists in the story, current mainline already treats that as a first-class proof path`

### Higher-order specialization plus trait/helper bounds

Current truth:

* generic lambdas with trait-bound receiver methods are covered
* captured lambdas with explicit trait calls are covered
* captured operator lambdas distinguish builtin lowering from trait lowering
* helper-trait qualified bounds such as `Helper.Addable` survive higher-order
  specialization
* helper-trait qualified bounds also survive alias-chain receivers inside
  higher-order generic lambda specialization

Primary test spine:

* `tools/nuisc/src/frontend/tests_higher_order.rs`

Short rule:

`helper traits, capture threading, alias expansion, and specialization now meet on one current route`

## Current Proven Routes

These are the shortest “already real together” routes worth remembering.

### Route A

`generic bound -> operator use -> alias chain -> diagnostic context`

Anchor:

* `tools/nuisc/src/frontend/tests_generic_method_bounds.rs`

### Route B

`generic bound -> lambda body -> operator/method validation`

Anchor:

* `tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs`

### Route C

`generic callable -> higher-order helper -> captured lambda -> trait/operator lowering`

Anchor:

* `tools/nuisc/src/frontend/tests_higher_order.rs`

### Route D

`qualified helper trait -> alias chain -> higher-order specialization -> monomorphized impl symbol`

Anchor:

* `tools/nuisc/src/frontend/tests_higher_order.rs`

## Current Boundaries

These are important because they explain why some tests assert different IR
shapes.

* explicit trait calls lower to trait impl symbols directly
* builtin numeric operators may still lower to builtin `NirExpr::Binary`
  even when generic validation required a trait bound earlier
* non-builtin structural/operator paths on custom types lower through impl
  symbols such as `impl.Helper.Addable.for.i64.add`

Short rule:

`bound validation and final lowered shape are related, but not always identical`

## Usage Rule

When adding the next frontend regression:

1. identify the most advanced interaction in the scenario
2. find the matching route above
3. add the test beside that route’s current owner first

If the scenario does not fit any route above, the matrix probably needs an
update along with the code.
