# `nuis` `0.20.0` Branch Runtime Lowering Matrix

This file is the short current-state matrix for the branch-local runtime
effect rewrites now defended in `if` / `match` lowering.

It describes the narrow but now well-tested shape:

`branch-local runtime effect -> selected/shared value -> observer suffix -> pure suffix`

Use this together with:

* [nuis-0.20.0-compile-gap-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-compile-gap-checklist.md)
* [../reference/control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)
* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)

## Short Rule

For the current `0.20.*` line, a branch-local runtime shape is now considered
materially covered when the repo has checked-in proofs for:

* direct runtime-effect selection
* alias-chain selection of the same runtime effect
* shared observer use after the selected effect result
* pure arithmetic/value suffix after the shared observer result

This is not yet the same thing as “general arbitrary effectful branch lowering.”

## Covered Runtime Families

### Unary runtime consumers

Covered:

* `join(...)`
* `join_result(...)`
* `thread_join(...)`
* `thread_join_result(...)`
* `cancel(...)`
* `mutex_lock(...)`

Current route:

* same-op branch pairs can be rewritten through one selected/shared input and
  one surviving runtime effect node
* for task/thread joins and `cancel`, the selected input may itself come from a
  selected `spawn(...)` / `thread_spawn(...)` rewrite

### Call-like runtime producers

Covered:

* `spawn(helper(...))`
* `thread_spawn(helper(...))`

Current route:

* same-callee branch pairs that differ only by argument values can be rewritten
  into `select arguments -> one async_call -> one spawn_task/spawn_thread`

### Binary runtime consumers

Covered:

* `timeout(task, limit)`

Current route:

* branch-local `timeout(...)` pairs can be rewritten through selected task and
  limit inputs
* the task input may itself come from a selected `spawn(...)` rewrite

## Coverage Matrix

### Direct branch result

Covered now:

* `if`: `spawn`, `thread_spawn`, `timeout`, `cancel`
* `match`: `spawn`, `thread_spawn`, `timeout`, `cancel`
* `if`: `join(spawn(...))`, `join_result(spawn(...))`
* `match`: `join(spawn(...))`, `join_result(spawn(...))`
* `if`: `thread_join(thread_spawn(...))`,
  `thread_join_result(thread_spawn(...))`
* `match`: `thread_join(thread_spawn(...))`,
  `thread_join_result(thread_spawn(...))`

### One-stage alias chain inside the branch

Covered now:

* `if` / `match`: `spawn(...) -> alias`
* `if` / `match`: `thread_spawn(...) -> alias`
* `if` / `match`: `timeout(...) -> alias`
* `if` / `match`: `mutex_lock(...) -> alias`
* `if` / `match`: `join_result(spawn(...)) -> alias -> return`
* `if` / `match`:
  `thread_join_result(thread_spawn(...)) -> alias -> return`

### Shared observer suffix after branch selection

Covered now:

* `if` / `match`: shared `TaskResult<T>` then `task_completed(...)`
* `if` / `match`: shared `TaskResult<T>` then `task_value(...)`
* `if` / `match`: shared `Thread<T>` then
  `thread_join_result(...) -> task_completed(...) / task_value(...)`
* `if` / `match`: shared `timeout(...)` task then
  `join_result(...) -> task_completed(...) / task_value(...)`
* `if` / `match`: shared `cancel(...)` task then
  `join_result(...) -> task_cancelled(...) / task_value(...)`
* `if` / `match`: shared `MutexGuard<T>` then `mutex_value(...)`
* `if` / `match`: nested task chains such as
  `timeout(cancel(spawn(...)), limit)` then
  `join_result(...) -> task_cancelled(...) / task_value(...)`
* `if` / `match`: nested task chains such as
  `join_result(timeout(spawn(...), limit))`

### Shared pure suffix after observer

Covered now:

* `if` / `match`: `task_value(joined)` feeding `+ 1`
* `if` / `match`: shared `Thread<T>` feeding
  `thread_join_result(...) -> task_value(...) -> + 1`
* `if` / `match`: shared `timeout(...)` task feeding
  `join_result(...) -> task_value(...) -> + 1`
* `if` / `match`: shared `cancel(...)` task feeding
  `join_result(...) -> task_value(...) -> + 1`
* `if` / `match`: nested `timeout(cancel(spawn(...)), limit)` feeding
  `join_result(...) -> task_value(...) -> + 1`
* `if` / `match`: `mutex_value(guard)` feeding `+ 1`
* `if` / `match`: two-stage pure suffixes such as
  `value -> value + 1 -> widened + 2`

## What This Matrix Does Not Claim

Not yet claimed here:

* arbitrary nested mixtures of unrelated runtime-effect families inside one
  branch tail
* branch-local runtime effects that differ by callee identity or arity
* general “any effectful branch will be normalized automatically” behavior
* arbitrary multi-layer nesting beyond the currently tested
  `binary(unary(call))` case `timeout(cancel(spawn(...)), limit)` and
  `unary(binary(call))` case `join_result(timeout(spawn(...), limit))`
* full CLI/source/project closure for every shape listed here

This file is a lowering-local coverage matrix, not a full compile-route matrix.

## Current Test Anchor

Primary lowering-local proof file:

* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)

Fast focused command:

```bash
cargo test -p nuisc lowers_dynamic_ -- --nocapture
```

Current dynamic branch proof count:

* `65` focused tests in the `lowers_dynamic_*` family

## Practical Reading Rule

If a new control-flow/runtime claim fits this matrix, extend the
`lowers_dynamic_*` family first.

If the claim goes beyond this matrix, do not quietly treat current greens as
proof. Either:

* add a new named matrix row and matching tests
* or document the remaining boundary explicitly in the compile-gap checklist
