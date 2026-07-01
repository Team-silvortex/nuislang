# `nuis` 0.19.0 Mainline Goals

This file is the short working map for the `0.19.0` line.

It exists to keep the next phase focused on one shared maturity question:

`can nuis turn its now-believable mainline into a more clearly internalized workflow instead of letting the story drift across docs, examples, gates, and source style?`

## Core Goal

`0.19.0` is where `nuis` should turn its current mainline from “proved in many
places” into “described and maintained as one current route” across:

* compile frontdoor commands
* source-style conventions
* project-backed regression anchors
* implementation-facing documentation
* versioned current-vs-historical reading order

Short rule:

`if a route is current mainline truth, a teammate should be able to find the right command, example family, test gate, and doc anchor without reconstructing the story by hand`

## Three Main Tracks

### 1. Workflow Internalization

The goal is not only “more commands exist”.

The goal is to make the current compile route easier to teach and repeat:

* `status/help` as the frontdoor orientation pair
* `workflow` as route classification
* `project-doctor/project-status/scheduler-view` as grouped preflight detail
* `check/test/build/release-check` as the action spine

Short rule:

`0.19.0` should make the compile frontdoor feel like one system, not one list`

### 2. Source-Style And Semantic Boundary Honesty

The goal is to reduce confusion between:

* preferred `.ns` source spelling
* lowered builtin memory/address vocabulary
* verifier/NIR/YIR truth

That means:

* checked-in source examples and `std` should keep one address style
* source-facing docs should point to that style directly
* implementation-facing docs should explain why builtin names still appear

Short rule:

`source syntax and lowered truth should both stay visible, but they should stop pretending to live at the same layer`

### 3. Regression Gate Consolidation

The goal is to make the current gate story easier to trust and rename cleanly:

* current checked-in `0.18` gate scripts should remain documented honestly
* current `0.19` docs should say what those gates actually defend
* current project-backed anchors should stay the primary evidence:
  `state`, `task`, `memory`, `shader`, `network`
* frontend regression ownership should be easier to place without reading the
  whole test tree by hand:
  [versioning/nuis-0.19.0-frontend-test-map.md](nuis-0.19.0-frontend-test-map.md)
* frontend capability combinations should be easier to restate without
  reconstructing them from scattered tests:
  [versioning/nuis-0.19.0-frontend-capability-matrix.md](nuis-0.19.0-frontend-capability-matrix.md)
* workflow/frontdoor/mainline compile combinations should be easier to restate
  without reconstructing them from separate command docs:
  [versioning/nuis-0.19.0-workflow-capability-matrix.md](nuis-0.19.0-workflow-capability-matrix.md)

Short rule:

`a current version line should not depend on folklore to explain which tests matter`

## Working Priorities

When choosing what to do next in `0.19.0`, prefer work in this order:

* first:
  remove one current/historical ambiguity from docs or workflow
* second:
  strengthen one project-backed anchor that represents a whole mainline slice
* third:
  rename or consolidate a gate/script only when the surrounding story is
  already clear

Avoid spending too long on changes that make one file read better but do not
reduce cross-layer ambiguity.

## Suggested Success Signals

By the time `0.19.0` feels real, these should be easier to say with a straight
face:

* the current version line is obvious from the main docs
* the compile workflow is easy to restate in one short ordered route
* `.ns` source style for the address surface no longer drifts across examples
  and `std`
* implementation-facing docs no longer look like accidental source-style
  recommendations
* current regression gates are easier to point to and easier to justify

## Rule Of Thumb

If a change makes the current mainline easier to read, teach, and keep honest,
it is probably `0.19.0` work.
