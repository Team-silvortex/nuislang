# `nuis` `beta-0.0.1` Mainline Entry

> Historical phase snapshot. For current mainline behavior, start with
> [nuis-beta-0.6.0-mainline-entry.md](nuis-beta-0.6.0-mainline-entry.md).

This file is the recorded short entry point for the first beta line.

`beta-0.0.1` means the alpha architecture has crossed into a sustained
foundation-hardening phase. It does not mean stable public APIs, package
compatibility, self-hosting, or production-complete heterogeneous execution.

The direct predecessor is:

* [nuis-alpha-0.20-mainline-entry.md](nuis-alpha-0.20-mainline-entry.md)

The current self-hosting horizon remains:

```text
beta-0.0.1 through beta-0.9.*
  -> foundation closure, bug fixing, performance, and repeatability
beta-0.10.*
  -> formal stage0-to-stage1 self-hosting migration begins
gamma-0.5.*
  -> target stage2-equivalent compiler ownership
```

Short rule:

`beta-0.0.1 preserves the alpha architecture, but raises the evidence bar from
promising routes to repeatable, provider-backed, diagnosable closure`

## Canonical Reading Order

1. [../current-mainline-map.md](../../docs/current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../../docs/reference/nuis-development-tensor.md)
3. [../reference/nuis-native-artifact-workflow.md](../../docs/reference/nuis-native-artifact-workflow.md)
4. [../reference/nsld-linker-frontdoor.md](../../docs/reference/nsld-linker-frontdoor.md)
5. [../reference/nsld-binary-assembly-gap-map.md](../../docs/reference/nsld-binary-assembly-gap-map.md)
6. [../reference/nustar-multi-backend-artifact-contract.md](../../docs/reference/nustar-multi-backend-artifact-contract.md)
7. [../reference/nuis-binary-format-protocol.md](../../docs/reference/nuis-binary-format-protocol.md)
8. [../reference/nsdb-yir-debugger-frontdoor.md](../../docs/reference/nsdb-yir-debugger-frontdoor.md)
9. [../reference/provider-completion-trust-registry.md](../../docs/reference/provider-completion-trust-registry.md)
10. [../reference/std-mainline-layering-contract.md](../../docs/reference/std-mainline-layering-contract.md)
11. [../reference/std-shader-kernel-project-contract.md](../../docs/reference/std-shader-kernel-project-contract.md)
12. [nuis-alpha-0.20-mainline-entry.md](nuis-alpha-0.20-mainline-entry.md)

## Current Connected Spine

```text
nuis source / nuis.toml
  -> nuis frontdoor
  -> nuisc
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar packages and backend artifacts
  -> Nsld object / container / closure / final-output planning
  -> NSB image or explicit host-assisted final-output boundary
  -> run-artifact / Nsdb trace, replay, and completion evidence
  -> development tensor handoff
```

The compiler continues to know the shape of Nustar registration contracts, not
the internal policy of every backend. Backend-specific parsing, artifact
emission, execution adapters, and verification evidence remain registered
capabilities rather than compiler-wide conditionals.

## Verified Truth Entering Beta

The repository enters beta with checked evidence for:

* one project-to-NIR-to-YIR-to-AOT workflow frontdoor
* Nsld-owned object, container, closure, final-stage, NSB image, and explicit
  host-assisted finalization boundaries
* Nsdb trace, replay, provider-completion, trust-registry, and protected-anchor
  metadata surfaces
* registered CPU, Data, Shader, Kernel, Network, and CFFI domain contracts
* real Metal/CoreML execution on supported Apple hosts and real CUDA/Vulkan
  execution on the checked Linux/NVIDIA lane
* one canonical Shader compute body lowered into MSL and SPIR-V instead of
  treating backend source as unrelated string payloads
* fixed per-output Shader write extents with bounds-safe MSL/SPIR-V stores and
  real Vulkan validation of reduced secondary output carriers
* std host IO, filesystem, text, tooling, task, result, network, and report
  surfaces broad enough for CLI-shaped examples
* PixelMagic and WitSage as official Galaxy pressure surfaces
* UTF-8/LF source policy, repository line budgets, and a clean recursive
  development-tensor coverage/drift model

These are integration facts, not compatibility guarantees.

## Beta-0.0.1 Main Target

The first beta target is repeatability across the full heterogeneous route:

1. consume reduced provider outputs as typed downstream graph inputs without
   widening them back to parent dispatch extents
2. preserve GLM, clock, dependency, comparison, replay, and final-image
   evidence across multi-node provider graphs
3. keep Nsld and Nsdb provider-neutral while real backends mature independently
4. keep native C/object support inside the registered CFFI compatibility domain
5. harden package/import and std behavior under larger CLI and heterogeneous
   programs
6. find correctness and performance regressions before expanding more surface
   area
7. keep the development tensor synchronized after each implementation tranche

## Honesty Boundary

`beta-0.0.1` should not claim:

* stable public API or package-format compatibility
* final self-hosting
* final std import/autoinjection semantics
* complete ELF, PE/COFF, and Mach-O parity
* production-complete GPU/NPU portability
* replacement of every host linker, debugger, runtime, or device toolchain
* mature Ns Nova framework readiness

Safe wording:

* `early beta foundation hardening`
* `registered heterogeneous provider closure`
* `self-contained NSB image route`
* `Nsld/NSB image closure`
* `Nsdb trace/replay evidence`
* `host-assisted final-output boundary`
* `shared Shader body contract for MSL/SPIR-V`
* `real provider evidence on checked hardware lanes`
* `self-hosting migration begins at beta-0.10.*`

## Version Surface Rule

`beta-0.0.1` is the project release line. Cargo package versions may remain on
their internal workspace version until the repository adopts one explicit
package-version synchronization policy. Documentation must not infer the
project phase from an individual crate manifest.
