# `nuis` `beta-0.6.*` Mainline Entry

This file is the short current-line anchor for the `beta-0.6.*` minor line.
The first recorded checkpoint is Git commit `4ee09008` (`beta-0.6.0`). Git
history remains authoritative for later patch checkpoints. Independent Cargo
package versions are implementation-package versions and do not currently
encode the Nuis project release line.

`beta-0.6.*` remains an early-beta foundation-hardening line. It records a
connected and heavily checked toolchain, not stable public APIs, complete
backend parity, production support, or compiler self-hosting.

The previous curated phase entry is
[nuis-beta-0.3.0-mainline-entry.md](nuis-beta-0.3.0-mainline-entry.md).
The `beta-0.4.*` and `beta-0.5.*` patch sequences remain available in Git
history; no retrospective phase snapshots are invented for them.

Short rule:

`beta-0.6.* closes the first evidence-persisted native final-output routes while keeping compiler, CLI, and Nustar providers structurally separate`

## Canonical Reading Order

1. [../current-mainline-map.md](../current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)
3. [../reference/nuis-self-hosting-readiness.md](../reference/nuis-self-hosting-readiness.md)
4. [../reference/nuis-native-artifact-workflow.md](../reference/nuis-native-artifact-workflow.md)
5. [../reference/nsld-linker-frontdoor.md](../reference/nsld-linker-frontdoor.md)
6. [../reference/nsld-binary-assembly-gap-map.md](../reference/nsld-binary-assembly-gap-map.md)
7. [../reference/nsld-executable-finalizer-registry.md](../reference/nsld-executable-finalizer-registry.md)
8. [../reference/nustar-multi-backend-artifact-contract.md](../reference/nustar-multi-backend-artifact-contract.md)
9. [../reference/nuis-binary-format-protocol.md](../reference/nuis-binary-format-protocol.md)
10. [../reference/cffi-von-neumann-domain-contract.md](../reference/cffi-von-neumann-domain-contract.md)
11. [README.md](README.md)

## Connected Spine

```text
nuis source / nuis.toml
  -> nuis workflow and project frontdoor
  -> nuisc frontend, NIR, YIR, verification, and AOT emission
  -> registered Nustar domain artifacts and execution capsules
  -> Nsld deterministic graph, closure, NSB, and native shell pipeline
  -> registered loader admission and explicit final-output selection
  -> nuis-runtime lifecycle/context dispatch
  -> run-artifact and Nsdb completion/replay evidence
  -> development-tensor handoff
```

Common syntax and semantic contracts belong to the compiler and YIR. Backend
ABI, lowering, artifact, execution, and verification behavior remains behind
registered Nustar capabilities rather than a finite compiler-owned backend
matrix.

## Current Milestone Evidence

At this documentation refresh the development tensor reports:

* clean recursive hierarchy validation across 54 nodes and three axes
* `26/26` milestone coordinates covered
* `593/593` implementation-drift checks passing
* bootstrap-critical average progress at `83/100` after five honest
  self-hosting preparation coordinates are registered
* `language-core/nuisc/bootstrap-language-subset` as the weakest
  bootstrap-critical coordinate at `early`, `20/100`
* `linker-toolchain/nsld/os-native-executable-finalization` closed at
  `stable`, `100/100`

Tensor `stable` describes the recorded milestone slice. It is not a promise of
language, ABI, package, or standard-library compatibility.

## Native Finalization Closure

The first provider routes now exercise real OS-native execution:

* ARM64 Mach-O carries program/runtime objects through deterministic placement,
  common/absolute/indirect symbol resolution, relocation, stub/GOT and shell
  serialization, ad-hoc signing, independent validation, bounded loader probe,
  admission replay, atomic publication, and ordinary final-output selection.
* x86_64 Linux ELF carries `ET_REL` inputs through deterministic placement,
  direct and platform relocation, PLT/GOT/dynamic/version structures, private
  shell serialization, independent validation, registered GNU loader admission,
  atomic publication, and ordinary final-output selection.
* the real Linux route resolves hash-whitelisted and versioned `libc` plus
  `libm` dependencies, including `getrandom`, `cos`, and `sched_yield`, without
  adding provider branches to Nuisc or the public `nuis` command layer.
* explicit private-output selection persists relocatable owner-private
  `nuis-nsld-final-output-selection-evidence-file-v1` JSON. Compatibility
  output remains the non-mutating default and writes no selection sidecar.
* stale admission, signature, registration, image, or selection identity blocks
  output mutation. Signature-only dependency drift is rejected even when ELF
  bytes remain unchanged.

These routes establish a real early-beta closure, not full platform parity.
They remain bootstrap compatibility/finalization backends inside the larger
Nuis model.

## Current Priorities

1. freeze the compiler-authoring language subset and its rejected-feature
   fixtures
2. establish the compiler data model and producer-neutral IR handoff records
3. add a stage0-to-stage1 driver plus fail-closed differential reproducibility
   report before the migration boundary at `beta-0.10.*`
4. continue Galaxy, ELF, PE/COFF, standard-library, provider, performance, and
   bug hardening without coupling those lanes to the bootstrap driver

The first three resolver-registration objectives from the entry checkpoint are
closed: `official.cffi` owns two GNU providers and four symbol-version rows,
Nuisc validates and preserves them, and Nsld generates a static table while
retaining runtime registry identity `0xc6631e590d61aca8`.

## Honest Boundaries

The current repository still does not claim:

* equal native-finalizer maturity across architectures and operating systems
* a PE/COFF final executable route
* stable package, source, ABI, linker-script, or public API compatibility
* complete raw-pointer and unsafe interoperability policy
* compiler self-hosting or a Nuis-native operating-system substrate

## Version And Documentation Rule

Use these sources in this order:

1. checked-in implementation and tests for exact behavior
2. `nuis dev-tensor --json` for progress and next-action evidence
3. `docs/reference/` for present-tense contracts and boundaries
4. this file for the `beta-0.6.*` phase summary
5. older versioning entries only for historical reconstruction

Do not infer the project line from individual Cargo package versions. Portable
contracts must not contain absolute local paths, private server addresses, or
assumptions about one host OS release.
