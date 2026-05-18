# `task_join_nonconsuming_probe_demo`

This project is a **design probe**.

It is intentionally legal today.

The point of the sample is to keep one very specific shape visible:

* direct `join(task)` payload extraction
* followed later by `join_result(task)` observation on the same task handle

Current meaning:

* the repository still treats `join(...)` as a task payload boundary
* it does **not** yet treat `join(...)` as a graph-level `GLM` consume boundary

That is why this sample currently passes `check` and `build`.

## Why This Sample Exists

If `Task<T>` later becomes a stronger `GLM` ownership object, one likely
tightening would be:

* `join(...)` becomes the final consuming payload path for a task handle

If that happens, this sample becomes a natural conflict probe because it does:

1. `join(task)`
2. `join_result(task)` later in the same flow

Under a stricter future model, that second step may need to become invalid, or
the task contract may need to define a more explicit split between:

* consumable payload handles
* reusable observation handles

## How To Read It Today

Treat this project as:

* a current regression probe for the repository's present task semantics
* not a promise that this shape will remain legal forever

If task ownership rules become stricter in future `GLM` work, revisit this
sample first.

For the likely migration direction under a stricter future consume rule, see:

* [FUTURE_CONSUME_SKETCH.md](/Users/Shared/chroot/dev/nuislang/examples/projects/task_join_nonconsuming_probe_demo/FUTURE_CONSUME_SKETCH.md)
