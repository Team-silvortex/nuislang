# Generic Diagnostic Ownership Contract

This file is the current implementation-facing contract for generic validation
diagnostic ownership in the `nuis` frontend.

It answers one practical question:

`when several generic-validation layers could all plausibly report the same failure, which layer currently owns the user-facing diagnostic?`

Read this together with:

* [../versioning/nuis-0.20.0-generic-validation-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-generic-validation-regression-matrix.md)
* [../versioning/nuis-0.19.0-generic-constraint-validator.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-generic-constraint-validator.md)
* [control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)

## Short Rule

Generic diagnostics are not owned by one monolithic pass.

They are currently owned by the first user-meaningful layer that can honestly
name the failing generic boundary.

In practice, today that means:

* explicit generic call failures prefer the call site
* direct expected-type reconstruction without a constrained alias prefers the
  inner generic call
* constrained aliases prefer the alias boundary once the expected type has been
  reconstructed all the way out to that alias application

Short rule:

`the current frontend prefers the most source-facing constraint boundary it can already prove, not necessarily the innermost implementation step`

## Fast Ownership Table

| Surface | Current owner | Short cue |
| --- | --- | --- |
| explicit generic call | call-site specialization/use-site validation | `call 'keep' generic parameter 'U'` |
| receiver-side method/operator/trait call | receiver-side method-bound validator | `calls method/operator/trait method ... on generic parameter 'T'` |
| direct expected-type chain | inner generic call | `call 'typed_zero' generic parameter 'T'` |
| constrained alias in expected-type chain | outer alias bound | `via type alias 'Alias' generic parameter 'T'` |
| helper-mediated receiver misuse in higher-order/result helpers | specialized helper body | `higher-order specialization body ... calls method ...` |

Shortest rule:

`call surface owns call misuse, alias owns alias misuse, receiver surface owns receiver misuse, and helper bodies own the misuse they themselves introduce`

## Current Ownership Layers

### 1. Explicit generic call ownership

Current owner:

* generic specialization / use-site validation on the call itself

Current shape:

* `function 'main' body call 'keep' generic parameter 'U'`
* branch-local variants such as `if-then call 'keep'`
* lambda-body variants such as `lambda body call 'keep'`

Use this mental model when:

* the user wrote explicit generic arguments
* or inference from direct parameter positions reaches a concrete generic call
  boundary without needing a constrained alias to explain the failure first

### 1b. Receiver-side method/operator/trait-call ownership

Current owner:

* the receiver-side generic method-bound validator

Current shape:

* direct receiver:
  `function 'bump' body calls method 'add' on generic parameter 'T' ...`
* operator receiver:
  `function 'same' body calls operator '==' on generic parameter 'T' ...`
* explicit trait receiver:
  `function 'bump' body calls trait method 'Addable.add' on generic parameter 'T' ...`
* alias-wrapped receiver:
  the same receiver-side diagnostic, but prefixed with alias-target context such
  as `via type alias 'Alias' target via type alias 'Outer' target`
* helper-mediated error/result route:
  higher-order helper bodies such as `result_map(...)` and
  `result_and_then(...)` can own the receiver-side misuse when the failing
  receiver operation lives inside the specialized helper body itself

This layer does not currently compete with expected-type ownership in the same
way zero-arg generic-call inference does.

Instead, its current rule is:

* receiver-side diagnostics stay owned by the method/operator/trait-call use
* alias chains enrich the receiver context when the receiver type was
  reconstructed through alias targets
* higher-order error/result helpers keep ownership when the failing
  method/operator/trait-call is introduced by the helper body rather than by the
  caller's outer expression

Short rule:

`receiver-side generic misuse stays owned by the receiver call surface; alias chains currently decorate that ownership rather than replacing it`

### 2. Direct expected-type ownership

Current owner:

* inner generic call reached through expected-type propagation

Current shape:

* local annotation drives the call:
  `function 'main' body local 'value' call 'typed_zero' generic parameter 'T'`
* branch-local expected type drives the call:
  `function 'main' body local 'value' if-then call 'typed_zero' generic parameter 'U'`
* return-position expected type drives the call:
  `function 'build' body call 'typed_zero' generic parameter 'U'`

This applies when:

* expected-type propagation crosses struct fields
* expected-type propagation crosses enum payload constructors
* expected-type propagation crosses error-style enum payload constructors such
  as `Result.Ok(...)`
* no constrained type alias becomes the clearer outer ownership boundary first

### 3. Constrained alias ownership

Current owner:

* alias generic-parameter bound validation

Current shape:

* local annotation:
  `function 'main' body local 'value' via type alias 'Alias' generic parameter 'T'`
* deep branch-local route:
  same local-owner context, still ending in `via type alias ...`
* return-position route:
  `function 'build' return type via type alias 'Alias' generic parameter 'T'`

This applies when:

* expected-type propagation successfully reconstructs a concrete alias
  application such as `Alias<Text>`
* the alias itself carries the relevant bound
* reporting the alias boundary is more direct than pretending the failure only
  belongs to the nested generic call

Short rule:

`once the frontend can honestly say “this alias application itself violates its bound”, that alias currently owns the diagnostic`

## Current Entry-Point Consistency

This ownership rule is now regression-backed across three expected-type entry
routes:

* local annotations
* branch-local expected-type propagation
* return-position expected-type propagation

Current truth:

* no alias:
  inner call remains the owner when expected-type propagation reaches the call
* constrained alias present:
  outer alias becomes the owner once the type has been reconstructed to that
  alias application

This is intentional current behavior, not just a side-effect of one test.

## Why Alias Can Win

The frontend currently validates explicit type annotations and return types as
their own generic-constraint surfaces after walking the value expression.

Separately, constrained aliases validate alias parameter bounds before expanding
their target shape.

That combination is why a deep chain like:

* `Option<Boxed<Alias<Text>>>`
* `Result<Boxed<Alias<Text>>, CoreError>`

can produce:

* `... via type alias 'Alias' generic parameter 'T'`

instead of always reporting only:

* `... call 'typed_zero' generic parameter 'U'`

This is not claiming the inner call is unimportant.

It is claiming that, once the outer constrained alias is concrete and visibly
violates its own declared bound, the alias is currently the cleaner ownership
boundary.

## Current Non-Goals

This contract does not yet promise:

* source span precision
* ranking between several simultaneously valid inner call candidates
* fully uniform wording across every future trait/generic/GLM validation layer
* CLI/source-compile parity beyond the frontend paths already covered by the
  current tests

It only promises the current ownership rule in the checked frontend behavior.

## Maintenance Rule

When changing generic diagnostics, do not ask only:

* `does it still fail?`

Also ask:

* `which layer now owns the failure?`
* `is that ownership more source-facing or less source-facing than before?`
* `does it still match the local / branch / return route contract?`

If ownership changes intentionally:

* update the regression tests
* update the `0.20.0` regression matrix
* update this contract if the rule itself changed
