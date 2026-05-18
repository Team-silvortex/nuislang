# Future Lifecycle Sketch

This note is intentionally **forward-looking**.

It does **not** describe the repository's current rules.

Instead, it records the most likely migration direction if future `GLM` work
decides that task lifecycle-shaping operations should carry stronger ownership
or lifetime meaning.

## Current Shape

The current project uses this shape:

```ns
let task: Task<i64> = timeout(spawn(ping()), 0);
let result: TaskResult<i64> = join_result(task);
if task_timed_out(result) {
  let summary: TaskSummary = timeout_summary();
  print(summary.message);
  return summary.code;
} else {
  let summary: TaskSummary = completed_summary();
  print(summary.message);
  return summary.code;
}
```

That shape is legal today because:

* `timeout(...)` already shapes lifecycle
* but it is not yet treated as a graph-level lifetime-end or ownership-transfer
  boundary in `GLM`

## Likely Future Tension

If future `GLM` work strengthens lifecycle semantics, then the repository may
want to say something firmer about:

* whether `timeout(...)` creates a task state that is terminal in an ownership
  sense
* whether a timed-out task can still be reused, re-observed, or rejoined
* whether lifecycle-shaping operations should produce more explicit boundary
  objects than a plain `Task<T>` handle

## Most Likely Rewrite Direction

The most natural migration direction would still preserve the observation path:

```ns
let task: Task<i64> = timeout(spawn(ping()), 0);
let result: TaskResult<i64> = join_result(task);
if task_timed_out(result) {
  return 74;
}
return 0;
```

If the semantics tighten, the likely change is not "stop observing lifecycle",
but rather:

* make the lifecycle boundary itself more explicit
* keep payload extraction and lifecycle observation separate

## Why This Sketch Exists

This note gives future `GLM` work a concrete migration target:

* if `timeout(...)` later gains stronger lifetime meaning
* then code should continue to prefer `join_result(...)` + `task_timed_out(...)`
  as the canonical observation path

In other words:

* current sample = "today's legal lifecycle-branch shape"
* this sketch = "the likely direction if timeout semantics grow sharper later"
