# Future Cancel Sketch

This note is intentionally **forward-looking**.

It does **not** describe the repository's current rules.

Instead, it records the most likely migration direction if future `GLM` work
decides that `cancel(...)` should carry stronger ownership or lifetime
consequences.

## Current Shape

The current project uses this shape:

```ns
let task: Task<i64> = cancel(spawn(ping()));
let result: TaskResult<i64> = join_result(task);
if task_cancelled(result) {
  let summary: TaskSummary = cancelled_summary();
  print(summary.message);
  return summary.code;
} else {
  let summary: TaskSummary = live_summary();
  print(summary.message);
  return summary.code;
}
```

That shape is legal today because:

* `cancel(...)` already shapes lifecycle
* but it is not yet treated as a graph-level lifetime-end effect in `GLM`

## Likely Future Tension

If `cancel(...)` later becomes stronger in the ownership model, the repository
may want to answer questions like:

* whether cancellation should terminate later payload extraction entirely
* whether cancelled task handles should still remain reusable for observation
* whether cancellation should become a more explicit graph boundary than a plain
  `Task<T>`-to-`Task<T>` transform

## Most Likely Rewrite Direction

The most natural migration direction would still stay on the observation path:

```ns
let task: Task<i64> = cancel(spawn(ping()));
let result: TaskResult<i64> = join_result(task);
if task_cancelled(result) {
  return 71;
}
return 0;
```

If cancellation semantics tighten, the likely future shift is:

* keep observation explicit
* avoid mixing cancellation with later direct payload extraction

## Why This Sketch Exists

This note gives future `GLM` work a concrete migration anchor:

* if `cancel(...)` later becomes a stronger lifetime boundary
* then `join_result(...)` + `task_cancelled(...)` should remain the cleanest
  migration path

In other words:

* current sample = "today's legal cancel-branch shape"
* this sketch = "the likely direction if cancel semantics become stricter"
