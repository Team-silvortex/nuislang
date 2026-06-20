# `nuis` `alpha-0.1.*` Mainline Status

This file is the short current-state anchor for the `alpha-0.1.*` line.

It is not a historical closeout file.

It is the shortest answer to one question:

`what should a teammate treat as the real current nuis mainline now that the repository is already past alpha-0.0.1 closeout?`

## Current Line

`alpha-0.1.*` should be read as:

* post-closeout consolidation
* frontdoor/workflow internalization
* native artifact closure visibility
* stronger project/example/doc routing
* continued compiler/runtime surface completion without pretending beta-like stability

Short rule:

`alpha-0.1.*` is where the repository should become easier to read, easier to route, and harder to misstate`

## Current Mainline Spine

The current compile spine should now be read as:

```text
nuis source / project
  -> nuis workflow
  -> project-doctor / project-status / scheduler-view
  -> check
  -> test
  -> build
  -> artifact-doctor
  -> run-artifact
  -> release-check
```

The current compiler/core spine remains:

```text
nuis
  -> nuisc
  -> NIR
  -> YIR
  -> LLVM / AOT packaging
```

## What Is Actually Solid Today

These are the strongest current repository truths:

* project and single-file workflow frontdoors are now explicit
* `workflow`, `project-status`, `project-doctor`, and `artifact-doctor` now expose one readable grouped surface family
* current frontdoor JSON surfaces now explicitly report artifact-readiness and link-plan summary fields
* project-form native artifact closure is checked in through
  [native_artifact_closure_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo)
* frontdoor JSON/output drift is now regression-backed in
  [main.rs](/Users/Shared/chroot/dev/nuislang/tools/nuis/src/main.rs)
* host-symbol-based std/runtime FFI routing is now broader and more legible
* current docs now distinguish current implementation truth from future self-owned linker/launcher architecture more clearly

## What Is Still Not Final

`alpha-0.1.*` should still not claim:

* final self-hosted linker ownership
* final runtime container/launcher architecture
* final frozen public CLI schema
* final `std` structure
* final `nustar` capability split
* beta-level stability guarantees

Short rule:

`current mainline truth is good enough to build on, not good enough to call finished`

## Current First Reading Route

If you are entering the repo fresh today, use this order:

1. [../current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
2. [../reference/nuis-frontdoor-surface-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-frontdoor-surface-reference.md)
3. [../reference/nuis-native-artifact-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-native-artifact-workflow.md)
4. [../reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
5. [../examples-freshness-audit.md](/Users/Shared/chroot/dev/nuislang/docs/examples-freshness-audit.md)

If you need the previous line that led into this one, then drop to:

* [nuis-alpha-0.0.1-preflight-report.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-preflight-report.md)
* [nuis-alpha-0.0.1-closeout-board.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-board.md)
* [nuis-alpha-0.0.1-closeout-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.0.1-closeout-checklist.md)

## Current Practical Rule

When updating current-facing docs, examples, or workflow output:

* prefer `alpha-0.1.*` wording for the present tense
* keep `0.19.*`, `0.20.*`, and `alpha-0.0.1` files as historical transition anchors
* do not let closeout-era wording remain the default frontdoor for current repo orientation

Short rule:

`old line docs are still valuable, but they are no longer the repo homepage in disguise`
