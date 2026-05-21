# Future Consume Sketch

This note is intentionally **forward-looking**.

It does **not** describe the repository's current rules.

Instead, it records the most likely migration direction if a future `GLM`
tightening decides that:

* `join(...)` should become the final consuming payload boundary for `Task<T>`

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

That is legal today because `join(...)` is still not treated as a graph-level
consume boundary.

## Likely Future Tension

If `join(...)` becomes consuming, then this pattern becomes suspect because it
tries to do both:

1. direct payload extraction
2. later lifecycle observation on the same task handle

Under a stricter model, those two paths would need to separate.

## Most Likely Rewrite Direction

The most natural rewrite would be to move fully onto the observation path:

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

This note gives future `GLM` work a concrete migration target:

* if `join(...)` is strengthened
* then `join_result(...)` becomes the canonical path whenever code needs both
  lifecycle knowledge and payload extraction

In other words:

* current probe = "what is still legal today"
* this sketch = "what code would most likely migrate toward tomorrow"
