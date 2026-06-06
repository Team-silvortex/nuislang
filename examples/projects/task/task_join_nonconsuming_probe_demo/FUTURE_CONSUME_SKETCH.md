# Future Consume Sketch

This note records the migration direction that the repository now follows:

* `join(...)` is the final consuming payload boundary for `Task<T>`
* `join_result(...)` is also consuming, so code that needs lifecycle metadata
  and payload should stay on the result-observer path from the start

## Current Shape

The current probe uses this shape:

```ns
let task: Task<i64> = spawn(ping());
let direct_value: i64 = join(task);
let observed_result: TaskResult<i64> = join_result(task);
if task_completed(observed_result) {
  return direct_value + task_value(observed_result);
}
return direct_value;
```

That shape is no longer legal because both task result paths now consume the
same handle.

## Likely Future Tension

Because `join(...)` is consuming, this pattern is now suspect because it tries
to do both:

1. direct payload extraction
2. later lifecycle observation on the same task handle

Under the stricter model, those two paths need to separate.

## Most Likely Rewrite Direction

The most natural rewrite is to move fully onto the observation path:

```ns
let task: Task<i64> = spawn(ping());
let result: TaskResult<i64> = join_result(task);
if task_completed(result) {
  return task_value(result);
}
return 0;
```

That shape already matches the repository's current observation guidance.

## Why This Sketch Exists

This note gives `GLM` work a concrete migration target that is now active:

* `join_result(...)` is the canonical path whenever code needs both lifecycle
  knowledge and payload extraction

In other words:

* current probe = "what is intentionally rejected today"
* this sketch = "what code should migrate toward now"
