# Window External-Handle Probe

This file is intentionally paired with:

* [hello_task_glm_window_external_handle_probe_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_task_glm_window_external_handle_probe_invalid.ns)

It is a **design probe**, not a current positive example.

## Why It Exists

The repository now treats `Window<...>` as a resource-bearing family that must
not silently cross the current async/task boundary.

So the paired `.ns` file is expected to fail today.

That failure is intentional.

## What It Is Probing

The shape:

* `frame: Window<i64>`
* `lane: i64`
* `domain_code: i64`

is meant to hint at a possible future external-handle direction:

* the window-like payload is not treated as a plain copied value
* it travels together with explicit scheduler/domain metadata
* it looks more like a task-external resource packet than a normal value packet

## What It Does **Not** Claim

This probe does **not** mean:

* `Window<...>` task payloads are already supported
* `lane` / `domain_code` are already the final contract
* `GLM` already knows how to model this crossing

It only exists so the repository has a concrete sample to point at when
discussing future external-handle semantics.
