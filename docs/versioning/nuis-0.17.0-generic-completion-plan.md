# `nuis` 0.17.0 Generic Completion Plan

This file is the practical execution map for the generic-completion track of
the `0.17.0` line.

It sits between the broad mainline goals and the older `0.16.0` closure docs.

Use it when the question is not:

* what generics are in principle supposed to do

but instead:

* which generic routes already feel closed enough to lean on
* which gaps most directly block the next stage of compiler maturity
* what order of work is most likely to strengthen multiple layers at once

## Reading Rule

Interpret this file conservatively:

* `foundation` means the route is already strong enough to reuse as a baseline
* `priority` means the route is a likely `0.17.0` target because it closes
  multiple remaining gaps
* `later` means the route may be valuable, but should not distract from the
  shared spine first

This file is a working map, not a promise that every item lands in one pass.

## Current Foundation We Should Preserve

These are the `0.16.*` generic truths that now form the base of `0.17.0`
rather than its main unfinished story:

* `foundation`:
  alias-aware generic structs, payload constructors, shorthand destructuring,
  shorthand `match`, and pattern-facing binding checks
* `foundation`:
  generic constraint and method-bound validation across alias chains,
  control-flow-local bindings, call-inferred locals, and lambda/higher-order
  helper contexts
* `foundation`:
  expression-level specialization through explicit helper chains
* `foundation`:
  generic-body internal explicit helper self-use under outer specialization
* `foundation`:
  higher-order generic helper routes that survive lambda lifting
* `foundation`:
  real project compile closure for the bridge-style
  `net_http_session_loop_bridge_recipe_demo` route

Short rule:

`0.17.0` should assume these routes are worth preserving and expanding, not reopening from scratch`

## Priority 1: Remove Remaining “Works Here But Not There” Gaps

These are the highest-value targets because they improve coherence across
frontend, rewrite, lowering, and project compilation at the same time.

### 1. Field-Access-Driven Generic Inference

Current pain:

* some generic routes were fragile when arguments flowed through field access
  instead of through a typed local or direct constructor expression

Why it matters:

* it affects ordinary source ergonomics
* it reduces confidence in higher-order and helper-heavy code
* it is exactly the kind of gap that makes the compiler feel less coherent than
  it already is

Checklist:

* [x] audit which generic inference paths still rely too heavily on pre-typed
  locals instead of source expressions
  current closed route:
  alias-aware field access feeding generic specialization now no longer
  requires workaround-shaped typed locals in the bridge-style helper path
* [x] add focused probes for field-access-fed generic helper calls
  current closed probes:
  `monomorphizes_higher_order_generic_mapper_from_field_access_arguments_without_typed_locals`
  and
  `lowers_generic_alias_payload_constructor_from_alias_field_access`
* [x] promote any fixed route into a real project compile anchor where possible
  current closed anchor:
  `net_http_session_loop_bridge_recipe_demo` no longer needs the typed-local
  workaround around `packet.payload` / `packet.packet_value`
* [ ] keep failures source-local and specific when unsupported routes remain
* [ ] continue auditing deeper field-access-heavy routes, especially where
  field access chains combine with more indirect higher-order or branch-local
  reconstruction patterns

### 2. Expected-Type Propagation Continuity

Current pain:

* some routes already propagated expected types well through nested helper
  chains, but that continuity was not yet uniform across more expression shapes

Why it matters:

* it is the backbone of generic ergonomics
* it directly affects async/task wrappers, branch-local reconstruction, and
  lambda-lifted helper bodies

Checklist:

* [x] audit expected-type propagation across field access, method calls, and
  reconstructed branch-local values
  current closed shapes:
  field-access-fed helper arguments,
  zero-arg generic calls used as method receivers,
  branch-local reconstructed payload values immediately consumed by a
  following generic call,
  and the same branch-local payload values forwarded through one simple local
  alias before the generic call,
  plus one simple forwarding helper call before the final generic consumer
* [x] verify explicit generic args and inferred generic args keep the same
  internal specialization truth where they should
  current practical closure:
  explicit helper chains, inferred alias payload routes, method-receiver
  specialization, and branch-local payload reconstruction all now reach the
  same `typed_zero__i64` / concrete helper specialization outcomes in checked
  probes
* [x] add at least one regression test per newly widened expression shape
  current added probes:
  `monomorphizes_zero_arg_generic_call_used_as_method_receiver`
  and
  `monomorphizes_branch_local_payload_reconstruction_before_generic_call`
* [ ] continue auditing expression shapes that still do not benefit from this
  continuity, especially deeper method-call receiver chains and less-immediate
  branch-local uses than the current simple local-forwarding-plus-one-helper
  lookahead route

### 3. Real Project Generic Closure Beyond One Anchor

Current pain:

* the bridge demo was a strong proof, but one real project anchor was not the
  same thing as broad project-level closure

Why it matters:

* `0.17.0` should not stop at frontend greenness
* project compilation is where integration truth becomes believable

Checklist:

* [x] identify the next best project demo that should carry generic helper or
  higher-order specialization truth
  current added anchor:
  `examples/projects/state/generic_payload_alias_method_hof_demo`
  and
  `examples/projects/state/generic_callable_forwarding_hof_demo`
* [x] ensure at least one non-network project route also exercises the same
  generic completion claims if practical
  current compile-harness proof:
  `tools/nuisc/tests/state_compile.rs`
* [x] add compile-harness checks instead of relying only on ad hoc manual runs
  current added proofs:
  `generic_payload_alias_higher_order_demo`
  and
  `generic_payload_alias_method_hof_demo`
  and now
  `generic_callable_forwarding_hof_demo` for project-shaped `Fn2` / `Fn3`
  callable forwarding through nested `relay -> chain -> apply` helper lanes

## Priority 2: Make Diagnostics Match The Stronger Surface

These are not purely cosmetic. They are important because a stronger generic
surface without correspondingly clear diagnostics becomes harder to trust.

### 4. Source-Facing Failure Precision

Checklist:

* [ ] keep synthesized helper names out of user-facing errors where possible
* [ ] make unresolved specialization failures point at the most local source
  route rather than a distant lowered helper when possible
* [ ] preserve source-facing context for higher-order and lambda-lifted routes

### 5. Unsupported-But-Intentional Generic Routes

Checklist:

* [ ] keep intentionally unsupported constructor or inference shapes failing
  honestly
* [ ] avoid regressions where unsupported routes degrade into unrelated arity or
  lowering errors
* [ ] update docs the same turn that an unsupported route becomes supported

## Priority 3: Broaden Only After Coherence Improves

These are valid future directions, but they should follow coherence work rather
than replace it.

### 6. Broader Constructor Inference

Checklist:

* [ ] decide whether inference should expand beyond the current strongest
  payload and struct-literal routes
* [ ] keep alias-shape support narrow unless the implementation truly becomes
  broader

### 7. Richer Lambda Surface

Checklist:

* [ ] do not widen lambda claims without matching generic validation,
  specialization, and diagnostics
* [ ] keep nested/capturing-lambda work explicitly scoped if it starts

### 8. Broader Pattern Completeness

Checklist:

* [ ] expand pattern-generic interactions only when the binding and validation
  model stays easy to explain
* [ ] avoid overclaiming full pattern completeness before it is real

## Suggested Work Order

If we want the shortest path to visible `0.17.0` progress, prefer this order:

1. field-access and expression-shape generic inference gaps
2. expected-type propagation continuity across more rewritten expressions
3. one more real project compile anchor
4. source-facing diagnostic precision cleanup
5. only then broader language-surface widening

## Success Signals

We should feel `0.17.0` progress here when:

* fewer examples need workaround-shaped typed locals just to help generics land
* more helper-heavy generic routes work the same in toy tests and project demos
* failure modes become easier to read when a route is still unsupported
* mainline docs can talk about generic completion as cross-layer coherence,
  not only as “more green tests”

## Rule Of Thumb

The best `0.17.0` generic work is not the work that makes the fanciest new
example pass.

It is the work that makes more ordinary valid generic code survive the whole
stack with fewer surprises.
