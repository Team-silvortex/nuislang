# `nuis` `0.20.0` Generic Validation Regression Matrix

This file is the short regression map for the current generic-validation
surface entering the `0.20.*` line.

It answers one practical question:

`which generic validation behaviors are already intentionally defended, which test files own them, and which gaps are still worth treating as active work?`

Read this together with:

* [nuis-0.19.0-generic-constraint-validator.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-generic-constraint-validator.md)
* [nuis-0.19.0-frontend-test-map.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-frontend-test-map.md)
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
* [../reference/generic-diagnostic-ownership-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/generic-diagnostic-ownership-contract.md)

## Short Rule

For `0.20.*`, generic validation should be read as one connected surface, not
as isolated syntax features.

The current proof spine now expects generic constraints to survive:

* declaration-time bound checking
* concrete use-site satisfaction
* generic receiver method/operator validation
* explicit generic call specialization
* struct-literal type-site reconstruction
* `if` / `match` result-branch composition
* lambda-body and higher-order helper contexts

Short rule:

`if a generic bound failure disappears just because the same type moved into a branch, lambda body, or explicit generic call, that is a regression`

## Current Regression Families

### Core constraint and use-site validation

Primary file:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

Current responsibility:

* declaration-time bound shape validation
* concrete type satisfaction for inferred and explicit generic arguments
* helper-visible trait-name ambiguity checks
* generic struct-literal type-site validation
* explicit generic function-call bound validation
* branch-result and lambda-body propagation of the same use-site checks

Current scale:

* about `1717` lines
* this is now the broadest single owner for generic constraint regressions

Short rule:

`if the expected failure is “type X does not satisfy bound Y”, this is usually the first home`

### Generic receiver method-bound validation

Primary files:

* [tests_generic_method_bounds.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)
* [tests_generic_method_bounds_if_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_if_bindings.rs)
* [tests_generic_method_bounds_lambda_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs)
* [tests_generic_method_bounds_nested_match.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs)

Current responsibility:

* generic receiver method calls require the right trait bound
* generic receiver operators require the right trait bound
* explicit trait calls on generic receivers require the right trait bound
* bound diagnostics stay stable through `if`, nested `match`, destructure, and
  lambda-body shapes

Current scale:

* `2208` lines in the main file
* another `1692` lines across the current shape-specific companions

Short rule:

`if the failure belongs to a generic receiver using a method/operator/trait call, the shape-specific method-bound family owns it`

## Current Proven Axes

### 1. Declaration-time bound shape

Proven today:

* unknown traits are rejected
* unsupported bound syntax is rejected
* impl declarations are checked for unknown trait, duplicate impl, and method
  coherence problems

Primary owner:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

### 2. Concrete use-site satisfaction

Proven today:

* inferred generic arguments and explicit generic arguments both flow through
  bound satisfaction checks
* helper-visible short trait names are accepted only when one visible variant
  is uniquely compatible
* ambiguous helper-visible trait variants are rejected

Primary owner:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

### 3. Generic receiver method/operator/trait-call validation

Proven today:

* missing or wrong receiver bounds are rejected
* candidate-bound suggestions are preserved where available
* explicit trait receiver routes must stay naming-consistent

Primary owners:

* [tests_generic_method_bounds.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* companion control-flow and lambda files listed above

### 4. Explicit generic call context

Proven today:

* explicit generic function-call specialization now reports bound failures with
  source-facing context instead of a detached specialization failure
* diagnostics can now name the call site with phrases like `call 'keep'`

Primary owner:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

### 5. Struct-literal generic type-site validation

Proven today:

* unannotated generic struct literals with explicit type arguments now reach
  bound validation
* this works even when the struct literal appears inside larger expression
  shapes

Primary owner:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

### 6. `if` / `match` result-branch propagation

Proven today:

* branch result expressions do not bypass generic constraint checks
* diagnostics preserve branch-local context such as `if-then` and `match-arm`

Primary owners:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)
* [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)
* [tests_generic_method_bounds_nested_match.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_nested_match.rs)

### 7. Lambda-body and higher-order context preservation

Proven today:

* lambda-body local aliases and explicit generic calls do not escape constraint
  validation
* diagnostics now prefer source-facing owner context like `function 'main' body
  lambda body` rather than leaking raw synthesized helper names

Primary owners:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)
* [tests_generic_method_bounds_lambda_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs)

### 8. Diagnostic-layer precedence in deep expected-type chains

Proven today:

* nested expected-type propagation can cross enum payload, struct field, and
  alias layers
* the same owner contrast is now defended on error-style enum payload routes
  like `Result.Ok(...)`, not only `Option.Some(...)`
* when the deep chain stays direct, diagnostics can still land on the inner
  generic call site
* when the deep chain is wrapped by a constrained type alias, the outer alias
  bound may be reported first
* the same contrast now holds in branch-local expected-type routes, not only in
  straight-line locals
* the same contrast now holds in return-position expected-type routes as well

Primary owner:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)

Short rule:

`deep expected-type propagation is now real; diagnostic priority is currently “outer constrained alias first, otherwise inner generic call when available”`

### 9. Receiver-side diagnostic ownership

Proven today:

* method/operator/explicit-trait-call misuse on generic receivers stays owned
  by the receiver-side call surface
* alias chains enrich that owner context through `via type alias ... target`
  prefixes instead of replacing the receiver-side owner entirely
* helper-mediated error/result routes such as `result_map(...)` keep ownership
  in the specialized helper body when that is where the failing receiver-side
  operation actually appears; this is now covered for both `result_map(...)`
  and `result_and_then(...)`

Primary owners:

* [tests_generic_method_bounds.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds.rs)
* [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs)
* [tests_generic_method_bounds_lambda_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs)

Short rule:

`expected-type chains may shift owner between alias and inner call, but receiver-side generic misuse currently remains owned by the method/operator/trait-call layer with alias-target context attached`

## Current Diagnostic Truth

The current mainline is now intentionally defending source-facing context in
generic validation diagnostics.

Current checked phrases include:

* `function 'main' body`
* `call 'keep'`
* `if-then`
* `match-arm`
* `lambda body`

Short rule:

`a generic failure should describe the user-visible source shape that triggered it, not only an internal helper or specialization path`

## Current Gaps Worth Keeping Active

These are the areas that still deserve explicit watchfulness in `0.20.*`:

* deeper type-inference interactions where no explicit generic arguments are
  written and the hard part is expected-type reconstruction through several
  nested expressions
* wider higher-order combinations where generic constraints, callable aliases,
  and capture threading all move together
* source-to-CLI compile-closure parity for the most advanced generic-control
  shapes already proven in frontend-only tests
* future enum/result-style error routes once generic validation starts feeding
  more of the checked error-handling surface

This does not mean the current validator is weak.

It means the current frontend proof is stronger than the full compile-chain
proof in some advanced compositions, so the repo should keep those layers
visibly separate.

## Fast Working Route

When a new generic validation regression appears, use this order:

1. decide whether it is a declaration-shape, use-site, or receiver-side bound
   failure
2. decide whether the interesting shape is plain body, explicit generic call,
   struct literal, `if`, `match`, or lambda body
3. add the test to the narrowest existing owner that already defends that shape
4. only after the frontend proof is clear, decide whether the deeper
   compile-chain route also needs a companion test

Short rule:

`frontend truth first, compile-closure truth second, and do not let one pretend to be the other`
