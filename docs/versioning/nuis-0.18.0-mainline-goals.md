# `nuis` 0.18.0 Mainline Goals

This file is the short working map for the `0.18.0` line.

It exists to keep the next phase focused on one shared maturity question:

`can nuis finally describe control flow as a coherent compiler mainline instead of a collection of supported subsets?`

## Core Goal

`0.18.0` is where `nuis` should turn its already-real control-flow islands into
a much clearer end-to-end control-flow story across:

* frontend expression lowering
* branch-local typing and specialization
* loop-family lowering
* async/task crossover
* real multi-file project compilation

Short rule:

`if a control-flow route feels ordinary at the source level, the compiler should increasingly either carry it through the stack or reject it locally and honestly`

## Three Main Tracks

### 1. Control-Flow Completion

The goal is not “more if/match/while tests” in the abstract.

The goal is to make the current control-flow surface behave more coherently
across:

* `if` expressions in non-trivial value positions
* `match` expressions and `match` lowering inside loops
* branch-local reconstruction across helper/generic routes
* `break` / `continue` interaction with loop carries
* `if` / `match` / `while` crossover with async/task shapes
* project-backed compile anchors that look like real programs

Working anchor:

* [nuis-0.18.0-control-flow-completion-plan.md](nuis-0.18.0-control-flow-completion-plan.md)

First checked-in `0.18.0` examples already moving this from plan to reality:

* state control-flow anchors for:
  `match_branching_while_demo`,
  `match_expr_branching_while_demo`,
  `bool_match_branching_while_demo`
* task/control-flow anchors for:
  `task_result_policy_branch_demo`,
  `task_lifecycle_branch_demo`

### 2. Lowering Truth Tightening

The goal is to reduce the remaining gap between “frontend says this route is
valid” and “lowering only supports a narrower internal shape”.

This especially applies to:

* counted/carry loop routes
* flow/post-flow loop routes
* lowered `match` conditions feeding loop predicates
* branch-local carry updates coming from helper-expanded bodies
* async control-flow routes that should survive into project-backed lowering

Short rule:

`frontend control-flow truth should increasingly become checked lowering truth`

### 3. Release-Grade Workflow Honesty

The goal is to make `0.18.0` easier to explain as one compiler workflow.

This means:

* fewer claims that only hold in toy snippets
* more project-backed anchors for real control-flow-heavy routes
* a clearer regression matrix for control-flow failures
* documentation that names the current supported loop/control families directly

Short rule:

`control-flow maturity is only real when it is test-backed, project-backed, and easy to point to`

## Working Priorities

When choosing what to do next in `0.18.0`, prefer work in this order:

* first:
  remove one control-flow mismatch that currently appears in multiple layers
* second:
  promote one real project example into a stronger control-flow anchor
* third:
  update release-facing docs so the claim stays honest

Avoid spending too long on control-flow work that only makes one tiny pattern
greener without strengthening the shared story.

## Suggested Success Signals

By the time `0.18.0` feels real, these should be easier to say with a straight
face:

* control flow no longer feels split between “frontend supports it” and
  “lowering only likes one exact shape”
* `if`, `match`, and `while` read more like one family than separate special
  cases
* loop carry and branch-local rewrites survive real project examples more often
* async/task/control-flow crossover stops feeling like an exception path
* mainline docs can point to a small set of strong control-flow anchors instead
  of many caveated subsets

The line has already started in the right direction:

* `match + loop + carry` now has project-backed state anchors
* `async/task + control flow` now has first project-backed task anchors

## Rule Of Thumb

If a change makes `nuis` easier to describe as one control-flow compiler story,
it is probably `0.18.0` work.

If it only adds one more isolated green case without improving the shared path,
it is probably lower priority.
