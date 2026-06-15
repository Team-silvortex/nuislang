# `core`

`core` is the smallest stable layer of the `nuis` standard library.

## Current Status

At the current repository stage, `core` is still mostly a layout/contract
layer, but it now also has its first small checked-in `.ns` source set.

That is intentional: the layer is still small and conservative, but we have now
started checking in the first canonical source modules for simple scalar and
struct patterns.

Intended scope:

* primitive source-level value vocabulary
* source-visible ownership/reference conventions that should remain stable across backends
* minimal math and scalar contracts that do not imply host OS, allocator, or rendering runtime policy
* the semantic baseline that `std` and `ns-nova` can safely build on

Non-goals:

* no heavy host integration policy
* no GPU/application framework assumptions
* no convenience-first collections layer unless it can stay backend-neutral and semantically small

Planned direction:

* typed scalar aliases and canonical prelude surface
* the lowest shared contracts for CPU/data/shader/kernel-facing source code

Source patterns that now exist:

* stable struct construction examples
* canonical enum-based `Option<T>` / `Result<T, E>` / `CoreError` patterns for source-level error handling
* `match`-driven success/error branching that higher layers can reuse before a fuller import/prelude story lands
* generic helper families such as `option_map`, `option_and_then`, `result_map`, `result_map_err`, `result_and_then`, and `result_from_option`
* direct-statement `?` propagation for `Result<Payload, Error>` in source, lowered through the same enum-based branching model

First source modules:

* [basic_scalars.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/basic_scalars.ns)
* [struct_patterns.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/struct_patterns.ns)
* [math_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/math_runtime.ns)
* [ref_runtime.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/ref_runtime.ns)
* [value_blueprint.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/value_blueprint.ns)
* [result_patterns.ns](/Users/Shared/chroot/dev/nuislang/stdlib/core/result_patterns.ns)

What is not true yet:

* `core` is not yet a populated importable source module tree
* `core` is still intentionally much thinner than `ns-nova`

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/core/module.toml)
