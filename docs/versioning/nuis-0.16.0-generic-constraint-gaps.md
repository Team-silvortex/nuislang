# `nuis` 0.16.0 Generic Constraint Remaining-Gaps Checklist

This file is the practical follow-up to
[nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md).

The coverage file answers:

* what is already strong enough to lean on

This file answers:

* what still looks worth tightening before or after the `0.16.0` line settles

Use it as a working checklist, not as a promise that every item belongs in the
release gate.

## Highest-Value Remaining Gaps

These are the most likely follow-up items if we want the generic constraint
surface to feel more complete without changing its overall philosophy.

### 1. Broader Payload-Constructor Inference

Current state:

* payload-style generic construction works
* direct single-field payload constructors can now infer from the payload
  expression itself
* transparent forwarding aliases such as `type Alias<T> = Just<T>` can reuse
  that route
* unsupported alias shapes now fail honestly instead of degrading into
  unrelated call errors

Good next checks:

* [ ] decide whether constructor inference should widen beyond the current
  single-field payload route
* [ ] decide whether non-transparent alias targets should remain intentionally
  unsupported for `0.16.0`
* [ ] decide whether call-argument expected-type inference should expand for
  payload constructors beyond today’s direct route
* [x] keep diagnostics honest if unsupported alias-constructor inference still
  fails

This is ergonomics work, not a correctness emergency.

### 2. Pattern Surface Breadth

Current state:

* payload and struct-field generic binding routes are practical
* the frontend does not claim full ADT-pattern completeness

Good next checks:

* [ ] decide whether more pattern forms should participate in generic
  receiver/binding validation
* [ ] confirm nested combinations keep consistent binding-type truth
* [ ] keep the docs narrow if the pattern language stays intentionally partial

This is mostly a clarity-and-scope item.

### 3. Diagnostic Source Precision

Current state:

* lambda and higher-order helper names are restored to source-facing function
  contexts

Good next checks:

* [ ] decide whether more synthesized helper families need source-context
  restoration
* [ ] decide whether line/region-style source hints are worth the complexity
* [ ] keep helper-name leakage out of user-facing diagnostics

This is UX work, not semantic correctness work.

### 4. More Complex Lambda Surface

Current state:

* no-capture lambdas are practical
* nested/capturing lambda growth is intentionally still narrow

Good next checks:

* [ ] keep generic method-bound validation aligned if lambda surface expands
* [ ] decide whether nested lambda forms belong before or after `0.16.0`
* [ ] do not claim capture support until lowering and diagnostics both match

This is a future-surface item, not a current regression.

## Lower-Level Validation Follow-Ups

These are more implementation-facing than release-facing, but they are worth
tracking.

### 5. Synthetic-Binding Coverage Audits

We already tightened:

* call-inferred locals
* call receivers
* call-root destructure bindings
* call-scrutinee payload bindings

Good next checks:

* [ ] audit whether any remaining local-binding routes still depend on
  variable-only type lookup
* [ ] audit whether any remaining control-flow routes clone binding envs too
  narrowly
* [ ] add one probe test whenever a new binding surface is introduced

### 6. Generic Return-Type Inference Coupling

Current state:

* method-bound validation now leans more on `infer_ast_expr_type(...)`

Good next checks:

* [ ] keep validation-side inference aligned with generic rewrite / lowering
* [ ] make sure inferred local types and receiver types do not silently drift
  from actual lowered behavior
* [ ] add regression tests when inference rules expand

This is the most important “keep it honest” implementation check.

## Ship-Versus-Later Split

These are reasonable `0.16.0` ship-level expectations:

* [ ] direct, alias-aware, control-flow-local, lambda, higher-order, payload,
  and destructure generic method-bound routes keep passing
* [ ] direct payload constructors and transparent alias payload constructors
  keep their current explicit / expected / inferred coverage
* [ ] unsupported alias-constructor shapes keep failing with explicit
  constructor-surface diagnostics
* [ ] user-facing diagnostics stay source-oriented enough to teach
* [ ] no new binding surface lands without at least one generic-bound probe

These are reasonable post-`0.16.0` growth items:

* [ ] broader unconstrained generic payload inference across more constructor
  shapes
* [ ] fuller pattern-language completeness
* [ ] richer source-location diagnostics
* [ ] broader lambda feature growth

## Working Rule

If a remaining item changes correctness, add tests first.

If a remaining item changes ergonomics only, update docs and examples together.

If a remaining item would widen the language claim, do not silently let the
docs get ahead of the implementation.
