# `core`

`core` is the smallest stable layer of the `nuis` standard library.

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
* stable struct/result/option-like source patterns once the frontend library/import model settles
* the lowest shared contracts for CPU/data/shader/kernel-facing source code

See metadata:

* [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/core/module.toml)

