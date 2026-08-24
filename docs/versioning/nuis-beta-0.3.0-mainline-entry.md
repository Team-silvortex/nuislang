# `nuis` `beta-0.3.*` Mainline Entry

> Historical phase snapshot. For current mainline behavior, start with
> [nuis-beta-0.6.0-mainline-entry.md](nuis-beta-0.6.0-mainline-entry.md).
> Present-tense wording below is preserved in its `beta-0.3.*` context.

This file is the recorded short entry point for the `beta-0.3.*` minor line.

Git history is authoritative for the exact patch checkpoint. The repository's
independent Cargo package versions are implementation-package versions and do
not currently encode the Nuis project release line.

`beta-0.3.*` remains an early-beta hardening line. It records a connected and
heavily checked foundation, not stable public APIs, complete heterogeneous
backend parity, production-ready native publication, or self-hosting.

The previous curated phase entry is
[nuis-beta-0.1.0-mainline-entry.md](nuis-beta-0.1.0-mainline-entry.md).
The `beta-0.2.*` patch sequence is preserved in Git history; this entry treats
it as the implementation transition into the current linker/runtime closure
rather than inventing a retrospective snapshot after the fact.

Short rule:

`beta-0.3.* hardens the full artifact/runtime path and moves Nsld from final-output planning into deterministic private OS-shell byte ownership`

## Canonical Reading Order

1. [../current-mainline-map.md](../current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)
3. [../reference/nuis-native-artifact-workflow.md](../reference/nuis-native-artifact-workflow.md)
4. [../reference/nsld-linker-frontdoor.md](../reference/nsld-linker-frontdoor.md)
5. [../reference/nsld-binary-assembly-gap-map.md](../reference/nsld-binary-assembly-gap-map.md)
6. [../reference/nsld-executable-finalizer-registry.md](../reference/nsld-executable-finalizer-registry.md)
7. [../reference/nustar-multi-backend-artifact-contract.md](../reference/nustar-multi-backend-artifact-contract.md)
8. [../reference/nuis-binary-format-protocol.md](../reference/nuis-binary-format-protocol.md)
9. [../reference/toolchain-galaxy-core-boundary.md](../reference/toolchain-galaxy-core-boundary.md)
10. [README.md](README.md)

## Connected Spine

```text
nuis source / nuis.toml
  -> nuis workflow and project frontdoor
  -> nuisc frontend, NIR, YIR, verification, and AOT emission
  -> registered Nustar domain artifacts and execution capsules
  -> Nsld deterministic graph, closure, NSB, and final-output pipeline
  -> registered native-shell provider or explicit blocked boundary
  -> nuis-runtime lifecycle/context dispatch
  -> run-artifact and Nsdb completion/replay evidence
  -> development-tensor handoff
```

The compiler owns common syntax and semantic contracts. Domain-specific ABI,
lowering, artifact, execution, and verification behavior remains registered
through Nustar capabilities rather than hardcoded as a finite backend matrix.

## Current Milestone Evidence

The live development tensor is the machine-readable source for detailed
progress. At this entry refresh it reports clean hierarchy, coverage, manifest,
milestone, and drift validation across every registered coordinate.

The current milestone slices include:

* stable compiler workflow, project orientation, and artifact-runtime closure
* stable type, control-flow, generic, std host-IO, filesystem, text, and
  task/thread/lock slices
* stable registered CFFI pointer/string/object admission and ownership slices
* stable registered Nustar domain contracts plus checked CUDA/Vulkan provider
  bring-up evidence
* stable NSB assembly and runtime lifecycle/context dispatch slices
* usable Galaxy source-import/lock resolution with remaining stabilization work
* an intentionally early, optional provider-neutral Data fabric lane

Here `stable` is the tensor protocol status for a defined milestone slice. It
does not promise language/API/ABI/package compatibility outside that slice.

## Nsld Frontier

Nsld now carries one deterministic ARM64 Mach-O route through:

```text
verified program/runtime objects
  -> merged placement and cross-object binding
  -> relocation and platform stub/GOT application
  -> final file/VM shell layout
  -> mach_header_64 and load-command serialization
  -> dyld rebase/bind, symbol, indirect-symbol, and string tables
  -> final-address instruction and pointer rewriting
  -> private deterministic shell image plus audit ledger
```

The private image deliberately stops at an empty `LC_CODE_SIGNATURE` payload.
It is not silently installed or advertised as the published runnable output.
The immediate bootstrap-critical work is to generate an audited ad-hoc
signature payload, independently validate Mach-O structure and signed ranges,
and emit an explicit publication-eligibility decision. ELF and PE/COFF remain
separate registered finalizer work, not Mach-O-shaped special cases in the core.

## Current Priorities

1. close the signed-private-image and independent validation gate without
   weakening compatibility output
2. finish Galaxy import/lock stabilization under larger real projects
3. keep provider finalization generic while adding ELF and PE/COFF backends
4. widen Data provider execution only through clock, GLM, ownership, and
   lifecycle-visible contracts
5. preserve focused tests, low-disk workflows, and tensor evidence while the
   beta foundation evolves

## Version And Documentation Rule

Use these sources in this order:

1. checked-in implementation and tests for exact behavior
2. `nuis dev-tensor --json` for current progress and next-action evidence
3. `docs/reference/` for present-tense protocol and boundary explanations
4. this file for the `beta-0.3.*` phase summary
5. older versioning entries only for historical reconstruction

Do not infer the current project line from individual Cargo package versions,
and do not copy absolute local paths, private server addresses, or one host OS
release into portable project contracts.
