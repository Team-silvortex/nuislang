# Future Hot Sync Contraction Sketch

This note is intentionally **forward-looking**.

It does **not** describe current repository behavior.

Instead, it marks a very small region in
[task_completed_observe_demo](./)
as a plausible first probe for future verifier-driven async-to-sync
contraction work.

## Current Shape

Today the project uses:

```ns
let task: Task<i64> = spawn(ping());
let result: TaskResult<i64> = join_result(task);
if task_completed(result) {
  return task_value(result);
}
return 0;
```

Current reading:

* explicit task creation
* explicit result observation
* explicit completed-state probe
* explicit payload extraction

That shape is good for current correctness and visibility.

## Why This Is A Good First Contraction Probe

Compared with many other task samples in the repository, this one is unusually
small and clean:

* no borrowed or resource-bearing payload
* no external observer after the local region
* no `cancel(...)` branch
* no `timeout(...)` branch
* no explicit lane-sensitive control decision
* no cross-domain timing bridge inside the local path itself

That makes it a good future probe for the question:

* can a very small local observe/completed/value region be proven equivalent to
  a simpler sync-shaped result path?

## The Likely Future Region

The most likely future contraction target is not the whole function at once.

It is the local region made of:

1. `spawn(ping())`
2. `join_result(task)`
3. `task_completed(result)`
4. `task_value(result)`

In other words, the interesting question is:

* when all of those stay local and closed,
* can the compiler later collapse their async-state scaffolding
* without changing the visible result contract?

## What Must Still Be Proven

Even for this tiny sample, a future contraction pass should still need to prove
things like:

* the payload remains value-like
* the observed task state does not escape
* there is no required later observer of the original task lifecycle
* removing the local async envelope does not erase meaningful timing semantics
* the region is not relying on future stronger `join(...)` ownership rules

So this note is not saying “this sample should obviously become sync.”

It is saying:

* if the repository wants a first narrow hot-sync probe,
* this sample is one of the safest places to start looking

## Most Likely Future Experiment

The likely first experiment would be to compare:

* the current explicit observe path

against a future lowered form that behaves more like:

* a locally contracted sync result path

while still preserving the same externally visible return behavior.

The exact lowered form should remain an implementation question until verifier
and clock contracts are stronger.

## Why This Note Exists

This note gives future optimization/contraction work a concrete starting point:

* first choose a tiny, value-like, observer-local region
* prove it
* validate it against this project
* only then generalize to more cancellation-sensitive or clock-sensitive paths

That fits the repository’s current philosophy well:

* async-first by default
* explicit semantics first
* contraction only after proof
