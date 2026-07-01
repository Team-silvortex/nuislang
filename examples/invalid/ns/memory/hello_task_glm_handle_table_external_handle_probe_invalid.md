# HandleTable External-Handle Probe

This file is intentionally paired with:

* [hello_task_glm_handle_table_external_handle_probe_invalid.ns](hello_task_glm_handle_table_external_handle_probe_invalid.ns)

It is a **design probe**, not a current positive example.

## Why It Exists

The repository currently treats `HandleTable<...>` as a control-plane family
that must stay outside the current async/task payload boundary.

So the paired `.ns` file is expected to fail today.

## What It Is Probing

The shape:

* `handles: HandleTable<FabricBindings>`
* `lane: i64`
* `domain_code: i64`

is meant to hint at a possible future external-handle direction:

* the handle-table payload is not treated as a copied plain value
* it travels together with explicit scheduler/domain metadata
* it looks more like a task-external routing/control packet than a normal
  value packet

## What It Does **Not** Claim

This probe does **not** mean:

* `HandleTable<...>` task payloads are already supported
* `lane` / `domain_code` are already the final contract
* `GLM` already knows how to model this crossing

It only exists so the repository has a concrete control-plane probe to point at
when future task-external handle semantics are discussed.
