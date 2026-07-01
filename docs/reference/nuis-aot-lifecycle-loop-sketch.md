# Nuis AOT Lifecycle Loop Sketch

This file sketches the likely execution model for a future `nuis` program if
the repository continues to prefer:

* ahead-of-time compilation
* a relatively light runtime
* repository-owned heterogeneous program organization

The central idea is:

* the host operating system launches a thin entry point
* `nuis` then maintains its own internal program lifecycle loop

So the right mental model is not “no runtime at all”.
It is:

* no heavy general-purpose VM-style runtime
* but a small, explicit, `nuis`-owned lifecycle runtime

## Shortest Rule

`host main starts the process; nuis owns the lifecycle loop`

## Why This Fits AOT

For an AOT-oriented language, a heavy host-managed runtime is usually the wrong
default.

The repository still needs somewhere to organize:

* scheduler ticks
* task/session state
* host bridge readiness
* packet and I/O boundaries
* resource cleanup
* heterogeneous capability coordination

Those are runtime responsibilities, but they do not require a large opaque VM.

They fit better as a narrow lifecycle layer that comes after startup.

## Phase Split

The long-range execution path should be treated as four phases.

### 1. Host Entry

This is the OS-visible `main` or equivalent entry shell.

Responsibilities:

* satisfy host ABI requirements
* enter the process
* locate or embed the `nuis` runtime container
* transfer control to the `nuis` bootstrap entry

This layer should stay minimal.

It should not own deep program semantics.

### 2. Bootstrap Phase

This is the first `nuis`-owned execution stage.

Responsibilities should eventually include:

* validate the embedded or attached program container
* resolve registered capability segments
* initialize required host bridges
* bind scheduler defaults
* create the first lifecycle state
* choose the initial unit/session/task entry

This is where startup becomes `nuis`-semantic rather than merely OS-semantic.

### 3. Internal Lifecycle Loop

This is the steady-state execution model.

Responsibilities may eventually include:

* task polling / completion handling
* host I/O readiness progression
* network/session progression
* packet movement and serialization boundaries
* shader/kernel submission orchestration
* result observation / summary updates
* lifecycle-phase transitions between active / waiting / draining states

This loop does not have to look like a traditional GUI event loop only.

It is better thought of as a repository-owned program pump that can unify:

* scheduler-style progression
* asynchronous host bridge progression
* heterogeneous work submission

### 4. Shutdown Phase

This is the final `nuis`-owned exit stage.

Responsibilities should eventually include:

* flushing pending result summaries
* shutting down bridges in a controlled order
* releasing owned resources
* mapping internal outcome state to host exit code / host status

This again keeps the semantic truth inside `nuis`, not inside the launcher
shell.

## Relationship To Launcher / Container / Linker

This document depends on the layering in
[nuis-launcher-container-linker-sketch.md](nuis-launcher-container-linker-sketch.md).

The split is:

* launcher shell
  * gets the process started
* `nuis` container
  * carries the real program payload
* `nuis` linker
  * decides how the payload is assembled
* lifecycle loop
  * decides how the assembled payload actually runs

So startup packaging and steady-state execution are related, but they are not
the same design question.

## Why This Helps Heterogeneous Programs

If a program may eventually coordinate:

* CPU work
* host I/O
* network traffic
* packet serialization boundaries
* shader/kernel submission

then a repository-owned lifecycle loop is much easier to evolve than treating
the host entry shell as the main organizer.

This is also a cleaner place to keep source frontend mostly uniform across the
same capability family while letting backend/platform specifics stay deeper in
registered targets and lowering.

## Conditional Compilation And Intrinsic Annotations

Conditional compilation and intrinsic annotations still matter in this model,
but they should support the lifecycle rather than replace it.

Good future uses:

* bridge-specific startup hooks
* backend-specific lifecycle capabilities
* shutdown policy hints
* scheduler / packet / host boundary intent

These should help the compiler and linker wire the lifecycle correctly without
forcing the language into a heavy reflective runtime.

## JIT-Compatible Reserved Cut

Even if the repository stays primarily AOT, the lifecycle model should leave a
clear managed cut for future JIT-related features.

That cut should live inside the `nuis` lifecycle phases, not in the host shell.

Possible reserved responsibilities:

* bootstrap-time runtime service registration
* optional code-region materialization hooks
* managed recompilation / specialization checkpoints
* lifecycle-safe patch / replacement points
* shutdown-time code cache cleanup

The important part is that these stay as explicit lifecycle-owned surfaces.

That way:

* AOT remains the default
* JIT does not force a total redesign
* future managed services can be inserted without changing the basic host entry
  contract

## What This Model Should Avoid

It should avoid:

* a large always-on VM as the default execution substrate
* pushing deep lifecycle truth into OS-native executable formats
* forcing source frontend to fork early by platform when lifecycle semantics are
  actually shared
* assuming that “AOT” means “no internal runtime organization at all”

## Practical Long-Range Rule

The practical long-range rule should be:

`AOT by default, light lifecycle runtime by design, managed cut reserved for future JIT`
