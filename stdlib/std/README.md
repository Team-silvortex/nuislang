# `std`

`std` is the practical systems layer above `core`.

## Current Status

At the current repository stage, `std` is also still mostly a layout/contract
layer, but it now has its first small checked-in `.ns` source set.

That means its role is already important for dependency boundaries, and it is
now starting to accumulate small reusable modules for data/window/pipe helper
patterns.

Intended scope:

* convenience APIs that still preserve the AOT-first and semantics-first nature of `nuis`
* data-plane and host-integration helper surfaces that are too opinionated for `core`
* common utilities that typical `nuis` projects should not have to rebuild each time

Expected areas:

* collections and builder-style helper surfaces
* host FFI helper facades for CPU-side integration
* common data/window/handle-table orchestration helpers
* project/runtime utility APIs that are broadly useful but not framework-specific

Relationship:

* `std` depends on `core`
* `ns-nova` will use `std` as its general-purpose support layer rather than duplicating systems helpers

First source modules:

* [window_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/window_runtime.ns)
* [pipe_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/pipe_runtime.ns)
* [fabric_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fabric_runtime.ns)
* [handle_table_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/handle_table_runtime.ns)

What is not true yet:

* `std` is not yet a populated importable source module tree
* `std` is still much thinner than the future practical systems layer it is
  meant to become

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/std/module.toml)
