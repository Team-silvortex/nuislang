# `nuis` Binary-Compile Maturity Spine For `0.16.0`

This file is the practical companion to
[nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md).

The workflow file answers:

* what commands should users run

This file answers:

* what those commands must be able to trust
* which compiler layers are already strong enough to stand on
* which remaining gaps still block a genuinely mature binary compile story

The goal for `0.16.0` is not “every feature is finished”.

The goal is narrower and more important:

* a `nuis` source or project can move through one coherent compile spine
* that spine has one visible validation path
* the final artifact contract is explicit enough to call release-ready

## Target Definition

For `0.16.0`, “mature binary compile” should mean:

```text
source/project
  -> frontend normalization
  -> project contract assembly
  -> YIR lowering and verification
  -> LLVM / AOT emission
  -> build manifest verification
  -> release-ready output directory
```

In repo terms:

* one front door: `nuis`
* one canonical project route:
  `project-doctor -> check -> test -> build -> release-check`
* one debug route:
  `dump-ast -> dump-nir -> dump-yir -> scheduler-view`
* one final release gate:
  `release-check`

## The Compile Spine

### 1. Input And Project Intake

This stage answers:

* is the input a single `.ns` file or a project
* if it is a project, is the shape healthy enough to compile
* what entry, domains, exchanges, ABI mode, and output intents are in play

Current stable surfaces:

* `project-doctor`
* `project-status`
* `project-lock-abi`
* compiler-side `ProjectCompilationPlan`

Current maturity level:

* strong enough to treat as the canonical project front door

What must stay true:

* `check`, `test`, `build`, `release-check`, `project-status`, and
  `project-doctor` must continue to derive their project view from the same
  normalized plan, not from duplicated ad hoc recomputation

### 2. Frontend Normalization

This stage answers:

* can the parser and frontend normalize surface syntax into a stable `NIR`
* do generic constraints, pattern bindings, callable families, and type
  expectations fail early enough and clearly enough

Current stable surfaces:

* generic callable families through `Fn1`, `Fn2`, `Fn3`
* alias-aware generic constraint validation
* generic method-bound diagnostics through nested alias chains and control-flow
  scopes
* lambda / higher-order helper context restoration for generic diagnostics
* call-inferred local / receiver method-bound validation
* call-root destructure binding validation
* struct destructuring and structured `match` bindings
* generic struct MVP
* generic payload-style constructor/pattern sugar under expected type

Current maturity level:

* strong enough for real project examples and binary-bound demos

Still intentionally narrow:

* generic payload constructors still rely on expected type
* pattern system is practical, not yet a full ADT-pattern language

### 3. Project Contract Assembly

This stage answers:

* what packet/contracts, support surfaces, ABI surfaces, and host-FFI surfaces
  the project actually requires
* whether those requirements match domain and profile expectations

Current stable surfaces:

* manifest parsing
* ABI recommendation and locking
* packet schema/contract indexing
* shader/kernel/data validation families
* bridge/stage contract materialization
* project organization / exchange / plan metadata output

Current maturity level:

* strong enough to treat the project layer as real compiler truth, not just a
  convenience wrapper

What must stay true:

* the emitted plan, organization, exchange, ABI, packet, and host-FFI metadata
  must continue to come from one shared project model

### 4. YIR Lowering And Verification

This stage answers:

* can normalized `NIR` lower into a stable executable `YIR` graph
* are scheduler contracts, loop forms, async/result families, and ownership
  edges explicit enough to verify

Current stable surfaces:

* direct-call recursion substrate
* self / mutual recursion and reachable helper closures
* higher-order recursion closure support
* loop-family lowering and scheduler contract emission
* statement-level effect sequencing
* task join / join-result consume boundaries
* GLM use-mode verification for `borrow`, `borrow_end`, `move`, `free`

Current maturity level:

* strong enough to call `YIR` the main semantic execution boundary today

Still not fully mature:

* async ownership rules beyond `join(...)` and `join_result(...)`
* especially `cancel(...)` and `timeout(...)`

That is the biggest remaining semantic gap if the goal is “mature binary
compile”, not just “successful binary compile”.

### 5. LLVM / AOT Emission

This stage answers:

* can verified `YIR` produce a concrete host-targeted output bundle
* can the output bundle state what was built clearly enough to verify later

Current stable surfaces:

* `build`
* `verify-build-manifest`
* `release-check`
* LLVM IR emission and binary build directory output
* per-domain ABI target metadata in build-manifest verification

Current maturity level:

* strong enough to serve as the release-facing artifact route

What must stay true:

* `release-check` remains the final gate
* `verify-build-manifest` remains the explicit artifact-contract verifier
* the build directory stays concrete and inspectable rather than magical

### 6. User-Facing Output Contract

This stage answers:

* if someone receives an output directory, can they tell what was built
* if a build fails, is there one clear route to inspect and debug it

Current stable surfaces:

* output directory with `nuis.build.manifest.toml`
* project-side plan / organization / exchange / packet indexes
* `dump-ast`, `dump-nir`, `dump-yir`, `scheduler-view`
* `project-status` / `project-doctor` compile-workflow hints

Current maturity level:

* close to mature enough for a `0.16.0` release story

## What Is Already Mature Enough To Lean On

These are the slices that already look like “real toolchain truth”, not probe
material:

* project normalization through a shared compilation plan
* generic callable and higher-order specialization through `Fn3`
* recursion closure lowering through direct-call substrate
* generic constraint and method-bound diagnostics
* practical struct destructuring and structured `match`
* packet/schema and project contract indexing
* scheduler/lane/result/observer lowering families
* build-manifest verification
* `release-check` as one canonical final gate

## Remaining `0.16.0` Maturity Gates

If we want to call the binary compile story genuinely mature, these are the
highest-signal remaining gates.

### Gate 1. Finish Async Ownership Tightening

Must close:

* `cancel(...)` ownership semantics
* `timeout(...)` ownership semantics
* any remaining task-handle double-consume or stale-handle verifier gaps

Why this matters:

* without it, async project builds can still succeed under an ownership model
  that is only partially finalized

### Gate 2. Keep The Release Route Singular

Must preserve:

* `release-check` as the one final gate
* `build` as artifact emission
* `verify-build-manifest` as artifact-contract inspection

Why this matters:

* if users need multiple competing “final” routes, the binary story is not yet
  mature no matter how many language features work

### Gate 3. Keep Project Truth Unified

Must preserve:

* one project plan
* one ABI story
* one organization/exchange model
* one packet/support/host-FFI metadata route

Why this matters:

* binary compile maturity depends on the project layer being compiler truth,
  not a shell around the compiler

### Gate 4. Keep Debugging Linear

Must preserve:

* `dump-ast -> dump-nir -> dump-yir -> scheduler-view`

Why this matters:

* if compile failures require “tribal” debugging paths, the toolchain is still
  not mature enough to teach cleanly

## Current Call

Today the repository is already beyond “toy compiler” territory.

The strongest honest summary is:

* frontend normalization is real
* project contract assembly is real
* `YIR` lowering/verification is real
* AOT output and build-manifest verification are real
* the remaining biggest gap is not breadth of language features
* the remaining biggest gap is finishing async ownership so the whole binary
  path has one consistent semantic contract

## `0.16.0` Ship Rule

We should call the `0.16.0` binary compile story ready when all of the
following feel true at once:

1. `project-doctor -> check -> test -> build -> release-check` is the
   shortest honest answer for normal project work
2. `dump-ast -> dump-nir -> dump-yir -> scheduler-view` is the shortest honest
   answer for compile debugging
3. async ownership no longer has obvious unfinished boundary semantics in the
   normal task route
4. build output directories and manifests are concrete enough to inspect
   without guessing
5. docs, help text, project hints, and JSON outputs all tell the same story

If one of those fails, the work is not “done later”; it is still part of the
binary-compile maturity track.
