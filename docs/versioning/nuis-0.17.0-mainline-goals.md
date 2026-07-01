# `nuis` 0.17.0 Mainline Goals

This file is the short working map for the `0.17.0` line.

It exists to keep the next phase focused on a few linked outcomes instead of
letting the repository drift into many unrelated local improvements.

## Core Goal

`0.17.0` is where `nuis` should start turning validated compile surfaces into
a more complete end-to-end compiler/runtime bridge.

## Three Main Tracks

### 1. Generic Completion

The goal is not “more generic features” in the abstract.

The goal is to make the already-visible generic system behave more coherently
across:

* explicit generic arguments
* expected-type propagation
* nested helper chains
* callable forwarding through nested higher-order helper chains
* alias-aware payload and struct routes
* control-flow-local specialization
* lambda-lifted and higher-order routes
* real multi-file project compilation

Short rule:

`if a generic route feels valid at the source level, the compiler should either support it through the current stack or reject it clearly and locally`

Working anchor:

* [nuis-0.17.0-generic-completion-plan.md](nuis-0.17.0-generic-completion-plan.md)
* [nuis-0.17.0-compile-workflow.md](nuis-0.17.0-compile-workflow.md)
* [nuis-0.17.0-mainline-regression-matrix.md](nuis-0.17.0-mainline-regression-matrix.md)

### 2. Lowering Completion

The goal is to reduce the front/back split.

If frontend validation says a route is real enough to stand on, lowering should
increasingly stop being the place where that route becomes fragile.

This especially applies to:

* control-flow-heavy specialization
* loop-family crossover with branch-local rewrites
* async/task lowering
* project-backed NIR -> YIR -> verifier continuity

Short rule:

`validated frontend truth should more often become checked lowering truth`

### 3. Network/Runtime Bridge

The goal is to keep `std net`, syscall work, http-oriented examples, and
async/task/memory/session groundwork on the same story.

This means:

* more real compile anchors that look like actual transport/session code
* better reuse of task/memory/session building blocks in network layers
* more honest runtime probes where host behavior matters
* less separation between “compiler demo” and “runtime-shaped example”

Short rule:

`compile ladders should keep getting closer to runtime-shaped structure without pretending host/runtime uncertainty has disappeared`

## Working Priorities

When choosing what to do next in `0.17.0`, prefer work that closes gaps across
layers:

* first:
  one bug fix or feature that unlocks multiple existing routes
* second:
  one real project compile anchor that proves the route outside toy snippets
* third:
  documentation and release-gate updates that keep the claim honest

Avoid spending too long on changes that only make one tiny example greener
without strengthening the shared spine.

## Suggested Success Signals

By the time `0.17.0` feels real, these should be easier to say with a straight
face:

* generic-heavy project examples need fewer workaround-shaped local rewrites
* lowering failures are less surprising relative to frontend truth
* async/task/memory/network examples compose more naturally
* the mainline docs can point to a few stronger real-project anchors instead of
  many narrow caveated ones

## Rule Of Thumb

If a change makes the repository easier to explain as one compiler story, it is
probably `0.17.0` work.

If it only adds one more isolated capability without improving the shared path,
it is probably lower priority.
