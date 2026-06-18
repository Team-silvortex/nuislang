# `nuis` `alpha-0.0.1` Closeout Checklist

This file is the practical closeout checklist for the line between late
`0.20.*` and `alpha-0.0.1`.

It is not a broad roadmap.

It is the compression file for one narrower question:

`what must be true before nuis can honestly call the next phase alpha?`

Read this together with:

* [nuis-0.20.x-to-alpha-bootstrap-roadmap.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.x-to-alpha-bootstrap-roadmap.md)
* [alpha-mainline-boundary-index.md](/Users/Shared/chroot/dev/nuislang/docs/reference/alpha-mainline-boundary-index.md)
* [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
* [nuis-0.20.0-frontend-cli-boundaries.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-frontend-cli-boundaries.md)

## Short Rule

Before `alpha-0.0.1`, the repository does not need every planned feature.

It does need one defendable toolchain center.

The core test is:

`can nuis explain, validate, compile, and regression-defend its own current mainline without leaning on too many historical exceptions?`

## Exit Condition For `alpha-0.0.1`

`alpha-0.0.1` is ready when all of these are true together:

* one clear compile workflow is the default reading route
* one clear repository mainline is visible across docs, examples, CLI, and
  tests
* the core frontend/lowering/control-flow/async/generic routes are coherent
  enough to serve as the toolchain center
* remaining gaps are explicit boundary/depth items, not confusion about what
  the mainline even is

## 1. Must Complete Before `alpha`

These are the hard gate items.

### 1.1 Compile Workflow Must Be Singular

The default route must stay readable as:

`nuis source/project -> nuis frontdoor -> nuisc -> NIR -> YIR -> LLVM/AOT`

Must be true:

* [README.md](/Users/Shared/chroot/dev/nuislang/README.md) keeps one obvious
  frontdoor workflow
* [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
  remains the canonical short router
* `project-doctor -> check -> test -> build -> release-check` remains the
  honest default CLI path
* the difference between frontend truth and compile-closure truth stays
  explicit, not blurred

Done when:

* a new reader can find the shortest current route without reconstructing it
  from memory

### 1.2 Mainline Boundary Docs Must Match Reality

The repository should not say “this is current” in one place and “that is
current” in another.

Must be true:

* [alpha-mainline-boundary-index.md](/Users/Shared/chroot/dev/nuislang/docs/reference/alpha-mainline-boundary-index.md)
  points to the right live boundaries
* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
  reflects active pipeline gaps only
* [nuis-0.20.0-frontend-cli-boundaries.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-frontend-cli-boundaries.md)
  reflects active frontend-vs-CLI boundaries only
* outdated “current” wording in older docs is removed, demoted, or clearly
  marked historical

Done when:

* the active docs tell one story about the current line

### 1.3 Core Semantic Spine Must Be Regression-Defended

Before `alpha`, the core language center must be defended by checked-in tests,
not only by prose confidence.

Must be true for the mainline semantic spine:

* generics + constraint validation
* control flow: `if`, `match`, `while`, recursion
* lambda / `Fn1` / `Fn2` / `Fn3`
* `?` / `await` / `Task<T>` composition
* pointer/address surface plus owner/borrow rules
* trait-bound method/operator routes that are already called current

Current high-signal anchors already include:

* [tests_generic_constraints.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_constraints.rs)
* [tests_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_control_flow.rs)
* [tests_lambda_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_lambda_higher_order.rs)
* [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
* [tests_try.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_try.rs)
* [tests/memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* [tests/network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

Done when:

* the features described as current already have positive regression anchors
  and the remaining negative boundaries are intentional

### 1.4 Frontend / Lowering / Compile-Closure Boundaries Must Be Honest

The repository can tolerate boundaries before `alpha`.

It cannot tolerate hidden ones.

Must be true:

* frontend-only truth is clearly labeled as frontend-only
* compile-closure truth is clearly labeled as compile-closure truth
* known branch-local runtime limits remain documented as policy, not as vague
  surprise
* the promoted compile-closure examples still survive real pipeline routes

Done when:

* failures are classifiable as `frontend gap`, `pipeline gap`, or `intentional
  contract`, instead of “something somewhere is inconsistent”

### 1.5 `nustar` Capability Split Must Be Good Enough To Stand On

`alpha` does not require the final package universe.

It does require that the repository can explain which truths belong in compiler
core and which increasingly belong in registered capability contracts.

Must be true:

* current `nustar` package set is still the visible capability center:
  `cpu`, `data`, `shader`, `kernel`, `network`
* package registration truth is not contradicted by scattered compiler-only
  assumptions
* `nuisc` remains able to load and explain package-level lowering/ABI truth
* current docs for capability split and ABI grain remain aligned with checked-in
  manifests under [nustar-packages](/Users/Shared/chroot/dev/nuislang/nustar-packages)

Done when:

* the active domain story reads as registered capability truth, not just
  compiler folklore

### 1.6 Example Tree Must Match The Mainline

Examples are part of the toolchain story now.

Must be true:

* `examples/projects` remains the primary runnable/project frontdoor
* `examples/ns` remains the primary single-file semantic frontdoor
* stale or misleading examples are retired, demoted, or marked legacy
* positive and negative anchors exist for the strongest current boundaries

Done when:

* the shortest example path teaches the same mainline that the docs and tests
  defend

## 2. Should Complete Before `alpha`

These are not as absolute as the hard gates, but the line will be noticeably
weaker if they are skipped.

### 2.1 Release Vocabulary Should Be Normalized

Should be true:

* artifact / manifest / report / verify wording stays stable
* CLI output names and doc wording do not drift by subsystem
* project/build/release naming remains grouped and deliberate

### 2.2 Standard Library Frontdoor Should Read As One Ladder

Should be true:

* `stdlib/std` reads as layered runtime recipes instead of disconnected files
* current workflow / command / filesystem / host / task / network entry docs
  still reflect what can actually compile today
* domain-facing `std` examples continue to agree with project-route examples

### 2.3 Historical Noise Should Be Reduced

Should be true:

* active docs do not compete with obviously older “current” documents
* obsolete examples are either removed or moved under legacy/historical paths
* release-era snapshots remain available without pretending to be current

### 2.4 Project-Route Self-Use Should Be Stronger

Should be true:

* more checked-in project examples use the real `nuis` frontdoor path
* compile/test/build/release-check examples remain readable enough to support
  future self-hosting pressure

## 3. Can Defer Until After `alpha`

These matter, but they do not need to block `alpha-0.0.1` if the mainline is
already coherent.

### 3.1 Full Trait / Generic Exhaustiveness

Can defer:

* every trait corner case
* every operator/receiver/generic interaction
* every generic diagnostic polish case

Requirement before deferring:

* the supported subset is explicit and regression-defended

### 3.2 Full Async Runtime Maturity

Can defer:

* final thread/mutex runtime completeness
* parameterized async entry if the current restriction remains explicit
* deeper scheduler/runtime ambitions beyond the current compile contract

Requirement before deferring:

* staged boundary docs remain accurate

### 3.3 Full Network / Shader / Kernel Richness

Can defer:

* broader domain facades
* richer service/client/server ergonomics
* larger heterogeneous runtime orchestration

Requirement before deferring:

* current `nustar` contracts and compile surfaces are already coherent

### 3.4 Full Self-Hosting

Can defer:

* writing major compiler subsystems in `nuis`
* a complete `yalivia` or `vulpoya` stack
* all workflow/build logic moving into `nuis`

Requirement before deferring:

* the line is visibly moving toward self-description and self-defense

## 4. Practical Closeout Pass

Use this pass order before declaring `alpha-0.0.1` ready:

1. docs compression
   - one active mainline route
   - one active boundary index
   - one active closeout checklist
2. example compression
   - current frontdoor examples
   - current invalid boundary anchors
   - legacy examples demoted
3. compiler regression compression
   - semantic spine tests
   - compile-closure tests
   - project-route tests
4. CLI/workflow compression
   - `workflow`
   - `project-doctor`
   - `check`
   - `test`
   - `build`
   - `release-check`
5. release statement compression
   - what is current
   - what is intentionally bounded
   - what is intentionally deferred

## 5. Final Alpha Question

Before cutting `alpha-0.0.1`, ask:

1. Can the repo explain its current mainline in a small number of files?
2. Can the toolchain defend that mainline with real tests?
3. Can the CLI, docs, examples, and compiler outputs all point at the same
   route?
4. Are the remaining gaps mostly explicit depth items rather than unresolved
   identity drift?

If the answer is “yes” to those four, the repository is probably ready for
`alpha-0.0.1`.
