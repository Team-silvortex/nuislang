# `nuis` 0.19.0 Generic Constraint Validator

This file is the short implementation-facing map for the current generic
constraint validator in the `0.19.*` line.

It answers one narrow question:

`what exactly is the current generic constraint validator responsible for, and where does each part live?`

Use it together with:

* [nuis-0.19.0-compile-workflow.md](nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-frontend-capability-matrix.md](nuis-0.19.0-frontend-capability-matrix.md)
* [nuis-0.19.0-mainline-regression-matrix.md](nuis-0.19.0-mainline-regression-matrix.md)

## Short Rule

The current validator is best read as four stacked checks:

1. declaration-time bound shape validation
2. use-site concrete bound satisfaction
3. generic receiver method/operator/explicit-trait-call bound validation
4. visible trait-name variant disambiguation

Short rule:

`0.19.*` generic validation is no longer just “does T have a bound?”; it is now “does the declaration make sense, does the concrete use satisfy it, does receiver-side use respect it, and are visible trait names unambiguous enough to trust?”`

## Layer 1: Declaration-Time Bound Shape Validation

Current responsibility:

* generic bounds must name visible traits
* generic bounds must currently stay in the supported “bare trait name” shape
* trait/impl declarations are checked for coherence against those visible names

Primary code:

* [validation_trait_bounds.rs](../../tools/nuisc/src/frontend/validation_trait_bounds.rs)
* [validation_generic_constraints.rs](../../tools/nuisc/src/frontend/validation_generic_constraints.rs)

Primary regression family:

* [tests_generic_constraints.rs](../../tools/nuisc/src/frontend/tests_generic_constraints.rs)

Current examples of failures here:

* unknown bound trait
* unsupported bound shape like `Pipe<i64>`
* impl for unknown trait
* duplicate impl for one `(trait, type)` pair
* impl method missing/extra/signature mismatch

## Layer 2: Use-Site Concrete Bound Satisfaction

Current responsibility:

* if a generic parameter resolves to a concrete type at specialization/use site,
  that concrete type must satisfy the declared bound
* this now applies consistently across inferred and explicit generic arguments
* helper-visible trait-name variants are accepted when one visible variant is
  uniquely compatible with the concrete impl set
* helper-visible trait-name variants are rejected when multiple compatible
  variants exist and the short-name use would be ambiguous

Primary code:

* [validation_trait_bounds.rs](../../tools/nuisc/src/frontend/validation_trait_bounds.rs)
* [generics.rs](../../tools/nuisc/src/frontend/generics.rs)

Primary regression family:

* [tests_generic_constraints.rs](../../tools/nuisc/src/frontend/tests_generic_constraints.rs)

Current supported truth:

* explicit generic call args and inferred generic call args now reach the same
  concrete bound-satisfaction helper
* visible helper trait variants can satisfy a short-name bound if and only if
  that match is unique

Current examples of failures here:

* `type Text does not satisfy bound Addable for generic parameter U`
* `type i64 ambiguously satisfies bound Addable for generic parameter U`

## Layer 3: Generic Receiver Bound Validation

Current responsibility:

* generic receiver method calls require the right trait bound
* generic receiver operators require the right trait bound
* explicit trait calls on generic receivers require the right trait bound
* all three now share one generic-receiver context/bound-resolution skeleton

Primary code:

* [validation_method_bounds.rs](../../tools/nuisc/src/frontend/validation_method_bounds.rs)

Primary regression families:

* [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* [tests_generic_method_bounds_control_flow.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)
* [tests_generic_method_bounds_if_bindings.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs)
* [tests_generic_method_bounds_lambda_bindings.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs)
* [tests_generic_method_bounds_nested_match.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs)

Current diagnostic truth:

* missing required bound
* wrong bound for the requested method/operator/trait
* candidate bound suggestions where available
* explicit helper trait-name consistency diagnostics

## Layer 4: Visible Trait-Name Variant Disambiguation

Current responsibility:

* helper-visible traits may be referenced by short or qualified names
* the validator now distinguishes:
  * one compatible visible variant:
    accept
  * multiple compatible visible variants:
    reject as ambiguous
  * incompatible visible variant naming on explicit trait receiver routes:
    reject and ask for one consistent visible name

Primary code:

* [validation_trait_bounds.rs](../../tools/nuisc/src/frontend/validation_trait_bounds.rs)
* [validation_method_bounds.rs](../../tools/nuisc/src/frontend/validation_method_bounds.rs)

Primary regression families:

* [tests_generic_constraints.rs](../../tools/nuisc/src/frontend/tests_generic_constraints.rs)
* [tests_generic_method_bounds.rs](../../tools/nuisc/src/frontend/tests_generic_method_bounds.rs)

Current examples:

* short-name bound accepted through one visible helper trait variant
* short-name bound rejected when `HelperA.Addable` and `HelperB.Addable` both
  match the same concrete impl target
* explicit trait call rejected when bound name and call name refer to the same
  visible trait through different spellings

## Current Working Mental Model

When debugging a generic constraint failure in `0.19.*`, read it in this order:

1. is the bound declaration itself valid?
2. did specialization/use-site choose a concrete type that satisfies the bound?
3. if the error comes from a generic receiver use, is it method/operator/trait-call specific?
4. if helper-visible trait names are involved, is the name unique or ambiguous?

Short rule:

`if a generic constraint failure feels surprising, the first thing to check is whether the failure belongs to declaration shape, use-site satisfaction, receiver-side method use, or visible-name ambiguity; the current repository has a different checked-in validator layer for each of those`
