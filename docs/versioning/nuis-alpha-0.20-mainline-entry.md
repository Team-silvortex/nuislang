# `nuis` `alpha-0.20.*` Mainline Entry

This file is the current short entry point for the `alpha-0.20.*` line.

`alpha-0.20.0` is the last alpha minor line. It is an alpha closeout and beta
foundation-readiness line, not a beta-stability claim.

Treat this as the alpha closeout and beta foundation-readiness checkpoint:
the current docs must keep Nustar backend-specific artifacts, the Nsld/NSB image
route, Nsdb trace/replay evidence, std pressure surfaces, and the development
tensor in one honest reading path.

Do not confuse this line with the historical pre-alpha `0.20.0` documents.
Files named `nuis-0.20.0-*` describe the final pre-alpha bootstrap transition.
Files named `nuis-alpha-0.20-*` describe the current alpha closeout mainline.

The direct predecessor is:

* [nuis-alpha-0.17-mainline-entry.md](nuis-alpha-0.17-mainline-entry.md)

Earlier executable, linking, std, tensor, and hardening anchors remain useful as
predecessor context:

* [nuis-alpha-0.16-mainline-entry.md](nuis-alpha-0.16-mainline-entry.md)
* [nuis-alpha-0.13-mainline-entry.md](nuis-alpha-0.13-mainline-entry.md)
* [nuis-alpha-0.10-mainline-entry.md](nuis-alpha-0.10-mainline-entry.md)
* [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
* [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
* [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
* [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)

Short rule:

`alpha-0.20.*` closes alpha by making the current compiler, std, Nustar,
Nsld, Nsdb, and development-tensor surfaces read as one beta-prep foundation.
The goal is not feature spectacle; the goal is that every real feature has a
registered route, a checked artifact, a trace/debug story, and a tensor status.

## Current Line Shape

Read the alpha progression as:

* `alpha-0.4.*` established the hardening baseline
* `alpha-0.6.*` introduced the named Nsld linker frontdoor
* `alpha-0.7.*` made std-backed tooling examples the default smoke surface
* `alpha-0.8.*` made binary-linking convergence the toolchain pressure
* `alpha-0.10.*` made executable-artifact closure the integration gate
* `alpha-0.13.*` broadened tensor-guided mainline hardening
* `alpha-0.16.*` made the development tensor the default steering surface
* `alpha-0.17.*` made registered heterogeneous worker execution the active
  integration gate
* `alpha-0.20.*` is the final alpha closeout line before beta foundation work

Current docs should use `alpha-0.20.*` for present-tense work. Older alpha
entries are predecessor or baseline context rather than competing current
routes.

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
12. [nuis-alpha-0.17-mainline-entry.md](nuis-alpha-0.17-mainline-entry.md)

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

The heterogeneous path should be read as an open registration path:

```text
source/module contract
  -> shared YIR/Nustar lowering contract
  -> provider-neutral artifact or source contract
  -> backend-owned emitter or execution adapter
  -> provider bundle / dispatch table / final-image binding proof
  -> Nsdb completion, replay, and debugger metadata
```

## Verified Truth Entering Alpha-0.20

The repository currently verifies or keeps active evidence for:

* frontdoor workflow/project/status commands with closure-summary plus tensor
  task-card orientation
* stable compiler-frontdoor, language-core, Nsld, native-binary, Nustar, std,
  PixelMagic, WitSage, CUDA, and development-tensor coordinates
* Nsld safe-next artifact-chain drive, final-output boundary reporting,
  self-contained NSB image paths, provider dispatch table validation, and
  final-image binding proof handoff
* Nsdb replay, cursor, provider-completion, trust-registry, and protected-anchor
  metadata surfaces
* registered Nustar provider bundles and code assets across Shader, Kernel,
  Data, CFFI, and host-compatible lanes
* real Metal/CoreML execution evidence on Apple hosts and real CUDA/Vulkan
  provider evidence on the Linux/NVIDIA lane where hardware is available
* canonical Shader body lowering shared across SPIR-V and MSL emitters, with
  the same copy/add/sub/mul `u32` contract feeding backend-specific artifacts
* std host IO/filesystem/text/tooling/task/result/network/report lanes that are
  broad enough to write real CLI-shaped examples
* PixelMagic and WitSage as official Galaxy pressure tests for image,
  shader/kernel, and classical ML routes
* UTF-8/LF source text policy, file-size hygiene, and milestone-backed
  development tensor coverage

This is real alpha integration evidence, but it is not final self-hosting and
not a beta compatibility promise.

## Main Target During Alpha-0.20

The highest-value target is alpha closeout discipline:

1. keep the development tensor clean and honest
2. keep all bootstrap-critical cells stable while new incomplete coordinates
   are registered explicitly
3. make generated heterogeneous artifacts consume shared contracts rather than
   backend-specific string islands
4. preserve Nsld and Nsdb as reusable toolchain surfaces, not CLI-only piles
5. keep CFFI/classic host compatibility as a registered domain rather than a
   linker special case
6. keep std, PixelMagic, and WitSage usable enough to pressure real examples
7. avoid claiming beta stability before import/package/self-hosting work has
   gone through its own beta line

## What Should Not Be Claimed Yet

`alpha-0.20.*` should not claim:

* final self-hosting
* beta-level API or package stability
* final std import/autoinjection semantics
* final production replacement for every host linker/debugger route
* complete ELF/PE/Mach-O parity
* production-complete GPU/NPU portability
* mature Ns Nova application/framework readiness

Safe wording:

* `alpha closeout`
* `beta foundation-readiness`
* `tensor-guided mainline`
* `registered heterogeneous provider closure`
* `self-contained NSB image route`
* `host-assisted final-output boundary`
* `shared Shader body contract for SPIR-V/MSL`
* `real provider evidence where hardware lanes are available`
* `std and official Galaxy pressure surfaces`
