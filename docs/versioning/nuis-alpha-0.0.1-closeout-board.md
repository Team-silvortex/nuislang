# `nuis` `alpha-0.0.1` Closeout Board

This file is the execution board companion to
[nuis-alpha-0.0.1-closeout-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-checklist.md).

It should now be read as a predecessor closeout board, not as the default
current-line frontdoor.

If you want the current line first, use:

* [nuis-alpha-0.1-mainline-status.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.1-mainline-status.md)

The checklist answers:

`what must be true before alpha?`

This board answers:

`what exactly are we still closing, what evidence counts, and when is each lane done?`

Use this file as the practical repo-level board for late `0.20.*` work.

## Board Rule

Every lane should stay classifiable as one of:

* `done`
* `active`
* `boundary`
* `defer-after-alpha`

Practical meaning:

* `done`: current line is coherent and defended
* `active`: still being closed before `alpha`
* `boundary`: intentionally limited, but the limit is documented honestly
* `defer-after-alpha`: not a blocker if the supported subset is already clear

## Exit Rule

`alpha-0.0.1` is ready when every `must-close` lane below is either:

* `done`, or
* `boundary` with explicit documentation, regression evidence, and no false
  "already complete" wording elsewhere

## 1. Toolchain Center

### 1.1 Singular Compile Workflow

* priority: `must-close`
* status: `active`
* target:
  `nuis source/project -> nuis frontdoor -> nuisc -> NIR -> YIR -> LLVM/AOT`
* key evidence:
  [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
  [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
  [nuis-0.20.0-abi-compile-vocabulary.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* close when:
  one default reader path exists and does not compete with older routes
* remaining pressure:
  keep frontend truth and compile-closure truth separated without splitting the
  user-facing story again

### 1.2 Frontend vs Compile-Closure Honesty

* priority: `must-close`
* status: `active`
* key evidence:
  [nuis-0.20.0-frontend-cli-boundaries.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-frontend-cli-boundaries.md)
  [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
  [alpha-mainline-boundary-index.md](/Users/Shared/chroot/dev/nuislang/docs/reference/alpha-mainline-boundary-index.md)
* close when:
  every known failure can be named as `frontend gap`, `pipeline gap`, or
  `intentional boundary`
* remaining pressure:
  avoid silent promotion of frontend-only success into full compile claims

## 2. Semantic Spine

### 2.1 Generics and Constraint Validation

* priority: `must-close`
* status: `active`
* key evidence:
  [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)
  [nuis-0.20.0-generic-validation-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-generic-validation-regression-matrix.md)
  [generic-diagnostic-ownership-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/generic-diagnostic-ownership-contract.md)
* close when:
  the currently claimed generic subset is positive-test-backed across explicit
  calls, receivers, literals, control-flow bodies, and lifted helpers
* current note:
  this lane is materially stronger now, but it still needs continued gap review
  instead of assuming exhaustiveness

### 2.2 Control Flow and Recursion

* priority: `must-close`
* status: `active`
* key evidence:
  [tests_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_control_flow.rs)
  [control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)
  [nuis-0.20.0-branch-runtime-lowering-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-branch-runtime-lowering-matrix.md)
* close when:
  `if`, `match`, `while`, break/continue shaping, and recursive routes all read
  as one defended lowering story
* remaining pressure:
  preserve clarity around post-flow/shared-suffix rewrites

### 2.3 Lambda and Higher-Order Routes

* priority: `must-close`
* status: `active`
* key evidence:
  [tests_lambda_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_lambda_higher_order.rs)
  [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
* close when:
  current `Fn1` / `Fn2` / `Fn3` routes, helper lifting, and expected-type-based
  inference survive ordinary control-flow, `?`, and `await` contexts
* current note:
  this lane is now much closer to "real mainline" than "special demo path"

### 2.4 Async, `Task<T>`, and `?`

* priority: `must-close`
* status: `active`
* key evidence:
  [tests_try.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_try.rs)
  [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
  [cpu-thread-lock-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-thread-lock-boundary.md)
* close when:
  the supported async/result composition route is explicit and regression-backed
  through nested control-flow and lifted helper positions
* remaining pressure:
  thread/lock maturity can still be staged, but the contract surface must stay
  honest

### 2.5 Address, Borrow, Pointer Surface

* priority: `must-close`
* status: `active`
* key evidence:
  [nuis-0.19.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-address-pointer-mainline.md)
  [address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)
  [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
  [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* close when:
  the currently supported owned/borrowed pointer story compiles, verifies, and
  is described with one surface contract
* remaining pressure:
  future host ABI expansion must not blur the current internal-vs-extern rule

## 3. Capability Split

### 3.1 `nustar` Package Boundary

* priority: `must-close`
* status: `active`
* key evidence:
  [nustar-capability-split-boundary.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-capability-split-boundary.md)
  [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)
  [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* close when:
  `cpu`, `data`, `shader`, `kernel`, and `network` remain explainable as
  registered capability truth instead of compiler folklore
* remaining pressure:
  shader/kernel still need strengthening, but the contract split itself should
  be stable before alpha

### 3.2 ABI and Lowering Grain

* priority: `must-close`
* status: `active`
* key evidence:
  [nuis-0.20.0-abi-compile-vocabulary.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
  [nustar-abi-grain-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-abi-grain-sketch.md)
* close when:
  the active vocabulary around package registration, lowering, ABI grain, and
  concrete artifact production no longer drifts by subsystem
* remaining pressure:
  later format evolution can wait, but alpha needs one stable wording set

## 4. Repo Surface

### 4.1 Example Tree Freshness

* priority: `must-close`
* status: `active`
* key evidence:
  [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
  [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
  [examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)
* close when:
  the shortest examples teach the same mainline the docs and tests defend
* remaining pressure:
  old examples can stay only if clearly demoted or still genuinely current

### 4.2 Standard Library Frontdoor

* priority: `should-close`
* status: `active`
* key evidence:
  [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
  [nuis-0.20.0-std-refactor-frontdoor.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
  [std-tooling-workflow-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-tooling-workflow-contract.md)
* close when:
  `std` reads as layered frontdoors instead of a flat bucket of recipes
* remaining pressure:
  networking and runtime-facing ladders should keep moving toward one coherent
  domain story

### 4.3 Historical Demotion

* priority: `should-close`
* status: `active`
* key evidence:
  [versioning/README.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/README.md)
  [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* close when:
  older "current" documents no longer compete with the live route
* remaining pressure:
  keep history accessible without letting it retake the frontdoor

## 5. Can Defer After `alpha`

### 5.1 Exhaustive Trait / Generic Corner Cases

* priority: `defer-after-alpha`
* status: `boundary`
* defer rule:
  allowed if the supported subset is explicitly documented and already
  regression-defended

### 5.2 Full Async Runtime Maturity

* priority: `defer-after-alpha`
* status: `boundary`
* defer rule:
  allowed if the staged `Task<T>` / thread / lock story remains honest and does
  not masquerade as finished runtime completeness

### 5.3 Final Self-Hosting Pressure

* priority: `defer-after-alpha`
* status: `boundary`
* defer rule:
  allowed if project-route self-use keeps strengthening and the current route is
  already stable enough to serve as the future bootstrap center

## 6. Practical Closeout Pass

When running a late-`0.20.*` closeout sweep, use this order:

1. fix wording drift in mainline docs before adding new claims
2. retire or demote stale examples before using them as evidence
3. add or refresh regression anchors for any feature promoted to "current"
4. classify every remaining failure as frontend, pipeline, or intentional
   boundary
5. only then promote the route as alpha-ready

Short rule:

`alpha is not feature-complete; alpha is route-coherent`
