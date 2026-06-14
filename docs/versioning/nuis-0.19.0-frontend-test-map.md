# `nuis` 0.19.0 Frontend Test Map

This file is the short placement map for current frontend regression work.

It exists to answer one practical question quickly:

`when a frontend feature moves, which test file should take the next proof?`

Companion current-state matrix:

* [nuis-0.19.0-frontend-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-frontend-capability-matrix.md)

## Short Rule

Prefer adding the new test to the narrowest existing file that already owns the
same failure mode.

If a new scenario combines several layers, place it where the most advanced
interaction is being defended, not where the syntax first appears.

## Current Test Spine

### Core generic specialization

File:
`tools/nuisc/src/frontend/tests_generics.rs`

Use this when the main thing being proved is:

* monomorphization of generic functions, aliases, or async bodies
* expected-type driven inference
* nested alias reconstruction
* generic control-flow lowering where higher-order/lambda behavior is not the
  primary novelty

Short rule:

`if the headline is generic specialization shape, start here`

### Higher-order and callable specialization

File:
`tools/nuisc/src/frontend/tests_higher_order.rs`

Use this when the main thing being proved is:

* `Fn1` / `Fn2` / `Fn3` helper specialization
* named function values passed as callables
* callable aliases such as `type Mapper<T> = Fn1<T, T>`
* interaction between generic specialization and callable threading
* capture threading through higher-order helpers
* recursive or control-flow scenarios whose key risk is still higher-order
  rewriting

Short rule:

`if a helper like __hof_* is part of the story, this is usually the right home`

### Lambda surface and capture lowering

File:
`tools/nuisc/src/frontend/tests_lambda_higher_order.rs`

Use this when the main thing being proved is:

* lambda synthesis into private helper functions
* immediate lambda invocation
* capture parameter threading
* named lambda bindings and invoke-form behavior
* non-generic higher-order wiring around lambdas

Short rule:

`if the question is “what exact lambda helper gets built?”, start here`

### Generic method-bound validation

Primary files:

* `tools/nuisc/src/frontend/tests_generic_method_bounds.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs`
* `tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs`

Use these when the main thing being proved is:

* generic parameter method use requires a trait bound
* the bound check must survive a particular surface shape:
  plain body, `if`, nested `match`, destructure, lambda body, or guard

Short rule:

`if the expected result is a bound error message, put the test in the shape-specific bound file`

## Placement Guide For Current Mainline Work

### Add to `tests_higher_order.rs` when

* generic named function values are threaded through `FnN`
* callable aliases and capture lambdas combine
* control-flow or recursion exists mainly to stress higher-order specialization
* trait bounds must survive helper specialization plus lambda capture

Recent examples:

* capturing generic lambda with bound
* nested `while`/`match` plus helper specialization
* recursive async generic body plus payload helper plus captured lambda

### Add to `tests_generics.rs` when

* the hard part is reconstructing concrete generic types
* the proof needs deep struct/alias result shapes
* the scenario is more about specialized return types than callable rewriting

### Add to `tests_lambda_higher_order.rs` when

* the scenario can stay concrete
* the proof cares about helper parameter order or direct invoke lowering
* there is no need to prove trait-bound behavior on generic parameters

## Escalation Rule

If a new behavior would require copying large helper utilities between test
files, prefer one of these first:

1. add the test beside the closest existing owner even if the file grows
2. extract a tiny local helper inside that file
3. create a new focused test file only when a distinct axis has clearly formed

Avoid creating a new test file just because one scenario is longer.

## Current Gaps To Watch

These areas are now present enough that future work should stay intentional:

* generic specialization plus higher-order helper plus capture threading
* bound validation plus lambda bodies
* recursive async generic bodies that still cross helper specialization
* qualified helper traits plus alias chains plus higher-order specialization

If one of these moves, update the owning test file first before broad cleanup.
