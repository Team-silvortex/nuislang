# CPU Task External Handle GLM Sketch

This document is a design sketch for one specific future question:

* if task-external handle families ever cross `spawn(...)`,
  what should `GLM` probably think they are?

It is intentionally a **sketch**, not a current implementation claim.

## Why This Question Matters

The repository now has a clearer split between:

* plain value-like task payloads
* external/resource-bearing task payload candidates

That split already appears in:

* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
* [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)

But once a handle-like payload is discussed as a future task-crossing
candidate, a deeper question appears:

* should `GLM` see that thing as `val`, `res`, or something in between?

## Three Candidate Readings

### 1. Plain `val`

This is the lightest interpretation.

It would mean:

* the handle crosses the task boundary like an ordinary SSA-style value
* `spawn(...)` does not introduce a resource/lifetime edge for it
* `join(...)` and `join_result(...)` would observe only task lifecycle, not the
  lifetime of the external handle itself

Why this is attractive:

* simplest lowering story
* smallest short-term compiler change

Why this is risky:

* it hides the fact that `Window`, `Pipe`, `Marker`, and `HandleTable` already
  carry domain/runtime meaning
* it makes later ownership/visibility rules harder to recover
* it risks turning hetero resource flow into “just another value” too early

Current sketch judgment:

* probably too weak for most external-handle families

### 2. Full `res`

This is the heaviest interpretation.

It would mean:

* the handle is modeled like a graph-level resource/object
* task crossing would introduce explicit lifetime/ownership edges
* `spawn(...)`, `join(...)`, `cancel(...)`, and `timeout(...)` might all need
  dedicated `GLM` resource semantics for these handles

Why this is attractive:

* matches the intuition that these are not plain values
* aligns better with `GLM`’s existing `res / Read / Write / Own` vocabulary

Why this is risky:

* could overcommit too early
* not every external-handle family may need full ownership transfer semantics
* might force the repository to settle task/runtime rules before the executor,
  lane, and visibility contracts are ready

Current sketch judgment:

* probably too strong as a first move

### 3. Bridge Object

This is the current most promising middle reading.

It would mean:

* the payload is not a plain value
* but it is also not yet a full `res` ownership object
* `GLM` would treat the crossing as a special bridge-shaped object carrying:
  * external handle identity
  * domain/lane metadata
  * possible visibility/ownership-transfer intent

Why this is attractive:

* preserves the fact that these families are semantically heavier than plain
  values
* avoids pretending the final `res` story is already solved
* matches the repository’s current staged style:
  * first make the bridge visible
  * later decide how much ownership/lifetime it should carry

Why this is still unfinished:

* `GLM` does not currently have a first-class bridge-object class
* `YIR` would need a clearer way to say “this is a cross-task external handle
  bridge”
* lane/clock/runtime visibility rules would still need to be attached later

Current sketch judgment:

* best current candidate for future external-handle task crossing

Current repository anchor:

* `yir-core` now carries `bridge` and `task-external-handle` as naming
  placeholders only
* they are not active `GLM` node-profile classes yet
* this is intentional, so the vocabulary can stabilize before the semantics do
* handwritten `YIR` may now also carry the same idea through comment-only
  sketch tags such as `# sketch.glm bridge` and
  `# sketch.bridge-kind task-external-handle`

## Family-by-Family Intuition

These are not commitments, just current directional hints.

### `Window<...>` / `WindowMut<...>`

Best current intuition:

* bridge object first
* maybe `res` later if lifetime/ownership transfer becomes explicit

Why:

* these already look like data/fabric resource carriers
* but the exact task-crossing semantics probably depend on later visibility and
  transfer rules

### `Pipe<...>`

Best current intuition:

* bridge object first

Why:

* pipes already express a fabric/data routing meaning that is heavier than
  plain values
* but they do not automatically imply direct ownership of the routed data

### `Marker<...>`

Best current intuition:

* bridge object first, probably never plain `val`

Why:

* markers are control-plane signals
* they behave more like coordination bridges than resource-owning objects

### `HandleTable<...>`

Best current intuition:

* bridge object first

Why:

* handle tables are routing/control structures
* they carry topology/schema meaning, not just data payload meaning

### `Instance<...>`

Best current intuition:

* unresolved, but likely closer to `res` than plain `val`

Why:

* staged domain ownership is already part of its meaning
* it seems less likely than `Marker` or `HandleTable` to remain “just a bridge”

## Relationship To Current Probes

Current design probes that motivate this sketch:

* [hello_task_glm_window_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_window_external_handle_probe_invalid.ns)
* [hello_task_glm_marker_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_marker_external_handle_probe_invalid.ns)
* [hello_task_glm_handle_table_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_handle_table_external_handle_probe_invalid.ns)
* [cpu_task_external_handle_bridge_probe.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu/cpu_task_external_handle_bridge_probe.yir)
* [data_external_handle_bridge_probe.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_external_handle_bridge_probe.yir)
* [shader_external_handle_bridge_probe.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_external_handle_bridge_probe.yir)

Those samples are still invalid today, but they help show what future
bridge-shaped task packets, Fabric-side bridge candidates, and render-side
packet/state bridge candidates might look like.

## Current Working Conclusion

If the repository later allows resource-bearing families across task boundaries,
the current best design direction is:

* do **not** silently treat them as plain `val`
* do **not** immediately force them into full `res` semantics
* first introduce a bridge-object reading
* then decide, family by family, whether some of them later harden into full
  `res` ownership objects

That approach fits the current `nuis` philosophy well:

* keep hetero semantics visible
* avoid lying about concurrency maturity
* grow the contract in stages instead of collapsing everything into a
  host-language value model

## Relationship To Other References

Read this together with:

* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
