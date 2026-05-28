# Nuis Launcher / Container / Linker Sketch

This file sketches a short design direction for how `nuis` should eventually
organize executable artifacts in an explicitly heterogeneous world.

The important split is:

* host platforms own process launch
* `nuis` owns program organization

That means an operating system's native executable format should be treated as
the startup shell, not as the deepest semantic truth for how a `nuis` program
is packaged internally.

## Why This Split Matters

If the repository eventually wants:

* unified frontend shapes across the same capability family
* cross-compilation
* heterogeneous static packaging
* host-replaceable `nustar` implementations
* domain-owned packet / task / network / shader contracts

then the host linker alone is not a large enough organizing model.

Native host formats such as:

* `mach-o`
* `elf`
* `pe/coff`

are good at:

* getting the process started
* satisfying host loader rules
* expressing host ABI facts

but they are not the natural place to encode:

* domain-family capability segments
* heterogeneous compute payload groupings
* std-owned packet contracts
* registered `nustar` capability composition
* cross-domain static linking intent

## Layer Split

The intended long-range split should be:

### 1. Launcher Shell

This is the host-native entry artifact.

Examples:

* `mach-o` launcher on Darwin
* `elf` launcher on Linux
* `pe/coff` launcher on Windows

Responsibilities:

* satisfy the host kernel / loader
* bootstrap the process
* find or embed the `nuis` container payload
* hand control to the `nuis` runtime entry

This layer should stay intentionally thin.

### 2. Nuis Runtime Container

This is the deeper program payload owned by `nuis`.

Responsibilities should eventually include:

* registered capability aggregation
* packaged `nustar` implementation segments
* domain-family metadata
* packet / serialization contract indexes
* host bridge declarations
* cross-domain resource layout
* static-link style composition for heterogeneous program parts

The host OS should not need to understand this layout in detail.

The host only needs enough shell logic to start it.

### 3. Nuis Linker

This is the repository-owned composition step that sits above host linkers.

Its job is not merely:

* “emit one native binary”

but more importantly:

* resolve registered capability requirements
* choose compatible `nustar` implementations
* validate `abi_targets`
* group heterogeneous payload segments
* freeze the internal container layout
* then ask the host toolchain to wrap that layout in a native launcher shell

So the host linker remains useful, but it becomes the last-mile wrapper rather
than the primary semantic organizer.

## ABI Grain

This suggests a three-level grain model:

### Package Grain

`nustar` packages should stay relatively coarse and follow capability families.

Examples:

* `official.cpu`
* `official.network`
* `official.shader`
* `official.kernel`

This keeps frontend shape stable for users working within the same family.

### ABI-Target Grain

Within one package, `abi_targets` can stay fine-grained.

Current checked-in targets already look like:

* `arch=...`
* `os=...`
* `object=...`
* `calling=...`
* `clang=...`
* optional `backend=...`

This is the right place to express:

* x86_64 vs arm64
* Linux vs Darwin vs Windows
* SysV vs Win64 vs AAPCS64
* backend-family differences

### Artifact Grain

The final built implementation artifact should remain concrete.

In other words:

* one logical package may register many ABI targets
* one built artifact should still correspond to one concrete target contract

This fits the repository's current exact-match machine-ABI direction better
than a single universal host artifact pretending to cover everything equally.

## Frontend Unification Rule

The value of “unified heterogeneous computing” is not that all targets become
identical internally.

The value is that source-facing frontend shape stays as uniform as possible
within one capability family.

So the preferred rule should be:

* frontend shape follows capability families
* ABI variation stays as deep in registration and lowering as possible

For example:

* CPU source frontend should not split early just because the target is
  Darwin vs Linux
* network frontend should not split early just because the socket backend lands
  on different host ABIs

Surface variation should appear only when the capability itself differs, not
just because the host launcher format differs.

## Conditional Compilation

Conditional compilation is still important, but it should be used carefully.

Good use:

* expressing genuine capability differences
* selecting platform-specific bridges
* enabling optional backend-owned features

Less desirable use:

* forcing ordinary CPU/network source frontend to fork early only because of
  platform packaging

The ideal relationship is:

* frontend remains mostly family-unified
* conditions express capability or bridge differences
* `abi_targets` and linker/container logic handle the majority of platform
  specificity

## Intrinsic Annotations

This also explains why intrinsic annotations will matter later.

Annotations can express:

* host-boundary facts
* packet/serialization facts
* backend requirements
* linker/container hints
* conditional capability intent

without requiring arbitrary open-ended macro systems.

In this model, annotations should help the compiler and linker organize the
program, not redefine the semantic truth from scratch.

## Shortest Rule

The shortest long-range packaging rule is:

`the operating system launches the program; nuis owns the program's real structure`

And the shortest ABI-grain rule is:

`frontend unifies by capability family; ABI specializes by registered target`
