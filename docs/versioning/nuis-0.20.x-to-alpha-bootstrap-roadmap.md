# `0.20.*` To `alpha-0.0.1` Bootstrap Roadmap

This file records the intended transition from the late `0.19.*` cleanup line
into the `0.20.*` base-completion line, then into `alpha-0.0.1`, and finally
toward the pre-`beta` self-hosting gate.

The point is not “add more version numbers”.

The point is to make explicit that the repository is moving from long-running
architecture expansion toward a smaller, stricter question:

`can nuis become a toolchain that increasingly organizes, checks, and builds its own mainline truth?`

## Phase Meaning

Use the phases this way:

* `0.20.*`
  base capability completion, cleanup, and contract tightening
* `alpha-0.0.1`
  first explicit “nuis is now trying to stand as a real product line” marker
* pre-`beta`
  self-hosting pressure line; the main question becomes how much of the
  compiler/tooling story can already be expressed and defended by `nuis`
  itself

## What `0.20.*` Is For

`0.20.*` should not be treated as “expand every frontier again”.

It should be treated as the line where the current foundation is checked for
gaps and tightened until the main route is coherent enough to deserve an
`alpha` label.

The current expectation for `0.20.*` is:

* keep the compile workflow readable end-to-end
* keep ABI wording stable and unambiguous
* reduce repository noise that hides the mainline
* close obvious capability holes in current core language/runtime routes
* make examples, docs, and compiler outputs tell the same story
* keep project-route compilation as the honest default path

In practice that means:

* finish the “current truth” cleanup around examples, docs, build artifacts,
  and workflow entrypoints
* keep tightening control-flow, generics, async, pointer/address, and ABI
  validation where the current line still has narrow or inconsistent behavior
* prefer fewer stronger routes over many half-current ones

## `0.20.*` Success Rule

`0.20.*` is successful when the repository can honestly say:

* there is one clear compile workflow
* there is one clear example tree hierarchy
* current ABI/project/build vocabulary is stable enough to build on
* the main language/runtime/compiler routes no longer drift by folder or by
  older naming habits
* remaining gaps are mostly depth/coverage problems, not “we still do not know
  what the mainline is”

That is the real handoff condition into `alpha-0.0.1`.

## What `alpha-0.0.1` Means

`alpha-0.0.1` should not claim maturity.

It should mark a change in how `nuis` is judged.

Before `alpha`, the repository can still tolerate some amount of “prototype
growth energy”.

After `alpha` begins, the bar changes:

* new work should increasingly justify itself against the self-hosting line
* the toolchain should increasingly be asked to carry its own organization
* examples and workflows should increasingly be judged by whether they help the
  compiler stand on itself
* “interesting capability” stops being enough; “can this become part of the
  self-sustaining mainline?” matters more

So `alpha-0.0.1` means:

`nuis is no longer only accumulating capabilities; it is beginning to organize those capabilities into a toolchain that must eventually help build itself`

## Pre-`beta` Self-Hosting Goal

Before `beta`, the central pressure should be self-hosting.

That does not require a fake all-at-once jump where every compiler subsystem is
already written in `nuis`.

It does require a real directional gate:

* more compiler-side workflow logic should become expressible in `nuis`
* more project/build/test/report organization should become representable
  through `nuis` routes
* more validation and contract truth should be surfaced in language/toolchain
  terms that `nuis` itself can consume

The honest question is not:

`is the compiler 100% self-hosted yet?`

The honest question is:

`is nuis visibly becoming more able to define, validate, and sustain its own mainline?`

## Pre-`beta` Bootstrap Gate

The pre-`beta` bootstrap gate should be read in layers.

### 1. Workflow Self-Description

`nuis` should be able to describe more of its own compile/test/build workflow
using its own project and std-facing vocabulary.

Signals:

* project routes stay first-class
* compile/test/build/release-check organization remains stable
* current workflow hints can be treated as real surface, not temporary prose

### 2. Capability Self-Use

The language should be able to use more of its own real features in examples or
support tooling without immediately falling back to “host-only escape hatch”
thinking.

Signals:

* current generic/control-flow/async/pointer routes are reliable enough to be
  used in more internal-facing code
* project examples increasingly look like pieces of a future self-hosted world,
  not disconnected demos

### 3. Contract Self-Defense

`YIR`, verifier, ABI contracts, and project summaries should increasingly act
as the machine-checkable backbone of the line.

Signals:

* compile outputs stay structurally explainable
* project ABI graph/summary/index vocabulary remains stable
* validator and verifier truth is strong enough to protect the line against
  drift

### 4. Mainline Compression

The number of “equally important” routes should keep shrinking.

Signals:

* current entrypoints remain few and deliberate
* older examples/docs are retired or demoted when they stop carrying real value
* the repository increasingly reads like one intentional toolchain, not a long
  archive of experiments

## What Still Does Not Need To Be True Yet

Before `beta`, these do not all need to be completely finished:

* every trait/generic case
* every async/runtime ambition
* every network/service abstraction
* every std facade family
* a fully independent formal-verification stack

The point is not “everything finished first”.

The point is that the mainline should already be strong enough that finishing
later work happens from a stable toolchain center rather than from ongoing
architectural drift.

## Practical Reading Rule

When evaluating work from now through `alpha` and toward `beta`, ask:

1. Does this strengthen the current mainline?
2. Does this reduce ambiguity or drift?
3. Does this help `nuis` describe/build/check more of itself?
4. If not, is it still important enough to justify delaying the bootstrap line?

If the answer is mostly “no”, the work is probably not core to the pre-`beta`
roadmap.

## Companion Docs

Read this file together with:

* [nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [nuis-0.20.0-abi-compile-vocabulary.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* [nuis-0.19.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-release-checklist.md)
* [nuis-0.17.0-self-hosted-mainline-gate-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-self-hosted-mainline-gate-plan.md)
