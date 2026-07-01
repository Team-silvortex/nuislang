# Host Read Bridge

This file explains the current bridge between compiler-recognized host-facing
reads and the checked-in `std` host facade modules.

It exists because those two things are related, but they are not the same layer
yet.

## Current Split

Today the repository has two parallel host-facing surfaces:

* compiler-recognized `NIR` host reads
* source-visible `std` host facade modules built mostly on `extern "c"`

The first surface is what the compiler can classify semantically today.

The second surface is what users can write against today in `.ns` source.

## Compiler-Recognized Host Reads

Current canonical source:

* [model.rs](../../crates/nuis-semantics/src/model.rs)

Current `HostReadOnly` examples include:

* `cpu_tick_i64(...)`
* `cpu_input_i64(...)`
* `ShaderTarget`
* `ShaderViewport`
* `ShaderPipeline`
* `ShaderInlineWgsl`

Current bridge helper:

* `nir_host_read_surface(...)`
* `nir_host_scheduler_bridge(...)`
* `NirHostTimingBridge`

Current host-read surface groups are:

* `SchedulerLane`
* `InputChannel`
* `ClockTick`
* `RenderDescriptor`

These are the places where the compiler already knows “this is a host-facing
read-like thing” without treating it as a generic opaque call.

For `SchedulerLane`, the compiler now also has a canonical scheduler bridge:

* `cpu_bind_core(0) -> host_main_lane`
* `cpu_bind_core(n>0) -> worker_lane`
* current resolved source name: `cpu_bind_core_lane`

## Clock / Test Alignment

The current clock story is intentionally split:

* compiler-known host read:
  * `cpu_tick_i64(...)`
  * classified as `HostReadOnly`
  * grouped under `ClockTick`
* staging global-clock bridge:
  * [clock_runtime.ns](../../stdlib/std/clock_runtime.ns)
  * [clock_domain_runtime.ns](../../stdlib/std/clock_domain_runtime.ns)
  * [clock_test_recipe.ns](../../stdlib/std/clock_test_recipe.ns)
  * [hello_clock_test_facades.ns](../../examples/ns/ffi/hello_clock_test_facades.ns)
  * `nuis test` runner metadata such as `declared_clock_domain` and
    `resolved_clock_domain`
  * compiler-aware timing bridge names such as:
    * `monotonic_tick`
    * `wall_deadline`
    * `global_to_monotonic_tick_bridge`
  * compiler-aware host-read surface names such as:
    * `clock_tick`

That means the compiler currently recognizes host-local ticking directly, but
the richer `global -> monotonic` bridge for async tests is still a front-door
and `std`-level contract rather than a fully compiler-owned host-read surface.
What has improved now is that this bridge has a canonical compiler-side name,
so the runner, semantics layer, and `std` bridge documents are no longer each
inventing their own wording for the same resolution step. The runner also now
exposes the resolved host-read surface directly, so the bridge is visible not
just as a policy or source string, but as a compiler-known surface category.

The same naming pattern now also applies to scheduler lanes: there is a
compiler-known `SchedulerLane` surface, plus a narrower scheduler bridge that
classifies the current `cpu_bind_core(...)` usage as either `host_main_lane` or
`worker_lane`.

## `std` Host Facades

Current practical source layer:

* [stdlib/std/README.md](../../stdlib/std/README.md)

Many `std` modules still work by declaring explicit host functions such as:

* `host_argv_count()`
* `host_monotonic_time_ns()`
* `host_stdout_write(...)`
* `host_fs_exists(...)`

That means they usually lower through `cpu.extern_call_*`, which currently
remains `Stateful` in compiler effect classification.

This is intentional for now: the compiler does not yet pretend that every host
FFI function is a safe read-only probe.

One concrete current example is the bootstrap `network` control bridge:

* [network_host_control_runtime_demo](../../examples/projects/domains/network_host_control_runtime_demo)

That sample lowers `host_network_connect_probe(...)`,
`host_network_accept_probe(...)`, and `host_network_close(...)` through
`cpu.extern_call_i64`. So even though those calls now participate in an
explicit `network` loader/runtime symbol contract, they still remain on the
conservative CPU host-FFI side of the current effect model.

## Current Contract

The safe thing to assume today is:

* compiler-recognized builtins may be classified as `HostReadOnly`
* raw host FFI facades in `std` should still be treated conservatively unless a
  more explicit bridge is added
* the current async/global clock bridge is explicit and visible, but it is not
  yet a universal compiler-level host-read contract

So the bridge is currently partial, not universal.

## Why This Matters

This split lets the repository move in a controlled way:

* users can already write practical CLI/tooling code through `std`
* the compiler can still be conservative about host-side effects
* future work can selectively promote specific host facade patterns into more
  explicit compiler-known host-read contracts

That is the intended direction for clock, input, and other runtime-facing
surfaces as `nuis` matures.
