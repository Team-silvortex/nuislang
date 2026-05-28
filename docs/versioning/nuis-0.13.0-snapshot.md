# `nuis` 0.13.0 Snapshot

This file is the first lightweight phase snapshot for the current repository
spine.

It is not a historical changelog dump and it is not a full compatibility
policy. It is the shortest “what is now solid enough to stand on” anchor for
the `0.13.0` phase.

## What `0.13.0` Means Here

`0.13.0` is the point where the repository has a meaningfully more coherent
language/toolchain surface across:

* source visibility and module boundaries
* intrinsic annotations as compiler/std-owned frontend conventions
* trait/generic parsing plus first constrained monomorphization slices
* packet schema/contract metadata
* low-level `std net` syscall/socket layering
* executable `while` subfamilies beyond pure guard stubs

This is still an architecture-building phase line, not a “language complete”
milestone.

## High-Signal Current Surface

The most important current truths for `0.13.0` are:

* `nuis -> NIR -> YIR -> LLVM/AOT` remains the mainline execution spine.
* `pub/private` now exists as a real source-organization boundary for:
  - `fn`
  - `struct`
  - `field`
  - `trait`
  - `extern`
  - `extern interface`
* `project-status` and `project-doctor` now expose a visible `public surface`
  summary instead of treating exported source structure as implicit.
* function annotations are now a real frontend container with MVP semantics for:
  - `@test(...)`
  - `@export(name = "...")`
  - `@inline`
  - `@noinline`
  - `@host_symbol("...")`
* `@packet`, `@packet_field`, and `@packet_control_field` now form a real
  packet-schema contract with build metadata, slot vocabulary, and basic packet
  validation.
* minimal trait/generic support now exists for:
  - `trait`
  - `impl`
  - `fn f<T: Trait>(...)`
  - narrow monomorphization
  - shaped inference for common wrappers such as `Pipe<T>`, `Window<T>`,
    `Task<T>`, and `DataResult<T>`
* `while` is no longer “parser-only”. There is now a real executable counted /
  carry / branch / flow subset.
* `std net` now has a clearer low-level ladder:
  - syscall edge
  - socket edge
  - client/server/datagram flow
  - session-facing recipes above that ladder

## What Is Still Intentionally Narrow

`0.13.0` should still be read with these boundaries in mind:

* visibility is intentionally small:
  - default private
  - explicit `pub`
  - no `pub(crate)` / `pub(super)` family yet
* trait/generic support is still MVP-only:
  - no `where`
  - no associated types
  - no trait objects
  - no blanket impl
* annotations are preferred frontend conventions, not the sole semantic truth.
  Stable truth still lives deeper in registered `nustar` capability contracts
  and lowering/binding contracts.
* packet support is schema/contract-first. It is not yet a fully generated
  serializer system.
* executable loop support is real, but still intentionally narrower than a full
  mature Rust loop model.

## Best Current Reading Order

For `0.13.0`, the shortest practical path is:

1. [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
2. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
3. [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
4. [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
5. [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)

## `0.13.0` Focus Areas

If you want the shortest thematic picture of this phase line:

* source organization:
  `pub/private + public-surface visibility`
* frontend conventions:
  `intrinsic annotations over open-ended metaprogramming`
* systems boundary:
  `AOT-first mainline, light lifecycle runtime, registered nustar contracts`
* network foundation:
  `syscall-facing std net before richer http convenience`
* future direction:
  `replaceable frontend forms, replaceable nustar implementations, stable
  registered capability contracts`

## Recommended Practical Commands

```bash
cargo run -p nuis -- project-status <project-dir>
cargo run -p nuis -- project-doctor <project-dir>
cargo run -p nuis -- check <project-dir>
cargo run -p nuis -- test <project-dir>
cargo run -p nuis -- build <project-dir> <output-dir>
```

For source/public-boundary inspection specifically:

```bash
cargo run -p nuis -- project-status <project-dir>
cargo run -p nuis -- project-doctor <project-dir>
```

These now surface `public_surface` summaries directly.

## Rule Of Thumb

If a future sketch and this snapshot differ:

* prefer the current implementation
* prefer `docs/reference`
* prefer `docs/current-mainline-map`
* treat this file as a short phase anchor, not as a source replacement
